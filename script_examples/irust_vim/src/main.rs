use rscript::{scripting::Scripter, Hook, ScriptType, VersionReq};
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
            irust_api::InputEvent::NAME,
            irust_api::Shutdown::NAME,
            irust_api::Startup::NAME,
        ]
    }
    fn version_requirement() -> VersionReq {
        VersionReq::parse(">=1.32.0").expect("correct version requirement")
    }
}

fn main() {
    let mut vim = Vim::new();
    Vim::execute(&mut |hook_name| Vim::run(&mut vim, hook_name));
}

impl Vim {
    fn run(&mut self, hook_name: &str) {
        match hook_name {
            irust_api::InputEvent::NAME => {
                let hook: irust_api::InputEvent = Self::read();
                let output = self.handle_input_event(hook);
                Self::write::<irust_api::InputEvent>(&output);
            }
            irust_api::Shutdown::NAME => {
                let hook: irust_api::Shutdown = Self::read();
                let output = self.clean_up(hook);
                Self::write::<irust_api::Shutdown>(&output);
            }
            irust_api::Startup::NAME => {
                let hook: irust_api::Startup = Self::read();
                let output = self.start_up(hook);
                Self::write::<irust_api::Startup>(&output);
            }
            _ => unreachable!(),
        }
    }
}
