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
                });
            }
        }
        Ok(())
    }

    pub fn write_str_at(&mut self, s: &str, x: usize, y: usize) -> Result<(), IRustError> {
        self.cursor.goto(x, y);
        self.terminal.write(s)?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> Result<(), IRustError> {
        self.move_screen_cursor_to_last_line();
        self.cursor.pos.screen_pos.1 += 1;
        self.cursor.pos.starting_pos.1 = self.cursor.pos.screen_pos.1;
        let _ = self.cursor.goto_internal_pos();

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), IRustError> {
        self.terminal.clear(ClearType::All)?;
        self.buf.goto_start();
        self.cursor.pos.starting_pos = (0, 0);
        self.cursor.goto(4, 0);
        self.print()?;
        Ok(())
    }

    pub fn scroll_up(&mut self, n: usize) {
        let _ = self.terminal.scroll_up(n as i16);
        self.cursor.pos.starting_pos.1 = self.cursor.pos.starting_pos.1.saturating_sub(n);
        self.cursor.pos.screen_pos.1 = self.cursor.pos.screen_pos.1.saturating_sub(n);
        let _ = self.cursor.goto_internal_pos();
    }
}
