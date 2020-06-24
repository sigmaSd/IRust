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
        s.chars().for_each(|c| self.insert(c));
    }

    pub fn set_buffer_pos(&mut self, pos: usize) {
        self.buffer_pos = pos;
    }

    pub fn set_buffer_pos_from_visible(&mut self, pos: usize, visible_i: usize) {
        self.buffer_pos = pos + visible_i;
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
            || self.previous_char() == Some(&'\n')
            || self.previous_char() == Some(&'\t')
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

    pub fn buffer_pos_to_relative_cursor_pos(
        &self,
        buffer_pos: usize,
        screen_width: usize,
    ) -> (usize, usize) {
        let max_line_chars = screen_width - super::cursor::INPUT_START_COL;

        let mut y = self
            .buffer
            .iter()
            .take(buffer_pos)
            .filter(|c| **c == '\n')
            .count();

        let mut x = 0;
        for i in 0..buffer_pos {
            match self.buffer.get(i) {
                Some('\n') => x = 0,
                _ => x += 1,
            };
            if x == max_line_chars {
                x = 0;
                y += 1;
            }
        }

        (x, y)
    }

    pub fn last_buffer_pos_to_relative_cursor_pos(&self, screen_width: usize) -> (usize, usize) {
        self.buffer_pos_to_relative_cursor_pos(self.buffer.len(), screen_width)
    }

    pub fn from_str(str: &str) -> Self {
        Self {
            buffer: str.chars().collect(),
            buffer_pos: 0,
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

    pub fn visible(&self, cursor_start_pos_y: isize, screen_width: usize) -> (String, usize) {
        if cursor_start_pos_y >= 0 {
            (self.to_string(),0)
        } else {
            crate::log!(cursor_start_pos_y);
            let max_line_chars = screen_width - super::cursor::INPUT_START_COL;
            let mut x = 0;
            let mut y = 0;

            for i in 0..self.len() {
                match self.buffer.get(i) {
                    Some('\n') => {
                        x = 0;
                        y += 1
                    }
                    _ => x += 1,
                };
                if x == max_line_chars {
                    x = 0;
                    y += 1;
                }
                if y == cursor_start_pos_y.abs() {
                    // +1 to skip last new line or  width
                    return (self.buffer.iter().skip(i + 1).collect(), i+1)
                }
            }
            (self.to_string(),0)
        }
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.buffer.iter().collect()
    }
}
