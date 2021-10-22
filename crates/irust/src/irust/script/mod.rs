use crossterm::event::Event;
use irust_api::{Command, GlobalVariables};

use self::script4::ScriptManager4;

use super::options::Options;

pub mod script4;

pub trait Script {
    fn input_prompt(&mut self, _global_variables: &GlobalVariables) -> Option<String>;
    fn get_output_prompt(&mut self, _global_variables: &GlobalVariables) -> Option<String>;
    fn while_compiling(&mut self, _global_variables: &GlobalVariables) -> Option<()>;
    fn input_event_hook(
        &mut self,
        _global_variables: &GlobalVariables,
        _event: Event,
    ) -> Option<Command>;
    fn after_compile(&mut self) -> Option<()>;
    fn output_event_hook(
        &mut self,
        _input: &str,
        _global_variables: &GlobalVariables,
    ) -> Option<String>;
    fn shutdown_hook(&mut self, _global_variables: &GlobalVariables) -> Vec<Option<Command>>;
    fn list(&self) -> Option<String>;
    fn activate(
        &mut self,
        _script: &str,
        _global_variables: &GlobalVariables,
    ) -> Result<Option<Command>, &'static str>;
    fn deactivate(
        &mut self,
        _script: &str,
        _global_variables: &GlobalVariables,
    ) -> Result<Option<Command>, &'static str>;
}

// Scripts
impl super::IRust {
    pub fn update_input_prompt(&mut self) {
        if let Some(ref mut script_mg) = self.script_mg {
            if let Some(prompt) = script_mg.input_prompt(&self.global_variables) {
                self.printer.set_prompt(prompt);
            }
        }
    }
    pub fn get_output_prompt(&mut self) -> String {
        if let Some(ref mut script_mg) = self.script_mg {
            if let Some(prompt) = script_mg.get_output_prompt(&self.global_variables) {
                return prompt;
            }
        }
        //Default
        self.options.output_prompt.clone()
    }
    pub fn while_compiling_hook(&mut self) {
        if let Some(ref mut script_mg) = self.script_mg {
            script_mg.while_compiling(&self.global_variables);
        }
    }
    pub fn input_event_hook(&mut self, event: Event) -> Option<Command> {
        if let Some(ref mut script_mg) = self.script_mg {
            return script_mg.input_event_hook(&self.global_variables, event);
        }
        None
    }
    pub fn after_compiling_hook(&mut self) {
        if let Some(ref mut script_mg) = self.script_mg {
            script_mg.after_compile();
        }
    }

    pub fn output_event_hook(&mut self, input: &str) -> Option<String> {
        if let Some(ref mut script_mg) = self.script_mg {
            return script_mg.output_event_hook(input, &self.global_variables);
        }
        None
    }

    pub fn shutdown_hook(&mut self) -> Vec<Option<Command>> {
        if let Some(ref mut script_mg) = self.script_mg {
            return script_mg.shutdown_hook(&self.global_variables);
        }
        vec![]
    }

    pub fn scripts_list(&self) -> Option<String> {
        if let Some(ref script_mg) = self.script_mg {
            return script_mg.list();
        }
        None
    }
    pub fn activate_script(&mut self, script: &str) -> Result<Option<Command>, &'static str> {
        if let Some(ref mut script_mg) = self.script_mg {
            return script_mg.activate(script, &self.global_variables);
        }
        Ok(None)
    }
    pub fn deactivate_script(&mut self, script: &str) -> Result<Option<Command>, &'static str> {
        if let Some(ref mut script_mg) = self.script_mg {
            return script_mg.deactivate(script, &self.global_variables);
        }
        Ok(None)
    }

    // internal
    ///////////
    ///
    pub fn choose_script_mg(options: &Options) -> Option<Box<dyn Script>> {
        if options.activate_scripting {
            ScriptManager4::new().map(|script_mg| Box::new(script_mg) as Box<dyn Script>)
        } else {
            None
        }
    }

    pub fn update_script_state(&mut self) {
        self.global_variables.prompt_position = self.printer.cursor.starting_pos();
        self.global_variables.cursor_position = self.printer.cursor.current_pos();
        self.global_variables.is_racer_suggestion_active = self
            .racer
            .as_ref()
            .map(|r| r.active_suggestion.as_ref())
            .flatten()
            .is_some();
    }
}
