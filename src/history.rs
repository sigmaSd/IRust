pub struct History {
    pub buffer_vec: Vec<String>,
    cursor: usize,
    empty: String,
}
impl Default for History {
    fn default() -> Self {
        Self {
            buffer_vec: Vec::new(),
            cursor: 0,
            empty: String::new(),
        }
    }
}
impl History {
    pub fn down(&mut self) -> String {
        self.cursor += 1;
        if self.cursor > self.buffer_vec.len() {
            self.cursor = self.buffer_vec.len();
        }
        self.buffer_vec
            .get(self.cursor)
            .unwrap_or(&self.empty)
            .clone()
    }
    pub fn up(&mut self) -> String {
        if self.cursor > 0 {
            self.cursor -= 1;
        }

        self.buffer_vec
            .get(self.cursor)
            .unwrap_or(&self.empty)
            .clone()
    }
    fn go_to_last(&mut self) {
        if !self.buffer_vec.is_empty() {
            self.cursor = self.buffer_vec.len();
        }
    }
    pub fn push(&mut self, buffer: String) {
        if !buffer.is_empty() {
            self.buffer_vec.push(buffer);
            self.go_to_last();
        }
    }
    pub fn _reset(&mut self) {
        *self = Self::default();
    }
}
