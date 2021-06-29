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
impl Prompt {}

fn main() {
    Prompt::greet();
    Prompt::execute(&mut |hook_name| Prompt::run(hook_name));
}

impl Prompt {
    fn prompt(global: GlobalVariables) -> String {
        format!("In [{}]: ", global.operation_number)
    }
    fn run(hook_name: &str) {
        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        match hook_name {
            script4::SetInputPrompt::NAME => {
                let script4::SetInputPrompt(global) =
                    bincode::deserialize_from(&mut stdin).unwrap();
                let output: String = Self::prompt(global);
                bincode::serialize_into(&mut stdout, &output).unwrap();
            }
            script4::SetOutputPrompt::NAME => {
                let script4::SetOutputPrompt(global) =
                    bincode::deserialize_from(&mut stdin).unwrap();
                let output: String = Self::prompt(global);
                bincode::serialize_into(&mut stdout, &output).unwrap();
            }
            script4::Shutdown::NAME => {
                let _hook: script4::Shutdown = bincode::deserialize_from(&mut stdin).unwrap();
                let output: Option<irust_api::Command> = Self::clean_up();
                bincode::serialize_into(&mut stdout, &output).unwrap();
            }
            _ => unreachable!(),
        }
    }
    fn clean_up() -> Option<irust_api::Command> {
        Some(irust_api::Command::ResetPrompt)
    }
}
