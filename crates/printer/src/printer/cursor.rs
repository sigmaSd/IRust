use std::{cell::RefCell, rc::Rc};
mod bound;
pub use bound::Bound;
mod raw;
use raw::Raw;

use crate::buffer::Buffer;
/// input is shown with x in this example
/// |In: x
/// |    x
/// |    x

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub current_pos: (usize, usize),
    pub starting_pos: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct Cursor<W: std::io::Write> {
    //pub for tests only
    #[cfg(test)]
    pub(super) pos: CursorPosition,
    #[cfg(test)]
    pub(super) bound: Bound,

    #[cfg(not(test))]
    pos: CursorPosition,
    #[cfg(not(test))]
    bound: Bound,

    pub prompt_len: usize,
    pub raw: Raw<W>,

    copy: CursorPosition,
}

impl<W: std::io::Write> Cursor<W> {
    pub fn new(raw: Rc<RefCell<W>>, prompt_len: usize) -> Self {
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
            prompt_len,
        }
    }

    pub fn width(&self) -> usize {
        self.bound.width
    }

    pub fn height(&self) -> usize {
        self.bound.height
    }

    pub fn current_pos(&self) -> (usize, usize) {
        self.pos.current_pos
    }

    pub fn set_current_pos(&mut self, xpos: usize, ypos: usize) {
        self.pos.current_pos = (xpos, ypos);
    }

    pub fn starting_pos(&self) -> (usize, usize) {
        self.pos.starting_pos
    }

    pub fn set_starting_pos(&mut self, xpos: usize, ypos: usize) {
        self.pos.starting_pos = (xpos, ypos);
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

    pub fn move_right_inner_optimized(&mut self) {
        // Performance: Make sure to not move the cursor if cursor_pos = last_cursor_pos+1 because it moves automatically
        if self.pos.current_pos.0 == self.bound.width - 1 {
            self.pos.current_pos.0 = self.prompt_len;
            self.pos.current_pos.1 += 1;
            self.goto_internal_pos();
        } else {
            self.pos.current_pos.0 += 1;
        }
    }
    fn move_right_inner(&mut self, bound: usize) {
        if self.pos.current_pos.0 == bound {
            self.pos.current_pos.0 = self.prompt_len;
            self.pos.current_pos.1 += 1;
        } else {
            self.pos.current_pos.0 += 1;
        }
        self.goto_internal_pos();
    }

    pub fn move_left(&mut self) {
        if self.pos.current_pos.0 == self.prompt_len {
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

    pub fn reset_bound(&mut self) {
        self.bound.reset();
    }

    pub fn bound_current_row_at_current_col(&mut self) {
        self.bound
            .set_bound(self.pos.current_pos.1, self.pos.current_pos.0);
    }

    /// Check if adding new_lines to the buffer will make it overflow the screen height and return
    /// that amount if so (0 if not)
    pub fn screen_height_overflow_by_new_lines(&self, buffer: &Buffer, new_lines: usize) -> usize {
        // if current row  + new lines < self.raw..bound.height there is no overflow so unwrap to 0
        (new_lines + self.input_last_pos(buffer).1).saturating_sub(self.bound.height - 1)
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
        self.pos.current_pos.0 = self.pos.starting_pos.0 + self.prompt_len;
        self.pos.current_pos.1 = self.pos.starting_pos.1;
        self.goto_internal_pos();
    }

    pub fn is_at_last_terminal_col(&self) -> bool {
        self.pos.current_pos.0 == self.bound.width - 1
    }

    pub fn is_at_last_terminal_row(&self) -> bool {
        self.pos.current_pos.1 == self.bound.height - 1
    }

    pub fn is_at_line_end(&self) -> bool {
        self.pos.current_pos.0 == self.current_row_bound()
    }

    pub fn is_at_line_start(&self) -> bool {
        self.pos.current_pos.0 == self.prompt_len
    }

    pub fn is_at_col(&self, col: usize) -> bool {
        self.pos.current_pos.0 == col
    }

    pub fn buffer_pos_to_cursor_pos(&self, buffer: &Buffer) -> (usize, usize) {
        let last_buffer_pos = buffer.len();
        let max_line_chars = self.bound.width - self.prompt_len;

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
        let x = relative_pos.0 + self.prompt_len;
        let y = relative_pos.1 + self.pos.starting_pos.1;

        (x, y)
    }

    pub fn move_to_input_last_row(&mut self, buffer: &Buffer) {
        let input_last_row = self.input_last_pos(buffer).1;
        self.goto(0, input_last_row);
    }
    pub fn goto_last_row(&mut self, buffer: &Buffer) {
        self.pos.current_pos.1 = self.input_last_pos(buffer).1;
        self.pos.current_pos.0 = std::cmp::min(self.pos.current_pos.0, self.current_row_bound());
        self.goto_internal_pos();
    }

    pub fn is_at_first_input_line(&self) -> bool {
        self.pos.current_pos.1 == self.pos.starting_pos.1
    }

    pub fn is_at_last_input_line(&self, buffer: &Buffer) -> bool {
        self.pos.current_pos.1 == self.input_last_pos(buffer).1
    }

    pub fn cursor_pos_to_buffer_pos(&self) -> usize {
        self.pos.current_pos.0 - self.prompt_len
            + self.bound.bounds_sum(
                self.pos.starting_pos.1,
                self.pos.current_pos.1,
                self.prompt_len,
            )
    }

    pub fn goto_next_row_terminal_start(&mut self) {
        self.goto(0, self.pos.current_pos.1 + 1);
    }

    pub fn update_dimensions(&mut self, width: u16, height: u16) {
        self.bound = Bound::new(width as usize, height as usize);
    }
}
