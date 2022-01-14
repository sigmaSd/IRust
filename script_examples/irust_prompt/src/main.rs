use irust_api::GlobalVariables;
use rscript::{scripting::Scripter, Hook, ScriptType, VersionReq};

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
            irust_api::SetInputPrompt::NAME,
            irust_api::SetOutputPrompt::NAME,
            irust_api::Shutdown::NAME,
        ]
    }
    fn version_requirement() -> VersionReq {
        VersionReq::parse(">=1.50.0").expect("correct version requirement")
    }
}

enum PType {
    In,
    Out,
}
impl std::fmt::Display for PType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PType::In => write!(f, "In"),
            PType::Out => write!(f, "Out"),
        }
    }
}

impl Prompt {
    fn prompt(global: GlobalVariables, ptype: PType) -> String {
        format!("{} [{}]: ", ptype, global.operation_number)
    }
    fn run(hook_name: &str) {
        match hook_name {
            irust_api::SetInputPrompt::NAME => {
                let irust_api::SetInputPrompt(global) = Self::read();
                let output = Self::prompt(global, PType::In);
                Self::write::<irust_api::SetInputPrompt>(&output);
            }
            irust_api::SetOutputPrompt::NAME => {
                let irust_api::SetOutputPrompt(global) = Self::read();
                let output = Self::prompt(global, PType::Out);
                Self::write::<irust_api::SetOutputPrompt>(&output);
            }
            irust_api::Shutdown::NAME => {
                let _hook: irust_api::Shutdown = Self::read();
                let output = Self::clean_up();
                Self::write::<irust_api::Shutdown>(&output);
            }
            _ => unreachable!(),
        }
    }
    fn clean_up() -> Option<irust_api::Command> {
        Some(irust_api::Command::ResetPrompt)
    }
}

fn main() {
    let _ = Prompt::execute(&mut |hook_name| Prompt::run(hook_name));
}
