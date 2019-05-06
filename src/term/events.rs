use crate::term::Term;

impl Term {
    pub fn handle_enter(&mut self) -> std::io::Result<()> {
        self.write_newline()?;
        self.reset_cursors()?;
        self.parse()?;
        self.write_out()?;
        self.history.push(self.buffer.drain(..).collect());
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

    pub fn handle_backspace(&mut self) -> std::io::Result<()> {
        self.cursor.move_left(1);
        self.internal_cursor.left();
        if !self.buffer.is_empty() {
            self.buffer.remove(self.internal_cursor.col);
        }
        self.backspace()?;
        Ok(())
    }
}
