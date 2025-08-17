use std::collections::HashMap;

use crossterm::event::Event;
use irust_api::{Command, GlobalVariables};

use super::Script;

const SCRIPT_CONFIG_NAME: &str = "script.toml";

pub struct ScriptManager {
    sm: rscript::ScriptManager,
    startup_cmds: Vec<Result<Option<Command>, rscript::Error>>,
}

macro_rules! mtry {
    ($e: expr) => {
        (|| -> Result<_, Box<dyn std::error::Error>> { Ok($e) })()
    };
}
impl ScriptManager {
    pub fn new() -> Option<Self> {
        let mut sm = rscript::ScriptManager::default();
        let script_path = crate::utils::irust_dirs::config_dir()?
            .join("irust")
            .join("script");
        sm.add_scripts_by_path(
            &script_path,
            rscript::Version::parse(crate::args::VERSION).expect("correct version"),
        )
        .ok()?;
        unsafe {
            sm.add_dynamic_scripts_by_path(
                script_path,
                rscript::Version::parse(crate::args::VERSION).expect("correct version"),
            )
            .ok()?;
        }
        dbg!("a");

        // read conf if available
        let script_conf_path = crate::utils::irust_dirs::config_dir()?
            .join("irust")
            .join(SCRIPT_CONFIG_NAME);

        // ignore any error that happens while trying to read conf
        // If an error happens, a new configuration will be written anyway when ScriptManager is
        // dropped

        let mut startup_cmds = vec![];
        if let Ok(script_state) =
            mtry!(toml::from_str(&std::fs::read_to_string(script_conf_path)?)?)
        {
            // type inference
            let script_state: HashMap<String, bool> = script_state;

            sm.scripts_mut().iter_mut().for_each(|script| {
                let script_name = &script.metadata().name;
                if let Some(state) = script_state.get(script_name) {
                    if *state {
                        script.activate();
                        // Trigger startup hook, in case the script needs to be aware of it
                        if script.is_listening_for::<irust_api::Startup>() {
                            startup_cmds.push(script.trigger(&irust_api::Startup()));
                        }
                    } else {
                        script.deactivate();
                    }
                }
            })
        }

        dbg!("azz");
        Some(ScriptManager { sm, startup_cmds })
    }
}
impl Drop for ScriptManager {
    fn drop(&mut self) {
        let mut script_state = HashMap::new();
        for script in self.sm.scripts() {
            script_state.insert(script.metadata().name.clone(), script.is_active());
        }
        // Ignore errors on drop
        let _ = mtry!({
            let script_conf_path = crate::utils::irust_dirs::config_dir()
                .ok_or("could not find config directory")?
                .join("irust")
                .join(SCRIPT_CONFIG_NAME);
            std::fs::write(script_conf_path, toml::to_string(&script_state)?)
        });
    }
}

/* NOTE: Toml: serilizing tuple struct is not working?
#[derive(Serialize, Deserialize, Debug)]
struct ScriptState(HashMap<String, bool>);
*/

impl Script for ScriptManager {
    fn input_prompt(&mut self, global_variables: &GlobalVariables) -> Option<String> {
        self.sm
            .trigger(irust_api::SetInputPrompt(global_variables.clone()))
            .next()?
            .ok()
    }
    fn get_output_prompt(&mut self, global_variables: &GlobalVariables) -> Option<String> {
        self.sm
            .trigger(irust_api::SetOutputPrompt(global_variables.clone()))
            .next()?
            .ok()
    }
    fn before_compiling(&mut self, global_variables: &GlobalVariables) -> Option<()> {
        self.sm
            .trigger(irust_api::BeforeCompiling(global_variables.clone()))
            .collect::<Result<_, _>>()
            .ok()
    }
    fn after_compiling(&mut self, global_variables: &GlobalVariables) -> Option<()> {
        self.sm
            .trigger(irust_api::AfterCompiling(global_variables.clone()))
            .collect::<Result<_, _>>()
            .ok()
    }
    fn input_event_hook(
        &mut self,
        global_variables: &GlobalVariables,
        event: Event,
    ) -> Option<Command> {
        self.sm
            .trigger(irust_api::InputEvent(global_variables.clone(), event))
            .next()?
            .ok()?
    }
    fn output_event_hook(
        &mut self,
        input: &str,
        global_variables: &GlobalVariables,
    ) -> Option<Command> {
        self.sm
            .trigger(irust_api::OutputEvent(
                global_variables.clone(),
                input.to_string(),
            ))
            .next()?
            .ok()?
    }
    fn trigger_set_title_hook(&mut self) -> Option<String> {
        self.sm.trigger(irust_api::SetTitle()).next()?.ok()?
    }

    fn trigger_set_msg_hook(&mut self) -> Option<String> {
        self.sm.trigger(irust_api::SetWelcomeMsg()).next()?.ok()?
    }

    fn list(&self) -> Option<String> {
        let mut scripts: Vec<String> = self
            .sm
            .scripts()
            .iter()
            .map(|script| {
                let meta = script.metadata();
                format!(
                    "{}\t{:?}\t{:?}\t{}",
                    &meta.name,
                    &meta.script_type,
                    &meta.hooks,
                    script.is_active()
                )
            })
            .collect();
        //header
        scripts.insert(0, "Name\tScriptType\tHooks\tState".into());

        Some(scripts.join("\n"))
    }

    fn activate(&mut self, script_name: &str) -> Result<Option<Command>, &'static str> {
        if let Some(script) = self
            .sm
            .scripts_mut()
            .iter_mut()
            .find(|script| script.metadata().name == script_name)
        {
            script.activate();
            // We send a startup message in case the script is listening for one
            if let Ok(maybe_command) = script.trigger(&irust_api::Startup()) {
                Ok(maybe_command)
            } else {
                Ok(None)
            }
        } else {
            Err("Script not found")
        }
    }

    fn deactivate(&mut self, script_name: &str) -> Result<Option<Command>, &'static str> {
        if let Some(script) = self
            .sm
            .scripts_mut()
            .iter_mut()
            .find(|script| script.metadata().name == script_name)
        {
            script.deactivate();
            // We send a shutdown message in case the script is listening for one
            if let Ok(maybe_command) = script.trigger(&irust_api::Shutdown()) {
                Ok(maybe_command)
            } else {
                Ok(None)
            }
        } else {
            Err("Script not found")
        }
    }

    fn startup_cmds(&mut self) -> Vec<Result<Option<Command>, rscript::Error>> {
        self.startup_cmds.drain(..).collect()
    }

    fn shutdown_cmds(&mut self) -> Vec<Result<Option<Command>, rscript::Error>> {
        self.sm
            .scripts_mut()
            .iter_mut()
            .filter(|script| script.is_listening_for::<irust_api::Shutdown>())
            .map(|script| script.trigger(&irust_api::Shutdown()))
            .collect()
    }
}
