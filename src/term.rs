use crossterm::{
    ClearType, Color, Crossterm, InputEvent, KeyEvent, Terminal, TerminalColor, TerminalCursor,
    TerminalInput,
};

use crate::repl::Repl;
mod parser;

const IN: &str = "In: ";
const OUT: &str = "Out: ";

pub struct Term {
    cursor: TerminalCursor,
    terminal: Terminal,
    input: TerminalInput,
    color: TerminalColor,
    buffer: String,
    repl: Repl,
}

impl Term {
    pub fn new() -> Self {
        let crossterm = Crossterm::new();
        let cursor = crossterm.cursor();
        let terminal = crossterm.terminal();
        let input = crossterm.input();
        let color = crossterm.color();
        let buffer = String::new();
        let repl = Repl::new();

        Term {
            cursor,
            terminal,
            input,
            color,
            buffer,
            repl,
        }
    }
    pub fn new_in(&self) -> std::io::Result<()> {
        //self.cursor.goto(0, self.cursor.pos().1)?;
        self.color.set_fg(Color::Yellow)?;
        self.terminal.write(IN)?;
        self.color.reset()?;
        Ok(())
    }
    pub fn prepare(&self) -> std::io::Result<()> {
        self.repl.prepare_ground()?;
        self.terminal.clear(ClearType::All)?;

        self.color.set_fg(Color::Blue)?;
        let slash = std::iter::repeat('-')
            .take(self.terminal.terminal_size().0 as usize / 3)
            .collect::<String>();
        self.terminal
            .write(format!("       {0}Welcome to IRust{0}\n", slash))?;
        self.color.reset()?;

        self.new_in()?;
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
                        if c == '\n' {
                            self.handle_enter()?
                        } else {
                            //let col = cursor_pos as usize - 4;
                            self.terminal.write(c)?;
                            //self.insert_write(c, col)?;
                            self.buffer.push(c);

                            // if !self.buffer.is_empty() && (self.cursor.pos().0 as usize) != self.buffer.len() + 4 {
                            //     dbg!(self.buffer.len());
                            //     dbg!(self.cursor.pos());
                            //     self.insert_write(c)?;
                            //     self.buffer.insert(self.cursor.pos().0 as usize - 5 , c);
                            // } else {
                            //     dbg!(self.buffer.len());
                            //     dbg!(self.cursor.pos());
                            //     self.terminal.write(c)?;
                            // }

                        }
                    }
                    // InputEvent::Keyboard(KeyEvent::Left) => {
                    //     if self.cursor.pos().0 as usize > 4 {
                    //         self.cursor.move_left(1);
                    //     }
                    // },
                    // InputEvent::Keyboard(KeyEvent::Right) => {
                    //     if self.cursor.pos().0 as usize <= self.buffer.len() + 3 {
                    //         self.cursor.move_right(1);
                    //     }

                    // },
                    InputEvent::Keyboard(KeyEvent::Up) => {
                        self.cursor.move_up(1);
                    },
                    InputEvent::Keyboard(KeyEvent::Down) => {
                        self.cursor.move_down(1);
                    },
                    InputEvent::Keyboard(KeyEvent::Esc) => self.terminal.exit(),
                    _ => (),
                }
            }
        }

    }

    fn _insert_write(&mut self, c: char, col: usize) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.terminal.write(c)?;
        self.cursor.save_position()?;
        for character in self.buffer[col ..].chars() {
             self.terminal.write(character)?;
        }
        self.cursor.reset_position()?;
        Ok(())
    }

    fn handle_enter(&mut self) -> std::io::Result<()> {
        self.terminal.write('\n')?;
        self.cursor.goto(0, self.cursor.pos().1)?;
        self.parse()?;
        self.buffer.clear();
        self.terminal.write("\n")?;
        self.new_in()?;
        Ok(())
    }
}
