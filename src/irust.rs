mod art;
mod cargo_cmds;
mod cursor;
mod debouncer;
mod events;
mod format;
mod help;
mod highlight;
mod history;
mod irust_error;
mod known_paths;
pub mod options;
mod parser;
mod printer;
mod racer;
mod repl;
mod writer;
use crossterm::event::*;
use crossterm::{style::Color, terminal::enable_raw_mode};
use cursor::{Cursor, INPUT_START_COL};
use debouncer::Debouncer;
use history::History;
use irust_error::IRustError;
use options::Options;
use racer::Racer;
use repl::Repl;
mod buffer;
use buffer::Buffer;
mod raw_terminal;
use known_paths::KnownPaths;
use raw_terminal::RawTerminal;

const IN: &str = "In: ";
const OUT: &str = "Out: ";
pub const CTRL_KEYMODIFIER: crossterm::event::KeyModifiers =
    crossterm::event::KeyModifiers::CONTROL;
const ALT_KEYMODIFIER: crossterm::event::KeyModifiers = crossterm::event::KeyModifiers::ALT;
const SHIFT_KEYMODIFIER: crossterm::event::KeyModifiers = crossterm::event::KeyModifiers::SHIFT;
pub const NO_MODIFIER: crossterm::event::KeyModifiers = crossterm::event::KeyModifiers::empty();

pub struct IRust {
    raw_terminal: RawTerminal,
    buffer: Buffer,
    repl: Repl,
    cursor: Cursor,
    history: History,
    options: Options,
    racer: Result<Racer, IRustError>,
    debouncer: Debouncer,
    known_paths: KnownPaths,
}

impl IRust {
    pub fn new() -> Self {
        let raw_terminal = RawTerminal::new();
        let known_paths = KnownPaths::new();
        raw_terminal.set_title(&format!("IRust: {}", known_paths.get_cwd().display()));

        let repl = Repl::new();
        let history = History::new(dirs::cache_dir().unwrap().join("irust")).unwrap_or_default();
        let options = Options::new().unwrap_or_default();
        let debouncer = Debouncer::new();
        let racer = if options.enable_racer {
            Racer::start()
        } else {
            Err(IRustError::RacerDisabled)
        };
        let size = {
            let (width, height) = raw_terminal.size().expect("Error getting terminal size");
            (width as usize, height as usize)
        };
        let cursor = Cursor::new(size.0, size.1);
        let buffer = Buffer::new(size.0 - INPUT_START_COL);

        IRust {
            cursor,
            raw_terminal,
            repl,
            history,
            options,
            racer,
            debouncer,
            buffer,
            known_paths,
        }
    }

    fn prepare(&mut self) -> Result<(), IRustError> {
        self.repl.prepare_ground()?;
        self.debouncer.run();
        self.welcome()?;
        self.write_from_terminal_start(IN, Color::Yellow)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), IRustError> {
        enable_raw_mode()?;
        self.prepare()?;

        loop {
            self.check_racer_callback()?;
            if let Ok(key_event) = read() {
                match key_event {
                    Event::Mouse(_) => (),
                    Event::Resize(_width, _height) => (),
                    Event::Key(key_event) => match key_event {
                        KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers: NO_MODIFIER,
                        }
                        | KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers: SHIFT_KEYMODIFIER,
                        } => {
                            self.handle_character(c)?;
                        }
                        KeyEvent {
                            code: KeyCode::Enter,
                            ..
                        } => {
                            self.handle_enter()?;
                        }
                        KeyEvent {
                            code: KeyCode::Tab, ..
                        } => {
                            self.handle_tab()?;
                        }
                        KeyEvent {
                            code: KeyCode::BackTab,
                            ..
                        } => {
                            self.handle_back_tab()?;
                        }
                        KeyEvent {
                            code: KeyCode::Left,
                            modifiers: NO_MODIFIER,
                        } => {
                            self.handle_left()?;
                        }
                        KeyEvent {
                            code: KeyCode::Right,
                            modifiers: NO_MODIFIER,
                        } => {
                            self.handle_right()?;
                        }
                        KeyEvent {
                            code: KeyCode::Up, ..
                        } => {
                            self.handle_up()?;
                        }
                        KeyEvent {
                            code: KeyCode::Down,
                            ..
                        } => {
                            self.handle_down()?;
                        }
                        KeyEvent {
                            code: KeyCode::Backspace,
                            ..
                        } => {
                            self.handle_backspace()?;
                        }
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_c()?;
                        }
                        KeyEvent {
                            code: KeyCode::Char('d'),
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_d()?;
                        }
                        KeyEvent {
                            code: KeyCode::Char('z'),
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_z()?;
                        }
                        KeyEvent {
                            code: KeyCode::Char('l'),
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_l()?;
                        }
                        KeyEvent {
                            code: KeyCode::Char('r'),
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_r()?;
                        }
                        KeyEvent {
                            code: KeyCode::Char('\r'),
                            modifiers: ALT_KEYMODIFIER,
                        } => {
                            self.handle_alt_enter()?;
                        }
                        KeyEvent {
                            code: KeyCode::Home,
                            ..
                        } => {
                            self.handle_home_key()?;
                        }
                        KeyEvent {
                            code: KeyCode::End, ..
                        } => {
                            self.handle_end_key()?;
                        }
                        KeyEvent {
                            code: KeyCode::Left,
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_left();
                        }
                        KeyEvent {
                            code: KeyCode::Right,
                            modifiers: CTRL_KEYMODIFIER,
                        } => {
                            self.handle_ctrl_right();
                        }
                        KeyEvent {
                            code: KeyCode::Delete,
                            ..
                        } => {
                            self.handle_del()?;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
