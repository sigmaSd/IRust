use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use irust_api::{Command, GlobalVariables, Hook, Message, ScriptInfo};
use serde::{de::DeserializeOwned, Serialize};

use std::io::{Stdin, Stdout, Write};

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

struct VimMode {
    active: bool,
    state: State,
    mode: Mode,
    stdin: Stdin,
    stdout: Stdout,
}

impl VimMode {
    fn new() -> Self {
        Self {
            active: true,
            state: State::Empty,
            mode: Mode::Insert,
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

        if !input.starts_with(":vim") {
            return self.write(&Option::<&str>::None);
        }

        let action = input.split_ascii_whitespace().nth(1);
        match action {
            Some("on") => {
                self.active = true;
                self.write(&Some("vim mode activated"))
            }
            Some("off") => {
                self.active = false;
                self.write(&Some("vim mode deactivated"))
            }
            _ => self.write(&Some(format!("vim mode state: {}", self.active))),
        }
    }

    fn handle_input_event(&mut self) -> bincode::Result<()> {
        let global = self.read::<GlobalVariables>()?;
        let event = self.read::<Event>()?;

        if !self.active {
            return self.write(&Option::<Command>::None);
        }

        macro_rules! reset_state {
            () => {
                self.state = State::Empty;
            };
        }

        let cmd = (|| match event {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers,
                } => {
                    if modifiers != KeyModifiers::NONE && modifiers != KeyModifiers::SHIFT {
                        return None;
                    }

                    if self.mode == Mode::Insert {
                        Some(Command::HandleCharacter(c))
                    } else {
                        match self.state {
                            State::f => return Some(Command::MoveForwardTillChar(c)),
                            State::F => return Some(Command::MoveBackwardTillChar(c)),
                            State::r => {
                                return Some(Command::Multiple(vec![
                                    Command::HandleDelete,
                                    Command::HandleCharacter(c),
                                    Command::HandleLeft,
                                ]))
                            }
                            State::ci => {
                                self.mode = Mode::Insert;
                                return Some(Command::Multiple(vec![
                                    Command::MoveBackwardTillChar(c),
                                    Command::HandleRight,
                                    Command::DeleteUntilChar(c, false),
                                ]));
                            }
                            State::di => {
                                return Some(Command::Multiple(vec![
                                    Command::MoveBackwardTillChar(c),
                                    Command::HandleRight,
                                    Command::DeleteUntilChar(c, false),
                                ]))
                            }
                            _ => (),
                        }
                        // Command Mode
                        match c {
                            'h' => Some(Command::HandleLeft),
                            'j' => Some(Command::HandleDown),
                            'k' => Some(Command::HandleUp),
                            'l' => Some(Command::HandleRight),
                            'b' => match self.state {
                                State::d => Some(Command::Multiple(vec![
                                    Command::HandleCtrlLeft,
                                    Command::DeleteNextWord,
                                ])),
                                State::c => {
                                    self.mode = Mode::Insert;
                                    Some(Command::Multiple(vec![
                                        Command::HandleCtrlLeft,
                                        Command::DeleteNextWord,
                                    ]))
                                }
                                _ => Some(Command::HandleCtrlLeft),
                            },
                            'w' => match self.state {
                                State::d => Some(Command::DeleteNextWord),
                                State::c => {
                                    self.mode = Mode::Insert;
                                    Some(Command::DeleteNextWord)
                                }
                                _ => Some(Command::HandleCtrlRight),
                            },
                            'g' => match self.state {
                                State::Empty => {
                                    self.state = State::g;
                                    Some(Command::Continue)
                                }
                                State::g => {
                                    reset_state!();
                                    let rows_diff =
                                        global.cursor_position.1 - global.prompt_position.1;
                                    Some(Command::Multiple(vec![Command::HandleUp; rows_diff]))
                                }
                                _ => {
                                    reset_state!();
                                    Some(Command::Continue)
                                }
                            },
                            'G' => {
                                if self.state == State::d {
                                    Some(Command::DeleteTillEnd)
                                } else {
                                    Some(Command::GoToLastRow)
                                }
                            }
                            'r' => {
                                if self.state == State::Empty {
                                    self.state = State::r;
                                }
                                Some(Command::Continue)
                            }
                            'x' => Some(Command::Multiple(vec![
                                Command::HandleDelete,
                                Command::PrintInput,
                            ])),
                            '$' => Some(Command::HandleEnd),
                            '^' => Some(Command::HandleHome),
                            'f' => match self.state {
                                State::Empty => {
                                    self.state = State::f;
                                    Some(Command::Continue)
                                }
                                _ => {
                                    reset_state!();
                                    Some(Command::Continue)
                                }
                            },
                            'F' => match self.state {
                                State::Empty => {
                                    self.state = State::F;
                                    Some(Command::Continue)
                                }
                                _ => {
                                    reset_state!();
                                    Some(Command::Continue)
                                }
                            },
                            'i' => match self.state {
                                State::c => {
                                    self.state = State::ci;
                                    Some(Command::Continue)
                                }
                                State::d => {
                                    self.state = State::di;
                                    Some(Command::Continue)
                                }
                                _ => {
                                    self.mode = Mode::Insert;
                                    Some(Command::SetThinCursor)
                                }
                            },
                            'I' => {
                                self.mode = Mode::Insert;
                                let commands = vec![Command::SetThinCursor, Command::HandleHome];
                                Some(Command::Multiple(commands))
                            }
                            'o' => {
                                self.mode = Mode::Insert;
                                let commands = vec![
                                    Command::SetThinCursor,
                                    Command::HandleEnd,
                                    Command::HandleAltEnter,
                                ];
                                Some(Command::Multiple(commands))
                            }
                            'a' => {
                                self.mode = Mode::Insert;
                                let commands = vec![Command::SetThinCursor, Command::HandleRight];
                                Some(Command::Multiple(commands))
                            }
                            'A' => {
                                self.mode = Mode::Insert;
                                let commands = vec![Command::SetThinCursor, Command::HandleEnd];
                                Some(Command::Multiple(commands))
                            }
                            'd' => match self.state {
                                State::Empty => {
                                    self.state = State::d;
                                    Some(Command::Continue)
                                }
                                State::d => {
                                    reset_state!();
                                    Some(Command::Multiple(vec![
                                        Command::HandleHome,
                                        Command::DeleteUntilChar('\n', true),
                                    ]))
                                }
                                _ => {
                                    reset_state!();
                                    Some(Command::Continue)
                                }
                            },
                            'D' => Some(Command::DeleteUntilChar('\n', false)),
                            'c' => match self.state {
                                State::Empty => {
                                    self.state = State::c;
                                    Some(Command::Continue)
                                }
                                State::c => {
                                    self.mode = Mode::Insert;
                                    reset_state!();
                                    Some(Command::Multiple(vec![
                                        Command::HandleHome,
                                        Command::DeleteUntilChar('\n', true),
                                    ]))
                                }
                                _ => {
                                    reset_state!();
                                    Some(Command::Continue)
                                }
                            },
                            'C' => {
                                self.mode = Mode::Insert;
                                Some(Command::DeleteUntilChar('\n', false))
                            }
                            _ => Some(Command::Continue),
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    self.mode = Mode::Normal;
                    Some(Command::SetWideCursor)
                }
                _ => None,
            },
            Event::Mouse(_) => None,
            Event::Resize(_, _) => None,
        })();

        // Second match to update the state
        if !matches!(cmd, Some(Command::Continue)) {
            reset_state!()
        }

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
        name: "Vim".into(),
        hooks: vec![Hook::InputEvent, Hook::OutputEvent],
        path: std::env::current_exe().unwrap(),
        is_daemon: true,
    };
    bincode::serialize_into(std::io::stdout(), &script_info).unwrap();
    std::io::stdout().flush().unwrap();

    let mut vim = VimMode::new();
    vim.run().unwrap()
}
