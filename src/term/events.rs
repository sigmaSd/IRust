use crate::term::Term;

impl Term {
    pub fn handle_character(&mut self, c: char) -> std::io::Result<()> {
        if c == '\n' {
            self.handle_enter()?
        } else {
            self.buffer.insert(self.internal_cursor.x, c);
            self.write_insert(c)?;
        }
        Ok(())
    }

    fn handle_enter(&mut self) -> std::io::Result<()> {
        // create a new line
        self.write_newline()?;

        // ignore all parsing errors
        let _ = self.parse();

        // ensure buffer is cleaned
        self.buffer.clear();

        // write out
        self.write_out()?;
        self.write_newline()?;
        self.write_in()?;

        Ok(())
    }

    pub fn handle_up(&mut self) -> std::io::Result<()> {
        self.reset_cursors()?;
        self.write_in()?;
        let up = self.history.up();
        self.buffer = up.clone();
        self.write_str(&up)?;
        Ok(())
    }

    pub fn handle_down(&mut self) -> std::io::Result<()> {
        self.reset_cursors()?;
        self.write_in()?;
        let down = self.history.down();
        self.buffer = down.clone();
        self.write_str(&down)?;
        Ok(())
    }

    pub fn handle_left(&mut self) -> std::io::Result<()> {
        if self.cursor.pos().0 as usize > 4 {
            self.cursor.move_left(1);
            self.internal_cursor.left();
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> std::io::Result<()> {
        if self.cursor.pos().0 as usize <= self.buffer.len() + 3 {
            self.cursor.move_right(1);
            self.internal_cursor.right();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x > 0 {
            self.cursor.move_left(1);
            self.internal_cursor.left();
            if !self.buffer.is_empty() {
                self.buffer.remove(self.internal_cursor.x);
            }
            self.backspace()?;
        }
        Ok(())
    }
}
