use crossterm::{ClearType, Color};

use crate::term::{Term, IN, OUT};

impl Term {
    pub fn write_str_at<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        s: &str,
        x: P,
        y: U,
    ) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(x, y)?;
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
        self.internal_cursor.move_right();

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

    pub fn move_cursor_to<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        x: P,
        y: U,
    ) -> std::io::Result<()> {
        let x = x.into();
        let y = y.into();

        let x = if x.is_some() {
            x.unwrap()
        } else {
            self.internal_cursor.x
        };

        let y = if y.is_some() {
            y.unwrap()
        } else {
            self.internal_cursor.y
        };

        self.cursor.goto(x as u16, y as u16)?;

        Ok(())
    }

    fn go_to_cursor(&mut self) -> std::io::Result<()> {
        self.cursor
            .goto(self.internal_cursor.x as u16, self.internal_cursor.y as u16)?;
        Ok(())
    }
}
