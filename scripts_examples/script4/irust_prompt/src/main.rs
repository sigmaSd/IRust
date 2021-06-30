use irust_api::{script4, GlobalVariables};
use rscript::{scripting::Scripter, Hook, ScriptType};

struct Prompt;

impl Scripter for Prompt {
    fn script_type() -> ScriptType {
        ScriptType::OneShot
    }

    fn name() -> &'static str {
        "prompt"
    }

    fn hooks() -> &'static [&'static str] {
        &[
            script4::SetInputPrompt::NAME,
            script4::SetOutputPrompt::NAME,
            script4::Shutdown::NAME,
        ]
    }
}

impl Prompt {
    fn prompt(global: GlobalVariables) -> String {
        format!("In [{}]: ", global.operation_number)
    }
    fn run(hook_name: &str) {
        match hook_name {
            script4::SetInputPrompt::NAME => {
                let script4::SetInputPrompt(global) = Self::read();
                let output: String = Self::prompt(global);
                Self::write(&output);
            }
            script4::SetOutputPrompt::NAME => {
                let script4::SetOutputPrompt(global) = Self::read();
                let output: String = Self::prompt(global);
                Self::write(&output);
            }
            script4::Shutdown::NAME => {
                let _hook: script4::Shutdown = Self::read();
                let output: Option<irust_api::Command> = Self::clean_up();
                Self::write(&output);
            }
            _ => unreachable!(),
        }
    }
    fn clean_up() -> Option<irust_api::Command> {
        Some(irust_api::Command::ResetPrompt)
    }
}

fn main() {
    Prompt::execute(&mut |hook_name| Prompt::run(hook_name));
}
