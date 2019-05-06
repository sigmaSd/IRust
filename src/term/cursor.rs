pub struct Cursor {
    pub x: usize,
}
impl Cursor {
    pub fn new(x: usize) -> Self {
        Self { x }
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
