use std::io::{Write, Stdin, Stdout};

use serde::{Serialize, de::DeserializeOwned};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use irust_api::{Command, GlobalVariables, Hook, Message, ScriptInfo};

enum State {
    Prefix,
    Normal,
}

struct EmacsMode {
    active: bool,
    state: State,

    stdin: Stdin,
    stdout: Stdout,
}

impl EmacsMode {
    fn new() -> Self {
        Self {
            active: true,
            state: State::Normal,
            stdin: std::io::stdin(),
            stdout: std::io::stdout(),
        }
    }

    fn read<T: DeserializeOwned>(&mut self) -> bincode::Result<T> {
        bincode::deserialize_from(&mut self.stdin)
    }

    fn write<T: Serialize>(&mut self, value: &T) -> bincode::Result<()> {
        bincode::serialize_into(&mut self.stdout, value)
    }

    fn handle_output_event(&mut self) -> bincode::Result<()> {
        let _ = self.read::<GlobalVariables>()?;
        let input = self.read::<String>()?;

        if !input.starts_with(":emacs") {
            return self.write(&Option::<&str>::None);
        }

        let action = input.split_ascii_whitespace().nth(1);
        match action {
            Some("on") => {
                self.active = true;
                self.write(&Some("Emacs mode activated"))
            }
            Some("off") => {
                self.active = false;
                self.write(&Some("Emacs mode deactivated"))
            }
            _ => {
                self.write(&Some(format!("Emacs mode state: {}", self.active)))
            }
        }
    }

    fn handle_input_event(&mut self) -> bincode::Result<()> {
        let global = self.read::<GlobalVariables>()?;
        let event = self.read::<Event>()?;

        if !self.active {
            return self.write(&Option::<Command>::None);
        }

        let cmd = match event {
            Event::Key(key) => match key {
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char(c),
                } => {
                    let ret = match c {
                        'a' => Some(Command::HandleHome),
                        'e' => match self.state {
                            State::Prefix => Some(Command::HandleCtrlE),
                            State::Normal => {
                                if global.is_racer_suggestion_active {
                                    Some(Command::AcceptSuggestion)
                                } else {
                                    Some(Command::HandleEnd)
                                }
                            },
                        },
                        'n' => Some(Command::HandleDown),
                        'p' => Some(Command::HandleUp),
                        'b' => Some(Command::HandleLeft),
                        'f' => Some(Command::HandleRight),
                        'g' => {
                            self.state = State::Normal;
                            Some(Command::RemoveRacerSugesstion)
                        },
                        'x' => {
                            self.state = State::Prefix;
                            None
                        }
                        _ => None,
                    };
                    if c != 'x' {
                        self.state = State::Normal;
                    }
                    ret
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => Some(Command::HandleCharacter(c)),
                _ => None,
            },
            _ => None,
        };

        self.write(&cmd)
    }

    fn run(&mut self) -> bincode::Result<()> {
        loop {
            let _ = self.read::<Message>()?;
            let hook = self.read::<Hook>()?;

            match hook {
                Hook::InputEvent => self.handle_input_event()?,
                Hook::OutputEvent => self.handle_output_event()?,
                _ => unreachable!(),
            }

            self.stdout.flush().unwrap();
        }
    }
}

fn main() {
    let message: Message = bincode::deserialize_from(&mut std::io::stdin()).unwrap();
    assert_eq!(message, Message::Greeting);

    let script_info = ScriptInfo {
        name: "Emacs".into(),
        hooks: vec![Hook::InputEvent, Hook::OutputEvent],
        path: std::env::current_exe().unwrap(),
        is_daemon: true,
    };
    bincode::serialize_into(std::io::stdout(), &script_info).unwrap();
    std::io::stdout().flush().unwrap();

    let mut emacs = EmacsMode::new();
    emacs.run().unwrap()
}
