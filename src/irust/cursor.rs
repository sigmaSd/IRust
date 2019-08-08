use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
mod bounds;
use bounds::Bounds;

use crossterm::TerminalCursor;

#[derive(PartialEq)]
pub enum Move {
    Free,
    Modify,
}

pub struct Cursor {
    pub screen_pos: (usize, usize),
    pub buffer_pos: usize,
    pub bounds: Bounds,
    pub lock_pos: (usize, usize),
    // TODO(smolck): This may not need to be a Mutex,
    // right now used as a workaround for being unable
    // to clone a `crossterm::TerminalCursor`.
    internal_cursor: TerminalCursor,
    origin: (usize, usize, usize),
    copy: Option<Box<Cursor>>,
}

impl Clone for Cursor {
    fn clone(&self) -> Self {
        let bounds = self.bounds.clone();
        let internal_cursor = TerminalCursor::now();

        Self {
            screen_pos: self.screen_pos,
            buffer_pos: self.buffer_pos,
            bounds,
            lock_pos: self.lock_pos,
            internal_cursor,
            origin: self.origin,
            copy: self.copy.clone(),
        }
    }
}

impl Cursor {
    pub fn new(internal_cursor: TerminalCursor, x: usize, y: usize, width: usize) -> Self {
        Self {
            internal_cursor: internal_cursor,
            screen_pos: (x, y),
            buffer_pos: 0,
            bounds: Bounds::new(y, (4, width)),
            lock_pos: (4, y),
            origin: (x, y, width),
            copy: None,
        }
    }

    // NOTE(smolck): May need to modify this (`save_position`)
    // to return a Result<T, E>
    pub fn save_position(&mut self) {
        self.copy = Some(Box::new(self.clone()));
    }

    // NOTE(smolck): May need to modify this (`reset_position`)
    // to return a Result<T, E>
    pub fn reset_position(&mut self) {
        if let Some(copy) = self.copy.take() {
            *self = *copy;
            self.copy = Some(Box::new(self.clone()));
        }
    }

    pub fn move_right(&mut self) {
        if self.screen_pos.0 == self.current_upper_bound() {
            if self.bounds.contains(self.screen_pos.1 + 1) {
                self.screen_pos.0 = self.bounds.lower_bound(self.screen_pos.1 + 1);
                self.screen_pos.1 += 1;
            } else {
                self.screen_pos.0 = 0;
                self.screen_pos.1 += 1;
                self.add_bounds();
            }
        } else {
            self.screen_pos.0 += 1;
        }

        self.goto_cursor().unwrap();
    }

    // NOTE(smolck): Is `move_one_left` a good name for this?
    pub fn move_one_left(&mut self, move_type: Move) {
        if self.screen_pos == self.lock_pos {
            return;
        }
        if self.screen_pos.0 == self.current_lower_bound() {
            if self.bounds.contains(self.screen_pos.1 - 1) {
                self.screen_pos.0 = self.bounds.upper_bound(self.screen_pos.1 - 1);
                self.screen_pos.1 -= 1;

                if move_type == Move::Modify {
                    self.current_bounds_mut().1 = self.origin.2;
                }
            }
        } else {
            self.screen_pos.0 -= 1;
        }

        self.goto_cursor().unwrap();
    }

    pub fn move_buffer_cursor_left(&mut self) {
        if self.buffer_pos > 0 {
            self.buffer_pos -= 1;
        }
    }

    pub fn move_buffer_cursor_right(&mut self) {
        self.buffer_pos += 1;
    }

    pub fn reset(&mut self) {
        *self = Self {
            // This may need to change.
            internal_cursor: TerminalCursor::now(),
            screen_pos: (self.origin.0, self.origin.1),
            buffer_pos: 0,
            bounds: Bounds::new(self.origin.1, (4, self.origin.2)),
            lock_pos: (4, self.origin.1),
            origin: self.origin,
            copy: None,
        };
    }

    fn current_lower_bound(&self) -> usize {
        self.bounds.lower_bound(self.screen_pos.1)
    }

    fn current_upper_bound(&self) -> usize {
        self.bounds.upper_bound(self.screen_pos.1)
    }

    pub fn current_bounds_mut(&mut self) -> &mut (usize, usize) {
        self.bounds.get_mut(self.screen_pos.1).unwrap()
    }

    pub fn add_bounds(&mut self) {
        self.bounds
            .insert(self.screen_pos.1, (self.screen_pos.0, self.origin.2));
    }

    pub fn reset_screen_cursor(&mut self) {
        self.screen_pos = self.lock_pos;
    }

    pub fn move_cursor_to<P: Into<Option<usize>>, U: Into<Option<usize>>>(
        &mut self,
        x: P,
        y: U,
    ) -> Result<(), IRustError> {
        let x = x.into();
        let y = y.into();

        let x = if x.is_some() {
            x.unwrap()
        } else {
            self.screen_pos.0
        };

        let y = if y.is_some() {
            y.unwrap()
        } else {
            self.screen_pos.1
        };

        self.internal_cursor.goto(x as u16, y as u16)?;

        Ok(())
    }

    pub fn is_at_line_end(&self, irust: &IRust) -> bool {
        irust.buffer.is_empty()
            || self.buffer_pos == StringTools::chars_count(&irust.buffer)
    }

    pub fn screen_height_overflow_by_str(&self, irust: &IRust, out: &str) -> usize {
        let new_lines =
            (StringTools::chars_count(out) + self.screen_pos.0) / irust.size.0;

        self.screen_height_overflow_by_new_lines(irust, new_lines)
    }

    pub fn screen_height_overflow_by_new_lines(&self, irust: &IRust, new_lines: usize) -> usize {
        // if corrected y  + new lines < self.size.1 there is no overflow so unwrap to 0
        (new_lines + self.screen_pos.1).saturating_sub(irust.size.1)
    }

    pub fn save_cursor_position(&mut self) -> Result<(), IRustError> {
        self.internal_cursor.save_position()?;
        // internal_cursor.save_position();
        Ok(())
    }

    pub fn reset_cursor_position(&mut self) -> Result<(), IRustError> {
        self.reset_position();
        self.internal_cursor.reset_position();
        Ok(())
    }

    pub fn goto_cursor(&mut self) -> Result<(), IRustError> {
        self.internal_cursor.goto(
            self.screen_pos.0 as u16,
            self.screen_pos.1 as u16,
        )?;
        Ok(())
    }

    pub fn move_cursor_right(&mut self, idx: u16) -> Result<(), IRustError> {
        self.internal_cursor.move_right(idx);
        self.goto_cursor()?;

        Ok(())
    }

    // NOTE(smolck): Maybe change variable name to something other than `n`
    // Also, the following functions work on internal cursor exclusively,
    // so we may want to remove them and make `internal_cursor` public in the
    // `Cursor` struct.
    pub fn move_up(&mut self, n: u16) {
        self.internal_cursor.move_up(n);
    }

    pub fn hide(&self) {
        self.internal_cursor.hide();
    }

    pub fn show(&self) {
        self.internal_cursor.show();
    }

    pub fn move_down(&mut self, idx: u16) {
        self.internal_cursor.move_down(idx);
    }

    pub fn move_left(&mut self, idx: u16) {
        self.internal_cursor.move_down(idx);
    }
}
