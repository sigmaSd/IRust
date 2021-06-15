use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use irust_api::{script4, Command};
use rscript::{Hook, ScriptType};

use std::io::{stdout, Write};

struct Vim {
    state: State,
    mode: Mode,
}
impl Vim {
    fn new() -> Self {
        Self {
            state: State::Empty,
            mode: Mode::Normal,
        }
    }
    fn clean_up(&mut self, _: script4::Shutdown) -> <script4::Shutdown as Hook>::Output {
        Some(Command::SetWideCursor)
    }
    fn handle_input_event(
        &mut self,
        input_event: script4::InputEvent,
    ) -> <script4::InputEvent as Hook>::Output {
        let script4::InputEvent(global, event) = input_event;
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
                                    Command::SetThinCursor,
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
                                        Command::SetThinCursor,
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
                                    Some(Command::Multiple(vec![
                                        Command::SetThinCursor,
                                        Command::DeleteNextWord,
                                    ]))
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
                                        Command::SetThinCursor,
                                        Command::HandleHome,
                                        Command::DeleteUntilChar('\n', false),
                                    ]))
                                }
                                _ => {
                                    reset_state!();
                                    Some(Command::Continue)
                                }
                            },
                            'C' => {
                                self.mode = Mode::Insert;
                                Some(Command::Multiple(vec![
                                    Command::SetThinCursor,
                                    Command::DeleteUntilChar('\n', false),
                                ]))
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

        cmd
    }
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

fn main() {
    let message: rscript::Message = bincode::deserialize_from(std::io::stdin()).unwrap();
    assert_eq!(message, rscript::Message::Greeting);
    let metadata = rscript::ScriptInfo::new(
        "Vim",
        ScriptType::Daemon,
        &[script4::InputEvent::NAME, script4::Shutdown::NAME],
    );
    bincode::serialize_into(std::io::stdout(), &metadata).unwrap();
    std::io::stdout().flush().unwrap();

    let mut vim = Vim::new();

    loop {
        let _message: rscript::Message = bincode::deserialize_from(std::io::stdin()).unwrap();
        let hook_name: String = bincode::deserialize_from(std::io::stdin()).unwrap();
        match hook_name.as_str() {
            script4::InputEvent::NAME => {
                let hook: script4::InputEvent =
                    bincode::deserialize_from(std::io::stdin()).unwrap();
                let output = vim.handle_input_event(hook);
                bincode::serialize_into(stdout(), &output).unwrap();
            }
            script4::Shutdown::NAME => {
                let hook: script4::Shutdown = bincode::deserialize_from(std::io::stdin()).unwrap();
                let output = vim.clean_up(hook);
                bincode::serialize_into(stdout(), &output).unwrap();
            }
            _ => unreachable!(),
        }

        std::io::stdout().flush().unwrap();
    }
}
