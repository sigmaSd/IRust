use crate::irust::IRust;
use crate::utils::StringTools;

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    origin: (usize, usize),
}
impl Cursor {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            origin: (x, y),
        }
    }

    pub fn move_right(&mut self, step: usize) {
        self.x += step;
    }

    pub fn move_left(&mut self, step: usize) {
        if self.x != 0 {
            self.x -= step
        }
    }

    pub fn reset(&mut self) {
        *self = Self {
            x: self.origin.0,
            y: self.origin.1,
            origin: self.origin,
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

    pub fn at_line_start(&self) -> bool {
        (self.internal_cursor.x + self.size.0) % self.size.0 == 1
    }

    pub fn at_line_end(&self) -> bool {
        (self.internal_cursor.x + self.size.0) % self.size.0 == 0
    }

}
