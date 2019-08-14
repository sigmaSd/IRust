#[derive(Clone, Default)]
pub struct Buffer {
    pub buffer: Vec<char>,
    pub buffer_pos: usize,
    max_line_character: usize,
}

impl Buffer {
    pub fn new(max_line_character: usize) -> Self {
        Self {
            max_line_character,
            ..Self::default()
        }
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.buffer_pos, c);
        self.move_forward();
    }

    pub fn remove_current_char(&mut self) -> Option<char> {
        if !self.is_empty() {
            let character = self.buffer.remove(self.buffer_pos);
            //self.move_backward();

            Some(character)
        } else {
            None
        }
    }

    pub fn current_char(&self) -> Option<&char> {
        self.buffer.get(self.buffer_pos.saturating_sub(1))
    }

    pub fn move_forward(&mut self) {
        self.buffer_pos += 1;
        // if self.buffer_pos == self.len() {
        //     self.buffer_pos = self.len().saturating_sub(1);
        // }
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

    pub fn push_str(&mut self, str: &str) {
        self.buffer.extend(str.chars());
        self.buffer_pos = self.buffer.len();
    }

    pub fn to_relative_screen_pos(&self) -> (usize, usize) {
        let x = self
            .buffer
            .iter()
            .take(self.buffer_pos)
            .filter(|c| **c != '\n')
            .count();

        let y = self
            .buffer
            .iter()
            .take(self.buffer_pos)
            .filter(|c| **c == '\n')
            .count();

        (x, y)
    }

    pub fn end_to_relative_screen_pos(&mut self) -> (usize, usize) {
        let tmp = self.buffer_pos;
        self.goto_end();
        let relative_pos = self.to_relative_screen_pos();
        self.buffer_pos = tmp;

        relative_pos
    }

    pub fn from_str(str: &str, max_line_character: usize) -> Self {
        Self {
            buffer: str.chars().collect(),
            buffer_pos: 0,
            max_line_character,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&char> {
        self.buffer.get(idx)
    }

    pub fn _last(&self) -> Option<&char> {
        self.buffer.last()
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.buffer.iter().collect()
    }
}
