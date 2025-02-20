use std::{
    io::{Read, Write},
    process::{ChildStdin, Command, Stdio},
    sync::mpsc,
};

use irust_api::color;
use rscript::{Hook, ScriptType, VersionReq, scripting::Scripter};

#[derive(Default)]
struct IPython {
    stdin: Option<ChildStdin>,
    stdout: Option<mpsc::Receiver<String>>,
}

impl Scripter for IPython {
    fn script_type() -> ScriptType {
        ScriptType::Daemon
    }

    fn name() -> &'static str {
        "IPython"
    }

    fn hooks() -> &'static [&'static str] {
        &[
            irust_api::SetTitle::NAME,
            irust_api::SetWelcomeMsg::NAME,
            irust_api::OutputEvent::NAME,
            irust_api::Startup::NAME,
            irust_api::Shutdown::NAME,
        ]
    }
    fn version_requirement() -> VersionReq {
        VersionReq::parse(">=1.50.0").expect("correct version requirement")
    }
}

fn main() {
    let mut ipython = IPython::default();
    let _ = IPython::execute(&mut |hook_name| IPython::run(&mut ipython, hook_name));
}

impl IPython {
    fn run(&mut self, hook_name: &str) {
        match hook_name {
            irust_api::OutputEvent::NAME => {
                let hook: irust_api::OutputEvent = Self::read();
                let output = self.handle_output_event(hook);
                Self::write::<irust_api::OutputEvent>(&output);
            }
            irust_api::SetTitle::NAME => {
                let _hook: irust_api::SetTitle = Self::read();
                Self::write::<irust_api::SetTitle>(&Some("IPython".to_string()));
            }
            irust_api::SetWelcomeMsg::NAME => {
                let _hook: irust_api::SetWelcomeMsg = Self::read();
                Self::write::<irust_api::SetWelcomeMsg>(&Some("IPython".to_string()));
            }
            irust_api::Startup::NAME => {
                let _hook: irust_api::Startup = Self::read();
                self.clean_up();
                *self = Self::start();
                Self::write::<irust_api::Startup>(&None);
            }
            irust_api::Shutdown::NAME => {
                let _hook: irust_api::Shutdown = Self::read();
                self.clean_up();
                Self::write::<irust_api::Shutdown>(&None);
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn handle_output_event(
        &mut self,
        hook: irust_api::OutputEvent,
    ) -> <irust_api::OutputEvent as Hook>::Output {
        if hook.1.starts_with(':') {
            return None;
        }

        if self.stdin.is_none() {
            *self = Self::start();
        }

        let stdin = self.stdin.as_mut().unwrap();
        let stdout = self.stdout.as_mut().unwrap();

        let input = hook.1 + "\n";
        stdin.write_all(input.as_bytes()).unwrap();
        stdin.flush().unwrap();
        let now = std::time::Instant::now();
        while now.elapsed() < std::time::Duration::from_millis(200) {
            if let Ok(out) = stdout.try_recv() {
                // Expression Or Statement
                if out.is_empty() {
                    return Some(irust_api::Command::PrintOutput(
                        "()\n".to_string(),
                        color::Color::Blue,
                    ));
                } else {
                    return Some(irust_api::Command::PrintOutput(
                        out + "\n",
                        color::Color::Blue,
                    ));
                }
            }
        }
        // Statement
        Some(irust_api::Command::PrintOutput(
            "()\n".to_string(),
            color::Color::Blue,
        ))
    }

    pub(crate) fn clean_up(&mut self) {
        // IPython could have already exited
        // So we ignore errors
        let _ = self.stdin.as_mut().map(|stdin| stdin.write_all(b"exit\n"));
        let _ = self.stdin.as_mut().map(|stdin| stdin.flush());
    }

    fn start() -> IPython {
        #[expect(clippy::zombie_processes)]
        let mut p = Command::new("ipython")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = p.stdin.take().unwrap();
        let mut stdout = p.stdout.take().unwrap();

        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut buf = [0; 512];
            let _ = stdout.read(&mut buf).unwrap();
            let _ = stdout.read(&mut buf).unwrap();
            tx.send(String::new()).unwrap();

            let mut out = String::new();

            loop {
                let n = stdout.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }

                let o = String::from_utf8(buf[..n].to_vec()).unwrap();
                out += &o;

                // Use prompt as delimiter
                if !out.contains("\nIn ") {
                    continue;
                }

                // Post Process
                let o = {
                    let mut o: Vec<_> = out.lines().collect();
                    o.pop();
                    let mut o = o.join("\n");
                    if o.contains("...:") {
                        o = o.rsplit("...:").next().unwrap().to_owned();
                    }
                    o
                };

                // Send output and clear it for the next read
                tx.send(o.trim().to_owned()).unwrap();
                out.clear();
            }
        });
        // Wait for IPython to start
        rx.recv().unwrap();

        IPython {
            stdin: Some(stdin),
            stdout: Some(rx),
        }
    }
}
