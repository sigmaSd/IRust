use crossterm::{ClearType, Color};

use crate::term::{Term, IN, OUT};

impl Term {
    pub fn write_str_at(
        &mut self,
        s: &str,
        x: Option<usize>,
        y: Option<usize>,
    ) -> std::io::Result<()> {
        self.go_to_cursor_at(x, y)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.internal_cursor.x += s.len();
        self.terminal.write(s)?;
        Ok(())
    }
    pub fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.internal_cursor.x += s.len();
        self.terminal.write(s)?;
        Ok(())
    }
    pub fn write_in(&mut self) -> std::io::Result<()> {
        self.go_to_cursor()?;
        self.color.set_fg(Color::Yellow)?;
        self.terminal.write(IN)?;
        self.color.reset()?;
        Ok(())
    }
    pub fn write_out(&mut self) -> std::io::Result<()> {
        if !self.output.is_empty() {
            self.go_to_cursor()?;
            self.color.set_fg(Color::Red)?;
            self.terminal.write(OUT)?;
            self.color.reset()?;
            let out = self.output.drain(..).collect::<String>();
            out.split('\n').for_each(|o| {
                let _ = self.terminal.write(o);
                let _ = self.write_newline();
            });
        }

        Ok(())
    }
    pub fn write_insert(&mut self, c: char) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;

        self.terminal.write(c)?;
        self.cursor.save_position()?;
        self.internal_cursor.right();

        for character in self.buffer[self.internal_cursor.x..].chars() {
            self.terminal.write(character)?;
        }
        self.cursor.reset_position()?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> std::io::Result<()> {
        self.terminal.write('\n')?;
        self.internal_cursor.x = 0;
        self.internal_cursor.y += 1;
        self.go_to_cursor()?;
        Ok(())
    }

    pub fn reset_cursors(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.go_to_cursor()?;
        Ok(())
    }

    pub fn cursors_to_origin(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.internal_cursor.y = 0;
        self.go_to_cursor()?;
        Ok(())
    }

    pub fn backspace(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.cursor.save_position()?;

        for character in self.buffer[self.internal_cursor.x..].chars() {
            self.terminal.write(character)?;
        }
        self.cursor.reset_position()?;
        Ok(())
    }

    pub fn welcome(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::All)?;

        self.color.set_fg(Color::Blue)?;
        let slash = std::iter::repeat('-')
            .take(self.terminal.terminal_size().0 as usize / 3)
            .collect::<String>();

        self.terminal
            .write(format!("       {0}Welcome to IRust{0}\n", slash))?;

        self.color.reset()?;

        Ok(())
    }

    fn go_to_cursor_at(&mut self, x: Option<usize>, y: Option<usize>) -> std::io::Result<()> {
        if let Some(x) = x {
            self.internal_cursor.x = x;
        }
        if let Some(y) = y {
            self.internal_cursor.y = y;
        }

        self.go_to_cursor()?;

        Ok(())
    }

    fn go_to_cursor(&mut self) -> std::io::Result<()> {
        self.cursor
            .goto(self.internal_cursor.x as u16, self.internal_cursor.y as u16)?;
        Ok(())
    }
}
