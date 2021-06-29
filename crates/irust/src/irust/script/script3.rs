use crossterm::event::Event;
use irust_api::{Command, GlobalVariables, Message};
use irust_api::{Hook, ScriptInfo};
use std::{cell::RefCell, collections::HashMap, io, rc::Rc};
use std::{fs, process::Stdio};

use std::{path::Path, process};
use std::{path::PathBuf, process::Child};

use super::Script;

type DaemonMap = HashMap<Hook, Vec<Rc<RefCell<Child>>>>;
type OneshotMap = HashMap<Hook, Vec<PathBuf>>;

pub struct ScriptManager3 {
    daemon_map: DaemonMap,
    oneshot_map: OneshotMap,
    meta: Vec<ScriptInfo>,
    //
    active_oneshot: HashMap<String, Child>,
}

impl Script for ScriptManager3 {
    fn while_compiling(&mut self, global_variables: &GlobalVariables) -> Option<()> {
        self.trigger_while_compiling_hook(global_variables)
    }

    fn input_event_hook(
        &mut self,
        global_variables: &GlobalVariables,
        event: Event,
    ) -> Option<Command> {
        self.trigger_input_event_hook(event, global_variables)
    }

    fn after_compile(&mut self) -> Option<()> {
        self.trigger_after_compile_hook()
    }

    fn input_prompt(&mut self, global_variables: &GlobalVariables) -> Option<String> {
        self.trigger_prompt_hook(Hook::SetInputPrompt, global_variables)
    }

    fn get_output_prompt(&mut self, global_variables: &GlobalVariables) -> Option<String> {
        self.trigger_prompt_hook(Hook::SetOutputPrompt, global_variables)
    }

    fn output_event_hook(&self, input: &str, global_variables: &GlobalVariables) -> Option<String> {
        self.trigger_output_event_hook(input, global_variables)
    }

    fn shutdown_hook(&mut self, global_variables: &GlobalVariables) -> Option<Command> {
        self.trigger_shutdown_hook(global_variables)
    }

    fn list(&self) -> Option<String> {
        if self.meta.is_empty() {
            return None;
        }
        let mut list = String::new();
        for meta in &self.meta {
            let line = format!(
                "{:10} || {:5} || {}",
                &meta.name,
                if meta.is_daemon { "daemon" } else { "oneshot" },
                &meta.path.display().to_string()
            );
            list.push_str(&line);
            list.push_str("\n\n");
        }
        // remove last \n\n
        list.pop();
        list.pop();

        Some(list)
    }
}

