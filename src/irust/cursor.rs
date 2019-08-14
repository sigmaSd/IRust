use crate::irust::{IRust, IRustError};
mod bound;
use bound::Bound;

use crossterm::TerminalCursor;

#[derive(Clone)]
pub struct CursorPosition {
    pub screen_pos: (usize, usize),
    pub starting_pos: (usize, usize),
}

pub struct Cursor {
    pub pos: CursorPosition,
    pub internal_cursor: TerminalCursor,
    copy: Option<Box<CursorPosition>>,
    pub bound: Bound,
}

impl Cursor {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            pos: CursorPosition {
                screen_pos: (x, y),
                starting_pos: (0, y),
            },
            internal_cursor: TerminalCursor::new(),
            copy: None,
            bound: Bound::new(width, height),
        }
    }

    pub fn save_position(&mut self) -> Result<(), IRustError> {
        self.copy = Some(Box::new(self.pos.clone()));
        self.internal_cursor.save_position()?;
        Ok(())
    }

    pub fn move_screen_cursor_right(&mut self) {
        if self.pos.screen_pos.0 == self.current_row_bound() {
            self.pos.screen_pos.0 = 4;
            self.pos.screen_pos.1 += 1;
        } else {
            self.pos.screen_pos.0 += 1;
        }
        let _ = self.goto_internal_pos();
    }

    pub fn move_screen_cursor_left(&mut self) {
        if self.pos.screen_pos.0 == 4 {
            self.pos.screen_pos.0 = self.previous_row_bound();
            self.pos.screen_pos.1 -= 1;
        } else {
            self.pos.screen_pos.0 -= 1;
        }
        let _ = self.goto_internal_pos();
    }

    pub fn previous_row_bound(&self) -> usize {
        *self.bound.get_bound(self.pos.screen_pos.1 - 1)
    }

    pub fn current_row_bound(&self) -> usize {
        *self.bound.get_bound(self.pos.screen_pos.1)
    }

    pub fn bound_current_row_at_current_col(&mut self) {
        self.bound
            .set_bound(self.pos.screen_pos.1, self.pos.screen_pos.0);
    }

    pub fn is_at_line_end(&self, irust: &IRust) -> bool {
        irust.buf.is_at_end()
            || self.pos.screen_pos.0 == *self.bound.get_bound(self.pos.screen_pos.1)
    }

    pub fn screen_height_overflow_by_new_lines(&self, irust: &IRust, new_lines: usize) -> usize {
        // if corrected y  + new lines < self.size.1 there is no overflow so unwrap to 0
        (new_lines + self.pos.screen_pos.1).saturating_sub(irust.size.1 - 1)
    }

    pub fn reset_position(&mut self) -> Result<(), IRustError> {
        if let Some(copy) = self.copy.take() {
            self.pos = *copy;
            self.copy = Some(Box::new(self.pos.clone()));
        }
        self.internal_cursor.reset_position()?;
        Ok(())
    }

    pub fn goto_internal_pos(&mut self) -> Result<(), IRustError> {
        self.internal_cursor
            .goto(self.pos.screen_pos.0 as u16, self.pos.screen_pos.1 as u16)?;
        Ok(())
    }

    pub fn goto(&mut self, x: usize, y: usize) {
        self.pos.screen_pos.0 = x;
        self.pos.screen_pos.1 = y;

        let _ = self.goto_internal_pos();
    }

    pub fn hide(&self) {
        self.internal_cursor.hide().unwrap();
    }

    pub fn show(&self) {
        self.internal_cursor.show().unwrap();
    }

    pub fn move_up(&mut self, count: u16) {
        self.internal_cursor.move_up(count);
        self.pos.screen_pos.1 -= count as usize;
    }

    pub fn move_down(&mut self, count: u16) {
        self.internal_cursor.move_down(count);
        self.pos.screen_pos.1 += count as usize;
    }

    pub fn goto_start(&mut self) {
        self.pos.screen_pos.0 = self.pos.starting_pos.0;
        self.pos.screen_pos.1 = self.pos.starting_pos.1;
        let _ = self.goto_internal_pos();
    }

    pub fn is_at_last_col(&self) -> bool {
        self.pos.screen_pos.0 as usize == self.bound.width - 1
    }
}
