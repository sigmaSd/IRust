use crossterm::event::Event;
use irust_api::{Command, GlobalVariables};

use self::{script1::ScriptManager, script2::ScriptManager2, script3::ScriptManager3};

use super::options::Options;

pub mod script1;
pub mod script2;
pub mod script3;

pub trait Script {
    fn input_prompt(&self, global_variables: &GlobalVariables) -> Option<String>;
    fn get_output_prompt(&self, global_variables: &GlobalVariables) -> Option<String>;
    fn while_compiling(&mut self, _global_variables: &GlobalVariables) -> Option<()> {
        None
    }
    fn input_event_hook(
        &mut self,
        _global_variables: &GlobalVariables,
        _event: Event,
    ) -> Option<Command> {
        None
    }

    fn after_compile(&mut self) -> Option<()> {
        None
    }
    fn output_event_hook(
        &self,
        _input: &str,
        _global_variables: &GlobalVariables,
    ) -> Option<String> {
        None
    }
}

// Scripts
impl super::IRust {
    pub fn update_input_prompt(&mut self) {
        if let Some(ref script_mg) = self.script_mg {
            if let Some(prompt) = script_mg.input_prompt(&self.global_variables) {
                self.printer.set_prompt(prompt);
            }
        }
    }
    pub fn get_output_prompt(&mut self) -> String {
        if let Some(ref script_mg) = self.script_mg {
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

    pub fn output_event_hook(
        &self,
        input: &str,
        global_variables: &GlobalVariables,
    ) -> Option<String> {
        if let Some(ref script_mg) = self.script_mg {
            return script_mg.output_event_hook(input, global_variables);
        }
        None
    }

    // internal
    ///////////
    ///
    pub fn choose_script_mg(options: &Options) -> Option<Box<dyn Script>> {
        if options.activate_scripting3 {
            ScriptManager3::new().map(|script_mg| Box::new(script_mg) as Box<dyn Script>)
        } else if options.activate_scripting2 {
            Some(Box::new(ScriptManager2::new()) as Box<dyn Script>)
        } else if options.activate_scripting {
            ScriptManager::new().map(|script_mg| Box::new(script_mg) as Box<dyn Script>)
        } else {
            None
        }
    }

    pub fn update_script_state(&mut self) {
        self.global_variables.prompt_position = self.printer.cursor.starting_pos();
        self.global_variables.is_racer_suggestion_active = self
            .racer
            .as_ref()
            .map(|r| r.active_suggestion.as_ref())
            .flatten()
            .is_some();
    }
}
