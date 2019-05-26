use crate::irust::IRust;
use crate::utils::StringTools;

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    x_offset: usize,
    origin: (usize, usize),
    pub current_wrapped_lines: usize,
    pub total_wrapped_lines: usize,
}
impl Cursor {
    pub fn new(x: usize, y: usize, x_offset: usize) -> Self {
        Self {
            x,
            y,
            x_offset,
            origin: (x, y),
            current_wrapped_lines: 0,
            total_wrapped_lines: 0,
        }
    }

    pub fn move_right(&mut self) {
        self.x += 1;
    }

    pub fn move_left(&mut self) {
        if self.x != 0 {
            self.x -= 1
        }
    }

    pub fn get_corrected_x(&self) -> usize {
        if self.x >= self.x_offset {
            self.x - self.x_offset
        } else {
            0
        }
    }

    pub fn get_corrected_y(&self) -> usize {
        self.y + self.current_wrapped_lines
    }

    pub fn reset(&mut self) {
        *self = Self {
            x: self.origin.0,
            y: self.origin.1,
            x_offset: self.x_offset,
            origin: self.origin,
            current_wrapped_lines: 0,
            total_wrapped_lines: 0,
        };
    }

    pub fn reset_wrapped_lines(&mut self) {
        self.current_wrapped_lines = 0;
        self.total_wrapped_lines = 0;
    }
}

impl IRust {
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

    pub fn go_to_cursor(&mut self) -> std::io::Result<()> {
        self.cursor.goto(
            self.internal_cursor.x as u16,
            self.internal_cursor.get_corrected_y() as u16,
        )?;
        Ok(())
    }

    pub fn at_screen_start(&self) -> bool {
        (self.internal_cursor.x + self.size.0) % self.size.0 == 0
    }

    pub fn at_screen_end(&self) -> bool {
        !self.buffer.is_empty() && (self.internal_cursor.x + self.size.0) % self.size.0 == 1
    }

    pub fn at_line_end(&self) -> bool {
        if self.internal_cursor.get_corrected_x() == 0 && !self.buffer.is_empty() {
            false
        } else {
            let chars_count = StringTools::chars_count(&self.buffer);

            (self.internal_cursor.get_corrected_x() + chars_count)
                .checked_rem(chars_count)
                .unwrap_or(0)
                == 0
        }
    }

    pub fn move_internal_cursor_left(&mut self) -> std::io::Result<()> {
        self.internal_cursor.move_left();
        if self.at_screen_start() {
            self.internal_cursor.current_wrapped_lines = self
                .internal_cursor
                .current_wrapped_lines
                .checked_sub(1)
                .unwrap_or(0);

            self.move_cursor_to(self.size.0, self.internal_cursor.get_corrected_y())?;
        }

        Ok(())
    }

    pub fn move_internal_cursor_right(&mut self) -> std::io::Result<()> {
        self.internal_cursor.move_right();
        if self.at_screen_end() {
            self.internal_cursor.current_wrapped_lines += 1;
            self.internal_cursor.total_wrapped_lines = std::cmp::max(
                self.internal_cursor.total_wrapped_lines,
                self.internal_cursor.current_wrapped_lines,
            );
            self.move_cursor_to(0, self.internal_cursor.get_corrected_y())?;
        }
        if self.internal_cursor.get_corrected_y() >= self.size.1 {
            self.clear()?;
        }

        Ok(())
    }

    pub fn will_overflow_screen_height(&self, out: &str) -> bool {
        let new_lines_count = (out.len() + (self.internal_cursor.x % self.size.0)) / self.size.0;
        new_lines_count + self.internal_cursor.get_corrected_y() >= self.size.1
    }
}
