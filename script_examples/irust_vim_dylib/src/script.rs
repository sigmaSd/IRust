use irust_api::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use irust_api::Command;
use rscript::Hook;

use super::{Mode, State, Vim};

impl Vim {
    pub const fn new() -> Self {
        Self {
            state: State::Empty,
            mode: Mode::Insert,
        }
    }
    pub fn start_up(&mut self, _: irust_api::Startup) -> <irust_api::Startup as Hook>::Output {
        self.state = State::Empty;
        self.mode = Mode::Insert;
        Some(Command::SetThinCursor)
    }
    pub fn clean_up(&mut self, _: irust_api::Shutdown) -> <irust_api::Shutdown as Hook>::Output {
        self.state = State::Empty;
        self.mode = Mode::Normal;
        Some(Command::SetWideCursor)
    }
    pub fn handle_input_event(
        &mut self,
        input_event: irust_api::InputEvent,
    ) -> <irust_api::InputEvent as Hook>::Output {
        let irust_api::InputEvent(global, event) = input_event;
        macro_rules! reset_state {
            () => {{
                self.state = State::Empty;
            }};
        }

        let cmd = (|| match event {
            Event::Key(key) => match key {
                KeyEvent {
                    kind: KeyEventKind::Release,
                    ..
                } => None,
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers,
                    ..
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
            Event::FocusGained => None,
            Event::FocusLost => None,
            Event::Paste(_) => None,
        })();

        // Second match to update the state
        if !matches!(cmd, Some(Command::Continue)) {
            reset_state!()
        }

        cmd
    }
}
