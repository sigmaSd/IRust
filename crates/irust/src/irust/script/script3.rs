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
}

impl Script for ScriptManager3 {
    fn while_compiling(&mut self, _global_variables: &GlobalVariables) -> Option<()> {
        None
    }

    fn input_event_hook(
        &mut self,
        global_variables: &GlobalVariables,
        event: Event,
    ) -> Option<Command> {
        self.trigger_input_event_hook(event, global_variables)
    }

    fn after_compiling(&mut self) -> Option<()> {
        None
    }

    fn input_prompt(&self, _global_variables: &GlobalVariables) -> Option<String> {
        None
    }

    fn get_output_prompt(&self, _global_variables: &GlobalVariables) -> Option<String> {
        None
    }
}

impl ScriptManager3 {
    pub fn new() -> Option<Self> {
        let script_path = dirs_next::config_dir()?.join("irust").join("script3");

        let (daemon_map, oneshot_map) = look_for_scripts(&script_path).ok()?;
        Some(Self {
            daemon_map,
            oneshot_map,
        })
    }

    pub fn trigger_input_event_hook(
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

        let daemon_fn = |script: Rc<RefCell<Child>>| -> Option<Command> {
            let mut script = script.borrow_mut();
            common_fn(&mut *script)
        };

        let oneshot_fn = |script_path: &Path| -> Option<Command> {
            let mut script = process::Command::new(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;
            common_fn(&mut script)
        };

        let commands = self.trigger_hook(hook, &daemon_fn, &oneshot_fn);
        if !commands.is_empty() {
            Some(Command::Multiple(commands))
        } else {
            None
        }
    }

    pub fn trigger_hook<T>(
        &mut self,
        hook: Hook,
        daemon_fn: &dyn Fn(Rc<RefCell<Child>>) -> Option<T>,
        oneshot_fn: &dyn Fn(&Path) -> Option<T>,
    ) -> Vec<T> {
        let mut commands = vec![];
        for (registered_hook, scripts) in self.daemon_map.iter() {
            if registered_hook == &hook {
                for script in scripts {
                    if let Some(command) = daemon_fn(script.clone()) {
                        commands.push(command);
                    }
                }
            }
        }
        for (registered_hook, scripts_path) in self.oneshot_map.iter() {
            if registered_hook == &hook {
                for script_path in scripts_path {
                    if let Some(command) = oneshot_fn(&script_path) {
                        commands.push(command);
                    }
                }
            }
        }
        commands
    }
}

fn look_for_scripts(dir: &Path) -> io::Result<(DaemonMap, OneshotMap)> {
    let mut daemon_map = HashMap::new();
    let mut oneshot_map = HashMap::new();

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
    Ok((daemon_map, oneshot_map))
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
