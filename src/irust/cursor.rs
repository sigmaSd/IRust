use super::Buffer;
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
mod bound;
use bound::Bound;

use super::raw_terminal::RawCursor;

/// input is shown with x in this example
/// |In: x
/// |    x
/// |    x
pub const INPUT_START_COL: usize = 4;

#[derive(Clone)]
pub struct CursorPosition {
    pub current_pos: (usize, usize),
    pub starting_pos: (usize, usize),
}

pub struct Cursor {
    pub pos: CursorPosition,
    pub cursor: RawCursor,
    copy: Option<Box<CursorPosition>>,
    pub bound: Bound,
}

impl Cursor {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            pos: CursorPosition {
                current_pos: (x, y),
                starting_pos: (0, y),
            },
            cursor: RawCursor::new(),
            copy: None,
            bound: Bound::new(width, height),
        }
    }

    pub fn save_position(&mut self) -> Result<(), IRustError> {
        self.copy = Some(Box::new(self.pos.clone()));
        self.cursor.save_position()?;
        Ok(())
    }

    pub fn move_right_unbounded(&mut self) {
        self.move_right_inner(self.bound.width - 1);
    }

    pub fn move_right(&mut self) {
        self.move_right_inner(self.current_row_bound());
    }

    fn move_right_inner(&mut self, bound: usize) {
        if self.pos.current_pos.0 == bound {
            self.pos.current_pos.0 = 4;
            self.pos.current_pos.1 += 1;
        } else {
            self.pos.current_pos.0 += 1;
        }
        let _ = self.goto_internal_pos();
    }

    pub fn move_left(&mut self) {
        if self.pos.current_pos.0 == 4 {
            self.pos.current_pos.0 = self.previous_row_bound();
            self.pos.current_pos.1 -= 1;
        } else {
            self.pos.current_pos.0 -= 1;
        }
        let _ = self.goto_internal_pos();
    }

    pub fn move_up_bounded(&mut self, count: u16) {
        self.move_up(count);
        self.pos.current_pos.0 = std::cmp::min(self.pos.current_pos.0, self.current_row_bound());
        let _ = self.goto_internal_pos();
    }

    pub fn move_up(&mut self, count: u16) {
        self.pos.current_pos.1 = self.pos.current_pos.1.saturating_sub(count as usize);
        let _ = self.cursor.move_up(count);
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
        let _ = self.goto_internal_pos();
    }

    pub fn move_down(&mut self, count: u16) {
        self.pos.current_pos.1 += count as usize;
        let _ = self.cursor.move_down(count);
    }

    pub fn use_current_row_as_starting_row(&mut self) {
        self.pos.starting_pos.1 = self.pos.current_pos.1;
    }

    pub fn previous_row_bound(&self) -> usize {
        *self.bound.get_bound(self.pos.current_pos.1 - 1)
    }

    pub fn current_row_bound(&self) -> usize {
        //crate::log!("cp: {}", self.pos.current_pos.1);
        *self.bound.get_bound(self.pos.current_pos.1)
    }

    pub fn bound_current_row_at_current_col(&mut self) {
        self.bound
            .set_bound(self.pos.current_pos.1, self.pos.current_pos.0);
    }

    pub fn is_at_line_end(&self, irust: &IRust) -> bool {
        irust.buffer.is_at_end()
            || self.pos.current_pos.0 == *self.bound.get_bound(self.pos.current_pos.1)
    }

    pub fn screen_height_overflow_by_str(&self, out: &str) -> usize {
        let new_lines =
            (StringTools::chars_count(out) + self.pos.current_pos.0) / (self.bound.width - 1);

        self.screen_height_overflow_by_new_lines(new_lines)
    }

    pub fn screen_height_overflow_by_new_lines(&self, new_lines: usize) -> usize {
        // if current row  + new lines < self.cursor.bound.height there is no overflow so unwrap to 0
        (new_lines + self.pos.current_pos.1).saturating_sub(self.bound.height - 1)
    }

    pub fn restore_position(&mut self) -> Result<(), IRustError> {
        if let Some(copy) = self.copy.take() {
            self.pos = *copy;
            self.copy = Some(Box::new(self.pos.clone()));
        }
        self.cursor.restore_position()?;
        Ok(())
    }

    pub fn goto_internal_pos(&mut self) -> Result<(), IRustError> {
        self.cursor
            .goto(self.pos.current_pos.0 as u16, self.pos.current_pos.1 as u16)?;
        Ok(())
    }

    pub fn goto(&mut self, x: usize, y: usize) {
        self.pos.current_pos.0 = x;
        self.pos.current_pos.1 = y;

        let _ = self.goto_internal_pos();
    }

    pub fn hide(&self) {
        self.cursor.hide().unwrap();
    }

    pub fn show(&self) {
        self.cursor.show().unwrap();
    }

    pub fn goto_start(&mut self) {
        self.pos.current_pos.0 = self.pos.starting_pos.0;
        self.pos.current_pos.1 = self.pos.starting_pos.1;
        let _ = self.goto_internal_pos();
    }

    pub fn goto_input_start_col(&mut self) {
        self.pos.current_pos.0 = self.pos.starting_pos.0 + INPUT_START_COL;
        self.pos.current_pos.1 = self.pos.starting_pos.1;
        let _ = self.goto_internal_pos();
    }

    pub fn _is_at_last_terminal_col(&self) -> bool {
        self.pos.current_pos.0 == self.bound.width - 1
    }

    pub fn is_at_last_terminal_row(&self) -> bool {
        self.pos.current_pos.1 == self.bound.height - 1
    }

    pub fn is_at_col(&self, col: usize) -> bool {
        self.pos.current_pos.0 == col
    }

    pub fn input_last_pos(&self, buffer: &Buffer) -> (usize, usize) {
        let relative_pos = buffer.last_buffer_pos_to_relative_cursor_pos();
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
