use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use irust_api::Command;
use once_cell::sync::Lazy;

static SCRIPT_PATH: Lazy<PathBuf> = Lazy::new(|| std::env::temp_dir().join("irust_input_event"));

fn main() {
    if !SCRIPT_PATH.exists() {
        let _ = std::fs::create_dir_all(&*SCRIPT_PATH);
    }

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let _globals: irust_api::GlobalVariables = bincode::deserialize_from(&mut handle).unwrap();
    let event: Event = bincode::deserialize_from(handle).unwrap();

    let mode = Mode::get();
    let state = State::get();

    macro_rules! reset_state {
        () => {
            State::set(State::Empty);
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
                            Mode::set(Mode::Insert);
                            Some(Command::SetThinCursor)
                        }
                        'I' => {
                            Mode::set(Mode::Insert);
                            let commands = vec![Command::SetThinCursor, Command::HandleHome];
                            Some(Command::Multiple(commands))
                        }
                        'a' => {
                            Mode::set(Mode::Insert);
                            let commands = vec![Command::SetThinCursor, Command::HandleRight];
                            Some(Command::Multiple(commands))
                        }
                        'A' => {
                            Mode::set(Mode::Insert);
                            let commands = vec![Command::SetThinCursor, Command::HandleEnd];
                            Some(Command::Multiple(commands))
                        }
                        'd' => {
                            match state {
                                State::Empty => {
                                    State::set(State::d);
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
                Mode::set(Mode::Normal);
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
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, PartialEq)]
enum State {
    Empty,
    d,
}

impl State {
    fn get() -> Self {
        if let Ok(f) = std::fs::File::open(SCRIPT_PATH.join("state")) {
            bincode::deserialize_from(f).unwrap()
        } else {
            State::Empty
        }
    }
    fn set(state: State) {
        let f = std::fs::File::create(SCRIPT_PATH.join("state")).unwrap();
        bincode::serialize_into(f, &state).unwrap();
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

impl Mode {
    fn get() -> Self {
        if let Ok(f) = std::fs::File::open(SCRIPT_PATH.join("mode")) {
            bincode::deserialize_from(f).unwrap()
        } else {
            Mode::Insert
        }
    }
    fn set(mode: Mode) {
        let f = std::fs::File::create(SCRIPT_PATH.join("mode")).unwrap();
        bincode::serialize_into(f, &mode).unwrap();
    }
}
