use crate::irust::Buffer;
use crate::utils::StringTools;
use std::{cell::RefCell, rc::Rc};
mod bound;
pub use bound::Bound;
mod raw;
use raw::Raw;
/// input is shown with x in this example
/// |In: x
/// |    x
/// |    x
pub const INPUT_START_COL: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub current_pos: (usize, usize),
    pub starting_pos: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct Cursor<W: std::io::Write> {
    pub pos: CursorPosition,
    copy: CursorPosition,
    pub bound: Bound,
    pub raw: Raw<W>,
}

impl<W: std::io::Write> Cursor<W> {
    pub fn new(raw: Rc<RefCell<W>>) -> Self {
        let mut raw = Raw { raw };
        let (width, height) = raw.size().unwrap_or((400, 400));
        let current_pos = raw.get_current_pos().unwrap_or((0, 0));

        let pos = CursorPosition {
            current_pos,
            starting_pos: (0, current_pos.1),
        };
        Self {
            pos,
            copy: pos,
            bound: Bound::new(width as usize, height as usize),
            raw,
        }
    }

    pub fn save_position(&mut self) {
        self.copy = self.pos;
        self.raw
            .save_position()
            .expect("failed to save cursor position");
    }

    pub fn move_right_unbounded(&mut self) {
        self.move_right_inner(self.bound.width - 1);
    }

    pub fn move_right(&mut self) {
        self.move_right_inner(self.current_row_bound());
    }

    fn move_right_inner(&mut self, bound: usize) {
        if self.pos.current_pos.0 == bound {
            self.pos.current_pos.0 = INPUT_START_COL;
            self.pos.current_pos.1 += 1;
        } else {
            self.pos.current_pos.0 += 1;
        }
        self.goto_internal_pos();
    }

    pub fn move_left(&mut self) {
        if self.pos.current_pos.0 == INPUT_START_COL {
            self.pos.current_pos.0 = self.previous_row_bound();
            self.pos.current_pos.1 -= 1;
        } else {
            self.pos.current_pos.0 -= 1;
        }
        self.goto_internal_pos();
    }

    pub fn move_up_bounded(&mut self, count: u16) {
        self.move_up(count);
        self.pos.current_pos.0 = std::cmp::min(self.pos.current_pos.0, self.current_row_bound());
        self.goto_internal_pos();
    }

    pub fn move_up(&mut self, count: u16) {
        self.pos.current_pos.1 = self.pos.current_pos.1.saturating_sub(count as usize);
        self.raw.move_up(count).expect("failed to move cursor up");
    }

    pub fn move_down_bounded(&mut self, count: u16, buffer: &Buffer) {
        self.move_down(count);
        // check if we're out of bound
        self.pos.current_pos.0 = std::cmp::min(self.pos.current_pos.0, self.current_row_bound());
        // check if we passed the buffer
        let last_pos = self.input_last_pos(buffer);
        if self.pos.current_pos.1 >= last_pos.1 && self.pos.current_pos.0 >= last_pos.0 {
            self.pos.current_pos = last_pos;
        }
        self.goto_internal_pos();
    }

    pub fn move_down(&mut self, count: u16) {
        self.pos.current_pos.1 += count as usize;
        self.raw
            .move_down(count)
            .expect("failed to move cursor down");
    }

    pub fn use_current_row_as_starting_row(&mut self) {
        self.pos.starting_pos.1 = self.pos.current_pos.1;
    }

    pub fn previous_row_bound(&self) -> usize {
        self.bound.get_bound(self.pos.current_pos.1 - 1)
    }

    pub fn current_row_bound(&self) -> usize {
        self.bound.get_bound(self.pos.current_pos.1)
    }

    pub fn bound_current_row_at_current_col(&mut self) {
        self.bound
            .set_bound(self.pos.current_pos.1, self.pos.current_pos.0);
    }

    pub fn is_at_line_end(&self, buffer: &crate::irust::Buffer) -> bool {
        buffer.is_at_end() || self.pos.current_pos.0 == self.bound.get_bound(self.pos.current_pos.1)
    }