impl ScriptManager3 {
    pub fn new() -> Option<Self> {
        let script_path = dirs_next::config_dir()?.join("irust").join("script3");

        let (daemon_map, oneshot_map, meta) = look_for_scripts(&script_path).ok()?;
        Some(Self {
            daemon_map,
            oneshot_map,
            meta,
            active_oneshot: HashMap::new(),
        })
    }
    fn trigger_while_compiling_hook(&mut self, global_variables: &GlobalVariables) -> Option<()> {
        let mut taged_script = None;

        let mut oneshot_fn = |script_path: &Path| -> Option<()> {
            let mut script = process::Command::new(script_path)
                .stdin(Stdio::piped())
                .spawn()
                .ok()?;
            let mut stdin = script.stdin.as_mut().expect("stdin is piped");
            bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
            bincode::serialize_into(&mut stdin, &Hook::WhileCompiling).ok()?;
            bincode::serialize_into(&mut stdin, global_variables).ok()?;

            taged_script = Some(script);
            Some(())
        };
        let _ = self.trigger_hook(Hook::WhileCompiling, None, Some(&mut oneshot_fn));
        self.active_oneshot
            .insert("while_compiling".to_string(), taged_script?);
        Some(())
    }
    fn trigger_after_compile_hook(&mut self) -> Option<()> {
        if let Some(mut process) = self.active_oneshot.remove("while_compiling") {
            process.kill().ok()?;
        }
        Some(())
    }
    fn trigger_prompt_hook(
        &self,
        hook: Hook,
        global_variables: &GlobalVariables,
    ) -> Option<String> {
        let common_fn = |script: &mut Child| {
            let mut stdin = script.stdin.as_mut().expect("stdin is piped");
            bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
            bincode::serialize_into(&mut stdin, &hook).ok()?;
            bincode::serialize_into(&mut stdin, global_variables).ok()?;
            let stdout = script.stdout.as_mut().expect("stdout is piped");
            let prompt: String = bincode::deserialize_from(stdout).ok()?;
            Some(prompt)
        };

        let mut daemon_fn = |script: Rc<RefCell<Child>>| -> Option<String> {
            let mut script = script.borrow_mut();
            common_fn(&mut *script)
        };

        let mut oneshot_fn = |script_path: &Path| -> Option<String> {
            let mut script = process::Command::new(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;
            common_fn(&mut script)
        };

        let mut commands = self.trigger_hook(hook, Some(&mut daemon_fn), Some(&mut oneshot_fn));
        if !commands.is_empty() {
            // If different scripts want to set the prompt, just select the first one
            Some(commands.remove(0))
        } else {
            None
        }
    }
    fn trigger_output_event_hook(
        &self,
        input: &str,
        global_variables: &GlobalVariables,
    ) -> Option<String> {
        let hook = Hook::OutputEvent;

        let common_fn = |script: &mut Child| {
            let mut stdin = script.stdin.as_mut().expect("stdin is piped");
            bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
            bincode::serialize_into(&mut stdin, &hook).ok()?;
            bincode::serialize_into(&mut stdin, global_variables).ok()?;
            bincode::serialize_into(&mut stdin, &input).ok()?;
            let stdout = script.stdout.as_mut().expect("stdout is piped");
            let command: Option<String> = bincode::deserialize_from(stdout).ok()?;
            command
        };

        let mut daemon_fn = |script: Rc<RefCell<Child>>| {
            let mut script = script.borrow_mut();
            common_fn(&mut *script)
        };

        let mut oneshot_fn = |script_path: &Path| {
            let mut script = process::Command::new(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;
            common_fn(&mut script)
        };

        let mut commands = self.trigger_hook(hook, Some(&mut daemon_fn), Some(&mut oneshot_fn));
        if !commands.is_empty() {
            // if multiple scripts want to act on the output event, use the first result
            Some(commands.remove(0))
        } else {
            None
        }
    }

    fn trigger_shutdown_hook(&self, global_variables: &GlobalVariables) -> Option<Command> {
        let hook = Hook::Shutdown;

        let common_fn = |script: &mut Child| {
            let mut stdin = script.stdin.as_mut().expect("stdin is piped");
            bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
            bincode::serialize_into(&mut stdin, &hook).ok()?;
            bincode::serialize_into(&mut stdin, global_variables).ok()?;
            let stdout = script.stdout.as_mut().expect("stdout is piped");
            let command: Option<Command> = bincode::deserialize_from(stdout).ok()?;
            command
        };

        let mut daemon_fn = |script: Rc<RefCell<Child>>| -> Option<Command> {
            let mut script = script.borrow_mut();
            common_fn(&mut *script)
        };

        let mut oneshot_fn = |script_path: &Path| -> Option<Command> {
            let mut script = process::Command::new(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;
            common_fn(&mut script)
        };

        let commands = self.trigger_hook(hook, Some(&mut daemon_fn), Some(&mut oneshot_fn));
        if !commands.is_empty() {
            Some(Command::Multiple(commands))
        } else {
            None
        }
    }

    fn trigger_input_event_hook(
        &mut self,
        event: Event,
        global_variables: &GlobalVariables,
    ) -> Option<Command> {
        let hook = Hook::InputEvent;

        let common_fn = |script: &mut Child| {
            let mut stdin = script.stdin.as_mut().expect("stdin is piped");
            bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
            bincode::serialize_into(&mut stdin, &hook).ok()?;
            bincode::serialize_into(&mut stdin, global_variables).ok()?;
            bincode::serialize_into(&mut stdin, &event).ok()?;
            let stdout = script.stdout.as_mut().expect("stdout is piped");
            let command: Option<Command> = bincode::deserialize_from(stdout).ok()?;
            command
        };

        let mut daemon_fn = |script: Rc<RefCell<Child>>| -> Option<Command> {
            let mut script = script.borrow_mut();
            common_fn(&mut *script)
        };

        let mut oneshot_fn = |script_path: &Path| -> Option<Command> {
            let mut script = process::Command::new(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;
            common_fn(&mut script)
        };

        let commands = self.trigger_hook(hook, Some(&mut daemon_fn), Some(&mut oneshot_fn));
        if !commands.is_empty() {
            Some(Command::Multiple(commands))
        } else {
            None
        }
    }

    fn trigger_hook<T>(
        &self,
        hook: Hook,
        daemon_fn: Option<&mut dyn FnMut(Rc<RefCell<Child>>) -> Option<T>>,
        oneshot_fn: Option<&mut dyn FnMut(&Path) -> Option<T>>,
    ) -> Vec<T> {
        let mut commands = vec![];
        if let Some(daemon_fn) = daemon_fn {
            for (registered_hook, scripts) in self.daemon_map.iter() {
                if registered_hook == &hook {
                    for script in scripts {
                        if let Some(command) = daemon_fn(script.clone()) {
                            commands.push(command);
                        }
                    }
                }
            }
        }
        if let Some(oneshot_fn) = oneshot_fn {
            for (registered_hook, scripts_path) in self.oneshot_map.iter() {
                if registered_hook == &hook {
                    for script_path in scripts_path {
                        if let Some(command) = oneshot_fn(&script_path) {
                            commands.push(command);
                        }
                    }
                }
            }
        }
        commands
    }
}

fn look_for_scripts(dir: &Path) -> io::Result<(DaemonMap, OneshotMap, Vec<ScriptInfo>)> {
    let mut daemon_map = HashMap::new();
    let mut oneshot_map = HashMap::new();
    let mut meta = vec![];

    for entry in fs::read_dir(dir)? {
        (|| -> Option<()> {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let mut script = process::Command::new(path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .ok()?;

                let mut stdin = script.stdin.as_mut().expect("stdin is piped");
                bincode::serialize_into(&mut stdin, &Message::Greeting).ok()?;

                let stdout = script.stdout.as_mut().expect("stdout is piped");
                let info: ScriptInfo = bincode::deserialize_from(stdout).ok()?;

                meta.push(info.clone());
                if info.is_daemon {
                    let script = Rc::new(RefCell::new(script));
                    for hook in info.hooks {
                        let hook_slot = daemon_map.entry(hook).or_insert_with(Vec::new);
                        hook_slot.push(script.clone());
                    }
                } else {
                    // oneshot
                    for hook in info.hooks {
                        let hook_slot = oneshot_map.entry(hook).or_insert_with(Vec::new);
                        hook_slot.push(info.path.clone());
                    }
                }
            }
            Some(())
        })();
    }
    Ok((daemon_map, oneshot_map, meta))
}

impl Drop for ScriptManager3 {
    fn drop(&mut self) {
        for scripts in self.daemon_map.values() {
            for script in scripts {
                let _ = script.borrow_mut().kill();
            }
        }
    }
}
