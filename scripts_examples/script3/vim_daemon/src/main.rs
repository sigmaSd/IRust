use std::io::Write;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use irust_api::{Command, GlobalVariables, Hook, Message, ScriptInfo};

fn main() {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let message: Message = bincode::deserialize_from(&mut handle).unwrap();
    assert_eq!(message, Message::Greeting);

    if message == Message::Greeting {
        let script_info = ScriptInfo {
            hooks: vec![Hook::InputEvent],
            path: std::env::current_exe().unwrap(),
            is_daemon: true,
        };
        bincode::serialize_into(std::io::stdout(), &script_info).unwrap();
        std::io::stdout().flush().unwrap();
    }

    let mut mode = Mode::Insert;
    let mut state = State::Empty;

    macro_rules! reset_state {
        () => {
            state = State::Empty;
        };
    }
    loop {
        // message is Message::Hook
        let _message: Message = bincode::deserialize_from(&mut handle).unwrap();
        let hook: Hook = bincode::deserialize_from(&mut handle).unwrap();
        assert_eq!(hook, Hook::InputEvent);
        let _g: GlobalVariables = bincode::deserialize_from(&mut handle).unwrap();
        let event: Event = bincode::deserialize_from(&mut handle).unwrap();

        let cmd = (|| match event {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers,
                } => {
                    if modifiers != KeyModifiers::NONE && modifiers != KeyModifiers::SHIFT {
                        return None;
                    }

                    if mode == Mode::Insert {
                        Some(Command::HandleCharacter(c))
                    } else {
                        // Command Mode
                        match c {
                            'h' => Some(Command::HandleLeft),
                            'j' => Some(Command::HandleDown),
                            'k' => Some(Command::HandleUp),
                            'l' => Some(Command::HandleRight),
                            'b' => {
                                if state == State::d {
                                    Some(Command::Multiple(vec![
                                        Command::HandleCtrlLeft,
                                        Command::DeleteNextWord,
                                    ]))
                                } else {
                                    Some(Command::HandleCtrlLeft)
                                }
                            }
                            'w' => {
                                if state == State::d {
                                    Some(Command::DeleteNextWord)
                                } else {
                                    Some(Command::HandleCtrlRight)
                                }
                            }
                            'x' => Some(Command::HandleDelete),
                            '$' => Some(Command::HandleEnd),
                            '^' => Some(Command::HandleHome),
                            'i' => {
                                mode = Mode::Insert;
                                Some(Command::SetThinCursor)
                            }
                            'I' => {
                                mode = Mode::Insert;
                                let commands = vec![Command::SetThinCursor, Command::HandleHome];
                                Some(Command::Multiple(commands))
                            }
                            'a' => {
                                mode = Mode::Insert;
                                let commands = vec![Command::SetThinCursor, Command::HandleRight];
                                Some(Command::Multiple(commands))
                            }
                            'A' => {
                                mode = Mode::Insert;
                                let commands = vec![Command::SetThinCursor, Command::HandleEnd];
                                Some(Command::Multiple(commands))
                            }
                            'd' => {
                                match state {
                                    State::Empty => {
                                        state = State::d;
                                        Some(Command::Continue)
                                    }
                                    State::d => {
                                        reset_state!();
                                        //TODO: ADD cut line command
                                        Some(Command::Multiple(vec![
                                            Command::HandleHome,
                                            Command::DeleteUntilNewLine(true),
                                        ]))
                                    }
                                }
                            }
                            'D' => Some(Command::DeleteUntilNewLine(false)),
                            _ => Some(Command::Continue),
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    mode = Mode::Normal;
                    Some(Command::SetWideCursor)
                }
                _ => None,
            },
            Event::Mouse(_) => None,
            Event::Resize(_, _) => None,
        })();

        // Second match to update the state
        if !matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE
            }) | Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::SHIFT
            })
        ) {
            reset_state!()
        }

        bincode::serialize_into(std::io::stdout(), &cmd).unwrap();
        std::io::stdout().flush().unwrap();
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
enum State {
    Empty,
    d,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}
