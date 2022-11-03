use irust_api::{Command, OutputEvent, Shutdown};
use rscript::scripting::Scripter;
use rscript::{Hook, VersionReq};

// need sync from crates/irust/src/irust/parser.rs
const COMMANDS: [&str; 32] = [
    ":reset",
    ":show",
    ":pop",
    ":irust",
    ":sync",
    ":exit",
    ":quit",
    ":help",
    "::",
    ":edit",
    ":add",
    ":hard_load_crate",
    ":hard_load",
    ":load",
    ":reload",
    ":type",
    ":del",
    ":dbg",
    ":color",
    ":cd",
    ":toolchain",
    ":main_result",
    ":check_statements",
    ":time_release",
    ":time",
    ":bench",
    ":asm",
    ":executor",
    ":evaluator",
    ":scripts",
    ":compile_time",
    ":expand",
];

fn split_cmds(buffer: String) -> Vec<String> {
    let mut new_buf = vec![];
    let mut tmp_vec = vec![];
    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        if COMMANDS.iter().any(|c| line.trim().starts_with(c)) {
            new_buf.push(tmp_vec.drain(..).collect::<Vec<_>>().join(""));
            new_buf.push(line.trim().to_owned());
        } else {
            tmp_vec.push(line);
        }
    }
    if !tmp_vec.is_empty() {
        new_buf.push(tmp_vec.drain(..).collect::<Vec<_>>().join(""));
    }
    new_buf
}

#[derive(Debug, Default)]
struct MixedCmds {}

impl Scripter for MixedCmds {
    fn name() -> &'static str {
        "MixedCmds"
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
    let _ = MixedCmds::execute(&mut |hook_name| MixedCmds::run(&mut MixedCmds {}, hook_name));
}

impl MixedCmds {
    fn run(&mut self, hook_name: &str) {
        match hook_name {
            OutputEvent::NAME => {
                let hook: OutputEvent = Self::read();
                let input = hook.1;
                let buffers = split_cmds(input);
                let cmds: Vec<_> = buffers.into_iter().map(Command::Parse).collect();
                let output = Some(Command::Multiple(cmds));
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

    fn clean_up(&self, _hook: Shutdown) -> Option<Command> {
        None
    }
}
