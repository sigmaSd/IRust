use std::collections::HashMap;

use irust_api::{color, Command, OutputEvent, Shutdown};
use rscript::scripting::Scripter;
use rscript::{Hook, VersionReq};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Fun {
    functions: HashMap<String, String>,
}

impl Scripter for Fun {
    fn name() -> &'static str {
        "Fun"
    }

    fn script_type() -> rscript::ScriptType {
        rscript::ScriptType::Daemon
    }

    fn hooks() -> &'static [&'static str] {
        &[OutputEvent::NAME, Shutdown::NAME]
    }

    fn version_requirement() -> rscript::VersionReq {
        VersionReq::parse(">=1.50.0").expect("correct version requirement")
    }
}

fn main() {
    let mut fun = Fun::new();
    let _ = Fun::execute(&mut |hook_name| Fun::run(&mut fun, hook_name));
}

impl Fun {
    fn run(&mut self, hook_name: &str) {
        match hook_name {
            OutputEvent::NAME => {
                let hook: OutputEvent = Self::read();
                let output = match self.handle_output_event(hook) {
                    Ok(out) => out,
                    Err(e) => Some(Command::PrintOutput(
                        e.to_string() + "\n",
                        color::Color::Red,
                    )),
                };
                Self::write::<OutputEvent>(&output);
            }
            Shutdown::NAME => {
                let hook: Shutdown = Self::read();
                let output = self.clean_up(hook);
                Self::write::<Shutdown>(&output);
            }
            _ => unreachable!(),
        }
    }
    fn new() -> Self {
        let functions = (|| {
            let fns =
                std::fs::read_to_string(dirs::config_dir()?.join("irust/functions.toml")).ok()?;
            toml::from_str(&fns).ok()
        })()
        .unwrap_or_default();

        Self { functions }
    }
    fn handle_output_event(&mut self, hook: OutputEvent) -> Result<<OutputEvent as Hook>::Output> {
        let input = hook.1;
        if !(input.starts_with(":fun") || input.starts_with(":f")) {
            return Ok(None);
        }

        let buffer = input;
        match buffer.splitn(4, ' ').collect::<Vec<_>>().as_slice() {
            [_, "def" | "d", name, fun] => {
                self.functions.insert(name.to_string(), fun.to_string());
                Ok(Some(Command::PrintOutput("Ok!".into(), color::Color::Blue)))
            }
            [_, name, invoke_arg @ ..] => {
                let mut function = self
                    .functions
                    .get(*name)
                    .ok_or(format!("function: `{}` is not defined", name))?
                    .to_owned();

                for (idx, arg) in invoke_arg.iter().enumerate() {
                    let arg_tag = "$arg".to_string() + &idx.to_string();
                    function = function.replacen(&arg_tag, arg, 1);
                }

                Ok(Some(Command::Parse(function)))
            }
            _ => Err("Incorrect usage of `fun`".into()),
        }
    }
    fn clean_up(&self, _hook: Shutdown) -> Option<Command> {
        (|| -> Option<()> {
            std::fs::write(
                dirs::config_dir()?.join("irust/functions.toml"),
                toml::to_string(&self.functions).ok()?,
            )
            .ok()
        })();
        None
    }
}
