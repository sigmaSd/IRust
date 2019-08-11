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

#[derive(Clone)]
pub struct CursorPosition {
    pub screen_pos: (usize, usize),
    pub buffer_pos: usize,
    pub bounds: Bounds,
    pub starting_pos: (usize, usize),
}

pub struct Cursor {
    pub pos: CursorPosition,
    internal_cursor: TerminalCursor,
    origin: (usize, usize, usize),
    copy: Option<Box<CursorPosition>>,
}

impl Cursor {
    pub fn new(x: usize, y: usize, width: usize) -> Self {
        Self {
            pos: CursorPosition {
                screen_pos: (x, y),
                buffer_pos: 0,
                bounds: Bounds::new(y, (4, width)),
                starting_pos: (4, y),
            },
            internal_cursor: TerminalCursor::new(),
            origin: (x, y, width),
            copy: None,
        }
    }

    pub fn save_position(&mut self) -> Result<(), IRustError> {
        self.copy = Some(Box::new(self.pos.clone()));
        self.internal_cursor.save_position()?;
        Ok(())
    }

    pub fn move_screen_cursor_right(&mut self) {
        if self.pos.screen_pos.0 == self.current_upper_bound() {
            if self.pos.bounds.contains(self.pos.screen_pos.1 + 1) {
                self.pos.screen_pos.0 = self.pos.bounds.lower_bound(self.pos.screen_pos.1 + 1);
                self.pos.screen_pos.1 += 1;
            } else {
                self.pos.screen_pos.0 = 0;
                self.pos.screen_pos.1 += 1;
                self.add_bounds();
            }
        } else {
            self.pos.screen_pos.0 += 1;
        }

        self.goto_internal_pos().unwrap();
    }

    pub fn move_screen_cursor_left(&mut self, move_type: Move) {
        if self.pos.screen_pos == self.pos.starting_pos {
            return;
        }
        if self.pos.screen_pos.0 == self.current_lower_bound() {
            if self.pos.bounds.contains(self.pos.screen_pos.1 - 1) {
                self.pos.screen_pos.0 = self.pos.bounds.upper_bound(self.pos.screen_pos.1 - 1);
                self.pos.screen_pos.1 -= 1;

                if move_type == Move::Modify {
                    self.current_bounds_mut().1 = self.origin.2;
                }
            }
        } else {
            self.pos.screen_pos.0 -= 1;
        }

        self.goto_internal_pos().unwrap();
    }

    pub fn move_buffer_cursor_left(&mut self) {
        if self.pos.buffer_pos > 0 {
            self.pos.buffer_pos -= 1;
        }
    }

    pub fn move_buffer_cursor_right(&mut self) {
        self.pos.buffer_pos += 1;
    }

    pub fn reset(&mut self) {
        self.pos = CursorPosition {
            screen_pos: (self.origin.0, self.origin.1),
            buffer_pos: 0,
            starting_pos: (4, self.origin.1),
            bounds: Bounds::new(self.origin.1, (4, self.origin.2)),
        };

        self.copy = None;
    }

    fn current_lower_bound(&self) -> usize {
        self.pos.bounds.lower_bound(self.pos.screen_pos.1)
    }

    fn current_upper_bound(&self) -> usize {
        self.pos.bounds.upper_bound(self.pos.screen_pos.1)
    }

    pub fn current_bounds_mut(&mut self) -> &mut (usize, usize) {
        self.pos.bounds.get_mut(self.pos.screen_pos.1).unwrap()
    }

    pub fn add_bounds(&mut self) {
        self.pos.bounds
            .insert(self.pos.screen_pos.1, (self.pos.screen_pos.0, self.origin.2));
    }

    pub fn reset_screen_cursor(&mut self) {
        self.pos.screen_pos = self.pos.starting_pos;
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
            self.pos.screen_pos.0
        };

        let y = if y.is_some() {
            y.unwrap()
        } else {
            self.pos.screen_pos.1
        };

        self.internal_cursor.goto(x as u16, y as u16)?;

        Ok(())
    }

    pub fn is_at_line_end(&self, irust: &IRust) -> bool {
        irust.buffer.is_empty()
            || self.pos.buffer_pos == StringTools::chars_count(&irust.buffer)
    }

    pub fn screen_height_overflow_by_str(&self, irust: &IRust, out: &str) -> usize {
        let new_lines =
            (StringTools::chars_count(out) + self.pos.screen_pos.0) / irust.size.0;

        self.screen_height_overflow_by_new_lines(irust, new_lines)
    }

    pub fn screen_height_overflow_by_new_lines(&self, irust: &IRust, new_lines: usize) -> usize {
        // if corrected y  + new lines < self.size.1 there is no overflow so unwrap to 0
        (new_lines + self.pos.screen_pos.1).saturating_sub(irust.size.1)
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
        self.internal_cursor.goto(
            self.pos.screen_pos.0 as u16,
            self.pos.screen_pos.1 as u16,
        )?;
        Ok(())
    }

    pub fn move_right(&mut self, count: u16) -> Result<(), IRustError> {
        self.internal_cursor.move_right(count);
        self.goto_internal_pos()?;

        Ok(())
    }

    pub fn move_left(&mut self, count: u16) -> Result<(), IRustError> {
        self.internal_cursor.move_left(count);
        self.goto_internal_pos()
    }

    pub fn hide(&self) {
        self.internal_cursor.hide().unwrap();
    }

    pub fn show(&self) {
        self.internal_cursor.show().unwrap();
    }

    pub fn move_up(&mut self, count: u16) {
        self.internal_cursor.move_up(count);
    }

    pub fn move_down(&mut self, count: u16) {
        self.internal_cursor.move_down(count);
    }
}
