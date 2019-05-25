use crossterm::{
    Crossterm, InputEvent, KeyEvent, Terminal, TerminalColor, TerminalCursor, TerminalInput,
};

use crate::history::History;
use crate::repl::Repl;
mod art;
mod cursor;
mod debouncer;
mod events;
mod format;
mod help;
pub mod options;
mod parser;
mod printer;
mod racer;
mod writer;
use cursor::Cursor;
use debouncer::Debouncer;
use options::Options;
use printer::Printer;
use racer::Racer;

const IN: &str = "In: ";
const OUT: &str = "Out: ";

pub struct IRust {
    cursor: TerminalCursor,
    terminal: Terminal,
    input: TerminalInput,
    printer: Printer,
    color: TerminalColor,
    buffer: String,
    repl: Repl,
    internal_cursor: Cursor,
    history: History,
    pub options: Options,
    racer: Option<Racer>,
    debouncer: Debouncer,
    size: (usize, usize),
}

impl IRust {
    pub fn new() -> Self {
        let crossterm = Crossterm::new();
        let cursor = crossterm.cursor();
        let terminal = crossterm.terminal();
        let input = crossterm.input();
        let printer = Printer::default();
        let color = crossterm.color();
        let buffer = String::new();
        let repl = Repl::new();
        let history = History::default();
        let internal_cursor = Cursor::new(0, 1, 4);
        let options = Options::new().unwrap_or_default();
        let debouncer = Debouncer::new();
        let racer = None;
        let size = {
            let (width, height) = terminal.terminal_size();
            (width as usize, height as usize)
        };

        IRust {
            cursor,
            terminal,
            input,
            printer,
            color,
            buffer,
            repl,
            history,
            options,
            internal_cursor,
            racer,
            debouncer,
            size,
        }
    }

    fn prepare(&mut self) -> std::io::Result<()> {
        self.repl.prepare_ground()?;
        self.start_racer();
        self.welcome()?;
        self.write_in()?;
        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.prepare()?;
        let mut stdin = self.input.read_sync();
        let _screen = crossterm::RawScreen::into_raw_mode()?;

        loop {
            if let Some(key_event) = stdin.next() {
                match key_event {
                    InputEvent::Keyboard(KeyEvent::Char(c)) => match c {
                        '\n' => self.handle_enter()?,
                        '\t' => self.handle_tab()?,
                        c => self.handle_character(c)?,
                    },
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
                        self.clear()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Home) => {
                        self.go_to_start()?;
                    }
                    InputEvent::Keyboard(KeyEvent::End) => {
                        self.go_to_end()?;
                    }
                    InputEvent::Keyboard(KeyEvent::CtrlLeft) => {
                        self.handle_ctrl_left();
                    }
                    InputEvent::Keyboard(KeyEvent::CtrlRight) => {
                        self.handle_ctrl_right();
                    }
                    _ => {}
                }
            }
        }
    }
}
