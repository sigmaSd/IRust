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

    pub fn move_right(&mut self) {
        self.x += 1;
    }

    pub fn move_left(&mut self) {
        if self.x != 0 {
            self.x -= 1
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
