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
        match hook_name {
            script4::InputEvent::NAME => {
                let hook: script4::InputEvent = Self::read();
                let output: Option<irust_api::Command> = self.handle_input_event(hook);
                Self::write(&output);
            }
            script4::Shutdown::NAME => {
                let hook: script4::Shutdown = Self::read();
                let output: Option<irust_api::Command> = self.clean_up(hook);
                Self::write(&output);
            }
            script4::Startup::NAME => {
                let hook: script4::Startup = Self::read();
                let output: Option<irust_api::Command> = self.start_up(hook);
                Self::write(&output);
            }
            _ => unreachable!(),
        }
    }
}
