use std::iter::FromIterator;

#[derive(Clone, Default)]
pub struct Buffer {
    pub buffer: Vec<char>,
    pub buffer_pos: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.buffer_pos, c);
        self.move_forward();
    }

    pub fn insert_str(&mut self, s: &str) {
        let chars: Vec<char> = s.chars().collect();
        self.buffer.extend(&chars);
        self.buffer_pos += chars.len();
    }

    pub fn set_buffer_pos(&mut self, pos: usize) {
        self.buffer_pos = pos;
    }

    pub fn remove_current_char(&mut self) -> Option<char> {
        if !self.is_empty() && self.buffer_pos < self.buffer.len() {
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

    pub fn is_at_string_line_start(&self) -> bool {
        self.is_empty()
            || self.buffer[..self.buffer_pos]
                .rsplitn(2, |d| d == &'\n')
                .next()
                .unwrap_or_default()
                .iter()
                .all(|c| c.is_whitespace())
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

    pub fn get(&self, idx: usize) -> Option<&char> {
        self.buffer.get(idx)
    }

    pub fn _last(&self) -> Option<&char> {
        self.buffer.last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &char> {
        self.buffer.iter()
    }

    pub fn take(&mut self) -> Vec<char> {
        let buffer = std::mem::take(&mut self.buffer);
        self.clear();
        self.goto_start();
        buffer
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.buffer.iter().collect()
    }
}

impl From<&str> for Buffer {
    fn from(string: &str) -> Self {
        Self {
            buffer: string.chars().collect(),
            buffer_pos: 0,
        }
    }
}
impl From<String> for Buffer {
    fn from(string: String) -> Self {
        Self {
            buffer: string.chars().collect(),
            buffer_pos: 0,
        }
    }
}
impl From<Vec<char>> for Buffer {
    fn from(buffer: Vec<char>) -> Self {
        Self {
            buffer,
            buffer_pos: 0,
        }
    }
}
impl FromIterator<char> for Buffer {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Buffer {
        let mut buffer = Buffer::new();
        for c in iter {
            buffer.buffer.push(c);
        }
        buffer
    }
}
