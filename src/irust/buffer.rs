#[derive(Clone, Default)]
pub struct Buffer {
    pub buffer: Vec<char>,
    pub buffer_pos: usize,
    max_line_char: usize,
}

impl Buffer {
    pub fn new(max_line_char: usize) -> Self {
        Self {
            max_line_char,
            ..Self::default()
        }
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.buffer_pos, c);
        self.move_forward();
    }

    pub fn insert_str(&mut self, s: &str) {
        s.chars().for_each(|c| self.insert(c));
    }

    pub fn remove_current_char(&mut self) -> Option<char> {
        if !self.is_empty() {
            let character = self.buffer.remove(self.buffer_pos);
            Some(character)
        } else {
            None
        }
    }

    pub fn next_char(&self) -> Option<&char> {
        self.buffer.get(self.buffer_pos + 1)
    }

    pub fn current_char(&self) -> Option<&char> {
        self.buffer.get(self.buffer_pos)
    }

    pub fn previous_char(&self) -> Option<&char> {
        if self.buffer_pos > 0 {
            self.buffer.get(self.buffer_pos - 1)
        } else {
            None
        }
    }

    pub fn move_forward(&mut self) {
        self.buffer_pos += 1;
    }

    pub fn move_backward(&mut self) {
        if self.buffer_pos != 0 {
            self.buffer_pos -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.buffer_pos = 0;
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_at_start(&self) -> bool {
        self.buffer_pos == 0
    }

    pub fn is_at_end(&self) -> bool {
        self.buffer_pos == self.buffer.len()
    }

    pub fn goto_start(&mut self) {
        self.buffer_pos = 0;
    }

    pub fn goto_end(&mut self) {
        self.buffer_pos = self.buffer.len();
    }

    pub fn _push_str(&mut self, str: &str) {
        self.buffer.extend(str.chars());
        self.buffer_pos = self.buffer.len();
    }

    pub fn to_relative_current_pos(&self) -> (usize, usize) {
        let mut y = self
            .buffer
            .iter()
            .take(self.buffer_pos)
            .filter(|c| **c == '\n')
            .count();

        let mut x = 0;
        for i in 0..self.buffer_pos {
            match self.buffer.get(i) {
                Some('\n') => x = 0,
                _ => x += 1,
            };
            if x == self.max_line_char {
                x = 0;
                y += 1;
            }
        }

        (x, y)
    }

    pub fn end_to_relative_current_pos(&mut self) -> (usize, usize) {
        let tmp = self.buffer_pos;
        self.goto_end();
        let relative_pos = self.to_relative_current_pos();
        self.buffer_pos = tmp;

        relative_pos
    }

    pub fn from_str(str: &str, max_line_char: usize) -> Self {
        Self {
            buffer: str.chars().collect(),
            buffer_pos: 0,
            max_line_char,
        }
    }

    pub fn _get(&self, idx: usize) -> Option<&char> {
        self.buffer.get(idx)
    }

    pub fn _last(&self) -> Option<&char> {
        self.buffer.last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &char> {
        self.buffer.iter()
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.buffer.iter().collect()
    }
}