    pub fn screen_height_overflow_by_str(&self, out: &str) -> usize {
        let new_lines =
            (StringTools::chars_count(out) + self.pos.current_pos.0) / (self.bound.width - 1);

        self.screen_height_overflow_by_new_lines(new_lines)
    }

    pub fn screen_height_overflow_by_new_lines(&self, new_lines: usize) -> usize {
        // if current row  + new lines < self.raw..bound.height there is no overflow so unwrap to 0
        (new_lines + self.pos.current_pos.1).saturating_sub(self.bound.height - 1)
    }

    pub fn restore_position(&mut self) {
        self.pos = self.copy;
        self.raw
            .restore_position()
            .expect("failed to restore cursor position");
    }

    pub fn goto_internal_pos(&mut self) {
        self.raw
            .goto(self.pos.current_pos.0 as u16, self.pos.current_pos.1 as u16)
            .expect("failed to move cursor");
    }

    pub fn goto(&mut self, x: usize, y: usize) {
        self.pos.current_pos.0 = x;
        self.pos.current_pos.1 = y;

        self.goto_internal_pos();
    }

    pub fn hide(&mut self) {
        self.raw.hide().expect("failed to hide cursor");
    }

    pub fn show(&mut self) {
        self.raw.show().expect("failed to show cursor");
    }

    pub fn goto_start(&mut self) {
        self.pos.current_pos.0 = self.pos.starting_pos.0;
        self.pos.current_pos.1 = self.pos.starting_pos.1;
        self.goto_internal_pos();
    }

    pub fn goto_input_start_col(&mut self) {
        self.pos.current_pos.0 = self.pos.starting_pos.0 + INPUT_START_COL;
        self.pos.current_pos.1 = self.pos.starting_pos.1;
        self.goto_internal_pos();
    }

    pub fn is_at_last_terminal_col(&self) -> bool {
        self.pos.current_pos.0 == self.bound.width - 1
    }

    pub fn is_at_last_terminal_row(&self) -> bool {
        self.pos.current_pos.1 == self.bound.height - 1
    }

    pub fn is_at_col(&self, col: usize) -> bool {
        self.pos.current_pos.0 == col
    }

    pub fn buffer_pos_to_cursor_pos(&self, buffer: &Buffer) -> (usize, usize) {
        let last_buffer_pos = buffer.len();
        let max_line_chars = self.bound.width - INPUT_START_COL;

        let mut y = buffer
            .iter()
            .take(last_buffer_pos)
            .filter(|c| **c == '\n')
            .count();

        let mut x = 0;
        for i in 0..last_buffer_pos {
            match buffer.get(i) {
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

    pub fn input_last_pos(&self, buffer: &Buffer) -> (usize, usize) {
        let relative_pos = self.buffer_pos_to_cursor_pos(buffer);
        //let relative_pos = buffer.last_buffer_pos_to_relative_cursor_pos(self.bound.width);
        let x = relative_pos.0 + INPUT_START_COL;
        let y = relative_pos.1 + self.pos.starting_pos.1;

        (x, y)
    }

    pub fn move_to_input_last_row(&mut self, buffer: &Buffer) {
        let input_last_row = self.input_last_pos(buffer).1;
        self.goto(0, input_last_row);
    }

    pub fn is_at_first_input_line(&self) -> bool {
        self.pos.current_pos.1 == self.pos.starting_pos.1
    }

    pub fn is_at_last_input_line(&self, buffer: &Buffer) -> bool {
        self.pos.current_pos.1 == self.input_last_pos(buffer).1
    }

    pub fn cursor_pos_to_buffer_pos(&self) -> usize {
        self.pos.current_pos.0 - INPUT_START_COL
            + self
                .bound
                .bounds_sum(self.pos.starting_pos.1, self.pos.current_pos.1)
    }

    pub fn goto_next_row_terminal_start(&mut self) {
        self.goto(0, self.pos.current_pos.1 + 1);
    }
}
