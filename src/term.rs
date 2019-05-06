use crossterm::{
    ClearType, Color, Crossterm, InputEvent, KeyEvent, Terminal, TerminalColor, TerminalCursor,
    TerminalInput,
};

use crate::history::History;
use crate::repl::Repl;
mod cursor;
mod events;
mod parser;
mod writer;
use cursor::Cursor;

const IN: &str = "In: ";
const OUT: &str = "Out: ";

pub struct Term {
    cursor: TerminalCursor,
    terminal: Terminal,
    input: TerminalInput,
    output: String,
    color: TerminalColor,
    buffer: String,
    repl: Repl,
    internal_cursor: Cursor,
    history: History,
}

impl Term {
    pub fn new() -> Self {
        let crossterm = Crossterm::new();
        let cursor = crossterm.cursor();
        let terminal = crossterm.terminal();
        let input = crossterm.input();
        let output = String::new();
        let color = crossterm.color();
        let buffer = String::new();
        let repl = Repl::new();
        let history = History::default();
        let internal_cursor = Cursor::new(0);

        Term {
            cursor,
            terminal,
            input,
            output,
            color,
            buffer,
            repl,
            history,
            internal_cursor,
        }
    }

    fn prepare(&self) -> std::io::Result<()> {
        self.repl.prepare_ground()?;
        self.terminal.clear(ClearType::All)?;

        self.color.set_fg(Color::Blue)?;
        let slash = std::iter::repeat('-')
            .take(self.terminal.terminal_size().0 as usize / 3)
            .collect::<String>();
        self.terminal
            .write(format!("       {0}Welcome to IRust{0}\n", slash))?;
        self.color.reset()?;

        self.write_in()?;
        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.prepare()?;
        let mut stdin = self.input.read_sync();

        loop {
            let _screen = crossterm::RawScreen::into_raw_mode()?;
            if let Some(key_event) = stdin.next() {
                match key_event {
                    InputEvent::Keyboard(KeyEvent::Char(c)) => {
                        self.handle_character(c)?;
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
                    InputEvent::Keyboard(KeyEvent::Esc) => self.terminal.exit(),
                    _ => (),
                }
            }
        }
    }
}
