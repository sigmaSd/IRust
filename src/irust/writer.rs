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
                    if self.internal_cursor.y + self.internal_cursor.total_wrapped_lines
                        > self.size.1
                    {
                        let _ = self.terminal.scroll_up(1);
                        self.internal_cursor.y -= 1;
                        self.cursor.move_up(1);
                    }
                    let _ = self.terminal.write(c);
                    let _ = self.move_cursor_right();
                });
            }
        }
        Ok(())
    }

    pub fn _writeln(&mut self, s: &str) -> Result<(), IRustError> {
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
        self.move_cursor_to(x, y)?;
        self.write(s)?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> Result<(), IRustError> {
        self.terminal.write('\n')?;
        self.internal_cursor.x = 0;
        self.internal_cursor.y += 1;
        // y should never exceed screen height
        if self.internal_cursor.y > self.size.1 {
            self.internal_cursor.y = self.size.1;
        }
        self.go_to_cursor()?;
        Ok(())
    }

    pub fn clear_suggestion(&mut self) -> Result<(), IRustError> {
        if self.at_line_end() {
            self.clear_from(
                self.internal_cursor.x,
                self.internal_cursor.get_corrected_y(),
            )?;
        }

        Ok(())
    }

    fn clear_from<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        x: P,
        y: U,
    ) -> Result<(), IRustError> {
        self.cursor.save_position()?;
        self.move_cursor_to(x, y)?;
        self.terminal.clear(ClearType::FromCursorDown)?;
        self.cursor.reset_position()?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), IRustError> {
        self.terminal.clear(ClearType::All)?;
        self.internal_cursor.reset();
        self.internal_cursor.y = 0;
        self.go_to_cursor()?;

        self.write_in()?;
        self.write(&self.buffer.clone())?;

        Ok(())
    }
}
