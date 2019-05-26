use crossterm::ClearType;

use crate::irust::IRust;
use crate::utils::StringTools;

impl IRust {
    pub fn write(&mut self, out: &str) -> std::io::Result<()> {
        if !out.is_empty() {
            if StringTools::is_multiline(&out) {
                let _ = self.write_newline();
                out.split('\n').for_each(|o| {
                    let _ = self.terminal.write(o);
                    let _ = self.write_newline();
                });
            } else {
                out.chars().for_each(|c| {
                    let _ = self.move_internal_cursor_right();
                    let _ = self.terminal.write(c);
                });
            }
        }
        Ok(())
    }

    pub fn _writeln(&mut self, s: &str) -> std::io::Result<()> {
        self.write_newline()?;
        self.write(s)?;
        Ok(())
    }

    pub fn write_str_at<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        s: &str,
        x: P,
        y: U,
    ) -> std::io::Result<()> {
        self.move_cursor_to(x, y)?;
        self.write(s)?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.internal_cursor.y += 1;

        self.go_to_cursor()?;
        Ok(())
    }

    pub fn clear_suggestion(&mut self) -> std::io::Result<()> {
        if self.at_line_end() {
            self.clear_from(self.internal_cursor.x, self.internal_cursor.get_y())?;
        }

        Ok(())
    }

    fn clear_from<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        x: P,
        y: U,
    ) -> std::io::Result<()> {
        self.cursor.save_position()?;
        self.move_cursor_to(x, y)?;
        self.terminal.clear(ClearType::FromCursorDown)?;
        self.cursor.reset_position()?;

        Ok(())
    }
}
