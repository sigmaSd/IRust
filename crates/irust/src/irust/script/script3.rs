use irust_api::{Command, HookData, Message};
use irust_api::{Hook, ScriptInfo};
use std::{cell::RefCell, collections::HashMap, io, rc::Rc};
use std::{fs, process::Stdio};

use std::{path::Path, process};
use std::{path::PathBuf, process::Child};

type DaemonMap = HashMap<Hook, Vec<Rc<RefCell<Child>>>>;
type OneshotMap = HashMap<Hook, Vec<PathBuf>>;

pub struct ScriptManager3 {
    daemon_map: DaemonMap,
    oneshot_map: OneshotMap,
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

    pub fn trigger_hook(&mut self, hook: Hook, hook_data: HookData) -> Option<Command> {
        let mut commands = vec![];
        for (registered_hook, scripts) in self.daemon_map.iter() {
            if registered_hook == &hook {
                for script in scripts {
                    if let Some(command) = (|| -> Option<Command> {
                        let mut script = script.borrow_mut();
                        let mut stdin = script.stdin.as_mut().expect("stdin is piped");
                        bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
                        bincode::serialize_into(&mut stdin, &hook).ok()?;
                        bincode::serialize_into(&mut stdin, &hook_data).ok()?;
                        let stdout = script.stdout.as_mut().expect("stdout is piped");
                        let command: Option<Command> = bincode::deserialize_from(stdout).ok()?;
                        command
                    })() {
                        commands.push(command);
                    };
                }
            }
        }
        for (registered_hook, scripts_path) in self.oneshot_map.iter() {
            if registered_hook == &hook {
                for script_path in scripts_path {
                    if let Some(command) = (|| -> Option<Command> {
                        let mut script = process::Command::new(script_path)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                            .ok()?;
                        let mut stdin = script.stdin.as_mut().expect("stdin is piped");
                        bincode::serialize_into(&mut stdin, &Message::Hook).ok()?;
                        bincode::serialize_into(&mut stdin, &hook).ok()?;
                        bincode::serialize_into(&mut stdin, &hook_data).ok()?;

                        let stdout = script.stdout.as_mut().expect("stdout is piped");
                        let command: Option<Command> = bincode::deserialize_from(stdout).ok()?;
                        command
                    })() {
                        commands.push(command);
                    }
                }
            }
        }
        if commands.is_empty() {
            None
        } else {
            Some(Command::Multiple(commands))
        }
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
