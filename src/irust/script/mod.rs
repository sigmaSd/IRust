use irust_api::GlobalVariables;

pub mod script1;
pub mod script2;

pub trait Script {
    fn input_prompt(&self, global_variables: &GlobalVariables) -> Option<String>;
    fn get_output_prompt(&self, global_variables: &GlobalVariables) -> Option<String>;
    fn while_compiling(&mut self, _global_variables: &GlobalVariables) -> Option<()> {
        None
    }
    fn after_compiling(&mut self) -> Option<()> {
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
    pub fn after_compiling_hook(&mut self) {
        if let Some(ref mut script_mg) = self.script_mg {
            script_mg.after_compiling();
        }
    }

    pub fn update_script_state(&mut self) {
        self.global_variables.prompt_position = self.printer.cursor.starting_pos();
    }
}
