use std::collections::HashMap;

use crossterm::event::Event;
use irust_api::{Command, GlobalVariables};

use super::Script;

pub struct ScriptManager4(rscript::ScriptManager);

macro_rules! mtry {
    ($e: expr) => {
        (|| -> Result<_, Box<dyn std::error::Error>> { Ok($e) })()
    };
}
impl ScriptManager4 {
    pub fn new() -> Option<Self> {
        let mut sm = rscript::ScriptManager::default();
        let script_path = dirs_next::config_dir()?.join("irust").join("script4");
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

        // read conf if available
        let script_conf_path = dirs_next::config_dir()?.join("irust").join("script4.conf");

        // ignore any error that happens while trying to read conf
        // If an error happens, a new configuration will be written anyway when ScriptManager is
        // dropped

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
                    } else {
                        script.deactivate();
                    }
                }
            })
        }

        Some(ScriptManager4(sm))
    }
}
impl Drop for ScriptManager4 {
    fn drop(&mut self) {
        let mut script_state = HashMap::new();
        for script in self.0.scripts() {
            script_state.insert(script.metadata().name.clone(), script.is_active());
        }
        // Ignore errors on drop
        let _ = mtry!({
            let script_conf_path = dirs_next::config_dir()
                .ok_or("could not find config directory")?
                .join("irust")
                .join("script4.conf");
            std::fs::write(script_conf_path, toml::to_string(&script_state)?)
        });
    }
}

/* NOTE: Toml: serilizing tuple struct is not working?
#[derive(Serialize, Deserialize, Debug)]
struct ScriptState(HashMap<String, bool>);
*/

impl Script for ScriptManager4 {
    fn input_event_hook(
        &mut self,
        global_variables: &GlobalVariables,
        event: Event,
    ) -> Option<Command> {
        self.0
            .trigger(irust_api::InputEvent(global_variables.clone(), event))
            .next()?
            .ok()?
    }
    fn output_event_hook(
        &mut self,
        input: &str,
        global_variables: &GlobalVariables,
    ) -> Option<String> {
        self.0
            .trigger(irust_api::OutputEvent(
                global_variables.clone(),
                input.to_string(),
            ))
            .next()?
            .ok()?
    }
    fn shutdown_hook(&mut self, global_variables: &GlobalVariables) -> Vec<Option<Command>> {
        self.0
            .trigger(irust_api::Shutdown(global_variables.clone()))
            .filter_map(Result::ok)
            .collect()
    }
    fn get_output_prompt(&mut self, global_variables: &GlobalVariables) -> Option<String> {
        self.0
            .trigger(irust_api::SetOutputPrompt(global_variables.clone()))
            .next()?
            .ok()
    }
    fn input_prompt(&mut self, global_variables: &GlobalVariables) -> Option<String> {
        self.0
            .trigger(irust_api::SetInputPrompt(global_variables.clone()))
            .next()?
            .ok()
    }
    fn list(&self) -> Option<String> {
        let mut scripts: Vec<String> = self
            .0
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
    fn activate(
        &mut self,
        script_name: &str,
        global_variables: &GlobalVariables,
    ) -> Result<Option<Command>, &'static str> {
        if let Some(script) = self
            .0
            .scripts_mut()
            .iter_mut()
            .find(|script| script.metadata().name == script_name)
        {
            script.activate();
            // We send a startup message in case the script is listening for one
            if let Ok(maybe_command) = script.trigger(&irust_api::Startup(global_variables.clone()))
            {
                Ok(maybe_command)
            } else {
                Ok(None)
            }
        } else {
            Err("Script not found")
        }
    }
    fn deactivate(
        &mut self,
        script_name: &str,
        global_variables: &GlobalVariables,
    ) -> Result<Option<Command>, &'static str> {
        if let Some(script) = self
            .0
            .scripts_mut()
            .iter_mut()
            .find(|script| script.metadata().name == script_name)
        {
            script.deactivate();
            // We send a shutdown message in case the script is listening for one
            if let Ok(maybe_command) =
                script.trigger(&irust_api::Shutdown(global_variables.clone()))
            {
                Ok(maybe_command)
            } else {
                Ok(None)
            }
        } else {
            Err("Script not found")
        }
    }

    fn while_compiling(&mut self, _global_variables: &GlobalVariables) -> Option<()> {
        None
    }

    fn after_compile(&mut self) -> Option<()> {
        None
    }
}
