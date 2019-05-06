use crossterm::{ClearType, Color};

use crate::term::{Term, IN, OUT};

impl Term {
    pub fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.internal_cursor.col += s.len();
        self.terminal.write(s)?;
        Ok(())
    }
    pub fn write_in(&self) -> std::io::Result<()> {
        self.cursor.goto(0, self.cursor.pos().1)?;
        self.color.set_fg(Color::Yellow)?;
        self.terminal.write(IN)?;
        self.color.reset()?;
        Ok(())
    }
    pub fn write_out(&mut self) -> std::io::Result<()> {
        if !self.output.is_empty() {
            self.color.set_fg(Color::Red)?;
            self.terminal.write(OUT)?;
            self.color.reset()?;
            self.terminal
                .write(&self.output.drain(..).collect::<String>())?;
        }
        Ok(())
    }
    pub fn write_insert(&mut self, c: char) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.terminal.write(c)?;
        self.cursor.save_position()?;
        self.internal_cursor.right();

        for character in self.buffer[self.internal_cursor.col..].chars() {
            self.terminal.write(character)?;
        }
        self.cursor.reset_position()?;

        Ok(())
    }

    pub fn write_newline(&self) -> std::io::Result<()> {
        self.terminal.write('\n')?;
        Ok(())
    }

    pub fn reset_cursors(&mut self) -> std::io::Result<()> {
        self.internal_cursor.col = 0;
        self.cursor.goto(0, self.cursor.pos().1)?;
        Ok(())
    }

    pub fn backspace(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.cursor.save_position()?;

        for character in self.buffer[self.internal_cursor.col..].chars() {
            self.terminal.write(character)?;
        }
        self.cursor.reset_position()?;
        Ok(())
    }
}
