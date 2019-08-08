use crossterm::ClearType;

use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;

impl IRust {
    pub fn write(&mut self, out: &str) -> Result<(), IRustError> {
        if !out.is_empty() {
            if StringTools::is_multiline(&out) {
                let _ = self.write_newline();
                out.split('\n').for_each(|o| {
                    let _ = self.terminal.write(o);
                    let _ = self.write_newline();
                });
            } else {
                out.chars().for_each(|c| {
                    let _ = self.terminal.write(c);
                    let _ = self.cursor.move_right();
                });
            }
        }
        Ok(())
    }

    fn _writeln(&mut self, s: &str) -> Result<(), IRustError> {
        self.write_newline()?;
        self.write(s)?;
        Ok(())
    }

    pub fn write_str_at<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        s: &str,
        x: P,
        y: U,
    ) -> Result<(), IRustError> {
        self.cursor.move_cursor_to(x, y)?;
        self.terminal.write(s)?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> Result<(), IRustError> {
        self.cursor.screen_pos.0 = 0;
        self.cursor.screen_pos.1 += 1;
        self.cursor.add_bounds();
        // y should never exceed screen height
        if self.cursor.screen_pos.1 == self.size.1 {
            self.scroll_up(1);
        }

        self.cursor.goto_cursor()?;
        Ok(())
    }

    pub fn clear_suggestion(&mut self) -> Result<(), IRustError> {
        if self.cursor.is_at_line_end(&self) {
            self.clear_from(
                self.cursor.screen_pos.0,
                self.cursor.screen_pos.1,
            )?;
        }

        Ok(())
    }

    fn clear_from<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        x: P,
        y: U,
    ) -> Result<(), IRustError> {
        self.cursor.save_position();
        self.cursor.move_cursor_to(x, y)?;
        self.terminal.clear(ClearType::FromCursorDown)?;
        self.cursor.reset_position();

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), IRustError> {
        self.terminal.clear(ClearType::All)?;
        self.cursor.reset();
        self.cursor.screen_pos.1 = 0;
        self.cursor.goto_cursor()?;

        self.write_in()?;
        self.write(&self.buffer.clone())?;
        self.cursor.buffer_pos = StringTools::chars_count(&self.buffer);

        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_empty() {
            StringTools::remove_at_char_idx(&mut self.buffer, self.cursor.buffer_pos);
        }
        self.write_insert(None)?;
        Ok(())
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.terminal.scroll_up(n as i16).unwrap();
        self.cursor.move_up(n as u16);
        self.cursor.screen_pos.1 = self.cursor.screen_pos.1.saturating_sub(n);
        self.cursor.lock_pos.1 = self.cursor.lock_pos.1.saturating_sub(n);
        self.cursor.bounds.shift_keys_left(n);
    }
}
