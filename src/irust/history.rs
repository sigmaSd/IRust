pub struct History {
    history: Vec<String>,
    current: String,
    cursor: usize,
}
impl Default for History {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            current: String::new(),
            cursor: 0,
        }
    }
}
impl History {
    pub fn down(&mut self) -> String {
        let filtered = self.filter();
        self.cursor += 1;
        if self.cursor >= filtered.len() {
            self.cursor = filtered.len();
            self.current.clone()
        } else {
            filtered[self.cursor].clone()
        }
    }

    pub fn up(&mut self) -> String {
        let filtered = self.filter();
        self.cursor = std::cmp::min(self.cursor, filtered.len());
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        if filtered.is_empty() {
            self.current.clone()
        } else {
            filtered[self.cursor].clone()
        }
    }

    pub fn push(&mut self, buffer: String) {
        if !buffer.is_empty() && Some(&buffer) != self.history.last() {
            self.history.push(buffer);
            self.go_to_last();
        }
    }

    pub fn update_current(&mut self, buffer: &str) {
        self.current = buffer.to_string();
        self.cursor = self.history.len();
    }

    fn filter(&self) -> Vec<String> {
        self.history
            .iter()
            .filter(|h| h.contains(&self.current))
            .map(ToOwned::to_owned)
            .collect()
    }

    fn go_to_last(&mut self) {
        if !self.history.is_empty() {
            self.cursor = self.history.len();
        }
    }

    fn _reset(&mut self) {
        *self = Self::default();
    }
}
