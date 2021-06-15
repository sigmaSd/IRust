use crossterm::event::Event;
use irust_api::{Command, GlobalVariables};

use super::Script;

pub struct ScriptManager4(rscript::ScriptManager);

impl ScriptManager4 {
    pub fn new() -> Option<Self> {
        let mut sm = rscript::ScriptManager::default();
        let script_path = dirs_next::config_dir()?.join("irust").join("script4");
        sm.add_scripts_by_path(script_path).ok()?;
        Some(ScriptManager4(sm))
    }
}

impl Script for ScriptManager4 {
    fn input_event_hook(
        &mut self,
        global_variables: &GlobalVariables,
        event: Event,
    ) -> Option<Command> {
        self.0
            .trigger(irust_api::script4::InputEvent(
                global_variables.clone(),
                event,
            ))
            .next()?
            .ok()?
    }
    fn shutdown_hook(&mut self, global_variables: &GlobalVariables) -> Option<Command> {
        self.0
            .trigger(irust_api::script4::Shutdown(global_variables.clone()))
            .next()?
            .ok()?
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
    fn activate(&mut self, script_name: &str) -> Result<(), &'static str> {
        if let Some(script) = self
            .0
            .scripts_mut()
            .iter_mut()
            .find(|script| script.metadata().name == script_name)
        {
            script.activate();
            Ok(())
        } else {
            Err("Script not found")
        }
    }
    fn deactivate(&mut self, script_name: &str) -> Result<(), &'static str> {
        if let Some(script) = self
            .0
            .scripts_mut()
            .iter_mut()
            .find(|script| script.metadata().name == script_name)
        {
            script.deactivate();
            Ok(())
        } else {
            Err("Script not found")
        }
    }
}
