use crossterm::{Color, InputEvent, KeyEvent, Terminal, TerminalColor, TerminalInput};

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
pub mod options;
mod parser;
mod printer;
mod racer;
mod repl;
mod writer;
use cursor::Cursor;
use debouncer::Debouncer;
use history::History;
use irust_error::IRustError;
use options::Options;
use printer::{Printer, INPUT_START_COL};
use racer::Racer;
use repl::Repl;
mod buffer;
use buffer::Buffer;

const _IN: &str = "In: ";
const OUT: &str = "Out: ";

pub struct IRust {
    terminal: Terminal,
    printer: Printer,
    color: TerminalColor,
    buffer: Buffer,
    repl: Repl,
    cursor: Cursor,
    history: History,
    options: Options,
    racer: Result<Racer, IRustError>,
    debouncer: Debouncer,
    size: (usize, usize),
}

impl IRust {
    pub fn new() -> Self {
        let terminal = Terminal::new();
        let printer = Printer::default();
        let color = TerminalColor::new();
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
            let (width, height) = terminal.terminal_size();
            (width as usize, height as usize)
        };
        let cursor = Cursor::new(0, 0, size.0, size.1);

        IRust {
            cursor,
            terminal,
            printer,
            color,
            repl,
            history,
            options,
            racer,
            debouncer,
            size,
            buffer: Buffer::new(size.0 - INPUT_START_COL),
        }
    }

    fn prepare(&mut self) -> Result<(), IRustError> {
        self.repl.prepare_ground()?;
        self.debouncer.run();
        self.welcome()?;
        self.write_from_terminal_start("In: ", Color::Yellow)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), IRustError> {
        self.prepare()?;
        let mut stdin = TerminalInput::new().read_sync();
        let _screen = crossterm::RawScreen::into_raw_mode()?;

        loop {
            self.check_racer_callback()?;
            if let Some(key_event) = stdin.next() {
                match key_event {
                    InputEvent::Keyboard(KeyEvent::Char(c)) => match c {
                        '\n' => self.handle_enter()?,
                        '\t' => self.handle_tab()?,
                        c => self.handle_character(c)?,
                    },
                    InputEvent::Keyboard(KeyEvent::BackTab) => {
                        self.handle_back_tab()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Left) => {
                        self.handle_left()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Right) => {
                        self.handle_right()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Up) => {
                        self.handle_up()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Down) => {
                        self.handle_down()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Backspace) => {
                        self.handle_backspace()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('c')) => {
                        self.handle_ctrl_c()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('d')) => {
                        self.handle_ctrl_d()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('z')) => {
                        self.handle_ctrl_z()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('l')) => {
                        self.handle_ctrl_l()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Home) => {
                        self.handle_home_key()?;
                    }
                    InputEvent::Keyboard(KeyEvent::End) => {
                        self.handle_end_key()?;
                    }
                    InputEvent::Keyboard(KeyEvent::CtrlLeft) => {
                        self.handle_ctrl_left();
                    }
                    InputEvent::Keyboard(KeyEvent::CtrlRight) => {
                        self.handle_ctrl_right();
                    }
                    InputEvent::Keyboard(KeyEvent::Delete) => {
                        self.handle_del()?;
                    }
                    _ => {}
                }
            }
        }
    }
}
