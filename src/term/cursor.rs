pub struct Cursor {
    pub x: usize,
    pub y: usize,
}
impl Cursor {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    pub fn right(&mut self) {
        self.x += 1;
    }

    pub fn left(&mut self) {
        if self.x != 0 {
            self.x -= 1
        }
    }
}
