use irust_api::script4;
use rscript::{scripting::Scripter, Hook, ScriptType};
mod script;

struct Vim {
    state: State,
    mode: Mode,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
enum State {
    Empty,
    c,
    ci,
    d,
    di,
    g,
    f,
    F,
    r,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

impl Scripter for Vim {
    fn script_type() -> ScriptType {
        ScriptType::Daemon
    }

    fn name() -> &'static str {
        "Vim"
    }

    fn hooks() -> &'static [&'static str] {
        &[
            script4::InputEvent::NAME,
            script4::Shutdown::NAME,
            script4::Startup::NAME,
        ]
    }
}

fn main() {
    let mut vim = Vim::new();
    Vim::execute(&mut |hook_name| Vim::run(&mut vim, hook_name));
}

impl Vim {
    fn run(&mut self, hook_name: &str) {
        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        match hook_name {
            script4::InputEvent::NAME => {
                let hook: script4::InputEvent = bincode::deserialize_from(&mut stdin).unwrap();
                let output: Option<irust_api::Command> = self.handle_input_event(hook);
                bincode::serialize_into(&mut stdout, &output).unwrap();
            }
            script4::Shutdown::NAME => {
                let hook: script4::Shutdown = bincode::deserialize_from(&mut stdin).unwrap();
                let output: Option<irust_api::Command> = self.clean_up(hook);
                bincode::serialize_into(&mut stdout, &output).unwrap();
            }
            script4::Startup::NAME => {
                let hook: script4::Startup = bincode::deserialize_from(&mut stdin).unwrap();
                let output: Option<irust_api::Command> = self.start_up(hook);
                bincode::serialize_into(&mut stdout, &output).unwrap();
            }
            _ => unreachable!(),
        }
    }
}
