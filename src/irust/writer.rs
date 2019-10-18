use crossterm::{ClearType, Color};

use crate::irust::{IRust, IRustError};

impl IRust {
    pub fn write(&mut self, out: &str, color: Color) -> Result<(), IRustError> {
        self.raw_terminal.set_fg(color)?;
        for c in out.chars() {
            self.raw_terminal.write(c)?;
            self.cursor.move_right_unbounded();
        }
        self.raw_terminal.reset_color()?;
        Ok(())
    }

    pub fn write_at(&mut self, s: &str, x: usize, y: usize) -> Result<(), IRustError> {
        self.cursor.goto(x, y);
        self.raw_terminal.write(s)?;
        Ok(())
    }

    pub fn write_at_no_cursor(&mut self, s: &str, x: usize, y: usize) -> Result<(), IRustError> {
        let origin_pos = self.cursor.pos.current_pos;
        self.cursor.goto(x, y);
        self.raw_terminal.write(s)?;
        self.cursor.goto(origin_pos.0, origin_pos.1);
        Ok(())
    }

    pub fn write_from_terminal_start(&mut self, out: &str, color: Color) -> Result<(), IRustError> {
        self.cursor.goto(0, self.cursor.pos.current_pos.1);
        self.write(out, color)?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> Result<(), IRustError> {
        self.cursor.move_to_input_last_row(&self.buffer);

        // check for scroll
        if self.cursor.is_at_last_terminal_row() {
            self.scroll_up(1);
        }
        self.cursor.move_down(1);
        self.cursor.use_current_row_as_starting_row();

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), IRustError> {
        self.raw_terminal.clear(ClearType::All)?;
        self.buffer.goto_start();
        self.cursor.pos.starting_pos = (0, 0);
        self.cursor.goto(4, 0);
        self.cursor.bound.reset();
        self.print_input()?;
        Ok(())
    }

    pub fn clear_last_line(&mut self) -> Result<(), IRustError> {
        let origin_pos = self.cursor.pos.current_pos;
        self.cursor.goto(0, self.cursor.bound.height - 1);
        self.raw_terminal.clear(ClearType::CurrentLine)?;
        self.cursor.goto(origin_pos.0, origin_pos.1);
        Ok(())
    }

    pub fn scroll_up(&mut self, n: usize) {
        let _ = self.raw_terminal.scroll_up(n as u16);
        self.cursor.move_up(n as u16);
        self.cursor.pos.starting_pos.1 = self.cursor.pos.starting_pos.1.saturating_sub(n);
    }

    pub fn write_from_next_line(&mut self) -> Result<(), IRustError> {
        self.buffer.insert('\n');
        self.print_input()?;
        self.cursor.goto(4, self.cursor.pos.current_pos.1 + 1);
        Ok(())
    }
}
