use crossterm::{ClearType, Color};

use crate::irust::{IRust, IN};
use crate::utils::StringTools;

impl IRust {
    pub fn _writeln(&mut self, s: &str) -> std::io::Result<()> {
        self.write_newline()?;
        self.write(s)?;
        Ok(())
    }

    pub fn write(&mut self, out: &str) -> std::io::Result<()> {
        if !out.is_empty() {
            self.go_to_cursor()?;

            if StringTools::is_multiline(&out) {
                let _ = self.write_newline();
                out.split('\n').for_each(|o| {
                    let _ = self.terminal.write(o);
                    let _ = self.write_newline();
                });
            } else {
                self.terminal.write(out)?;
                self.move_cursor_to(out.len(), None)?;
                self.internal_cursor
                    .move_right(StringTools::chars_count(&out));
            }
        }
        Ok(())
    }

    pub fn write_str_at<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        s: &str,
        x: P,
        y: U,
    ) -> std::io::Result<()> {
        self.move_cursor_to(x, y)?;
        self.write_str(s)?;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.terminal.write(s)?;
        self.internal_cursor.move_right(StringTools::chars_count(s));
        Ok(())
    }

    pub fn write_in(&mut self) -> std::io::Result<()> {
        self.go_to_cursor()?;
        self.color.set_fg(Color::Yellow)?;
        self.terminal.write(IN)?;
        self.color.reset()?;
        Ok(())
    }

    pub fn write_insert(&mut self, c: char) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;

        self.terminal.write(c)?;
        self.cursor.save_position()?;
        self.internal_cursor.move_right(1);

        for character in self.buffer.chars().skip(self.internal_cursor.x) {
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

        for character in self.buffer.chars().skip(self.internal_cursor.x) {
            self.terminal.write(character)?;
        }
        self.cursor.reset_position()?;
        Ok(())
    }
}
