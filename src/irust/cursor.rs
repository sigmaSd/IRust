use crate::irust::IRust;

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    x_offset: usize,
    origin: (usize, usize),
    pub wrapped_lines: usize,
}
impl Cursor {
    pub fn new(x: usize, y: usize, x_offset: usize) -> Self {
        Self {
            x,
            y,
            x_offset,
            origin: (x, y),
            wrapped_lines: 0,
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

    pub fn get_x(&self) -> usize {
        if self.x >= self.x_offset {
            self.x - self.x_offset
        } else {
            0
        }
    }

    pub fn get_y(&self) -> usize {
        self.y + self.wrapped_lines
    }

    pub fn reset(&mut self) {
        *self = Self {
            x: self.origin.0,
            y: self.origin.1,
            x_offset: self.x_offset,
            origin: self.origin,
            wrapped_lines: 0,
        };
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
        self.cursor
            .goto(self.internal_cursor.x as u16, self.internal_cursor.y as u16)?;
        Ok(())
    }

    pub fn at_line_end(&self) -> bool {
        (self.internal_cursor.x + self.size.0) % self.size.0 == 0
    }

    pub fn move_internal_cursor_left(&mut self) -> std::io::Result<()> {
        self.internal_cursor.move_left();
        if self.at_line_end() {
            self.internal_cursor.wrapped_lines = self
                .internal_cursor
                .wrapped_lines
                .checked_sub(1)
                .unwrap_or(0);
            self.move_cursor_to(self.size.0, self.internal_cursor.get_y())?;
        }

        Ok(())
    }

    pub fn move_internal_cursor_right(&mut self) -> std::io::Result<()> {
        self.internal_cursor.move_right();
        if self.at_line_end() {
            self.internal_cursor.wrapped_lines += 1;
            self.move_cursor_to(0, self.internal_cursor.get_y())?;
        }
        if self.internal_cursor.get_y() >= self.size.1 {
            self.clear()?;
        }

        Ok(())
    }
}
