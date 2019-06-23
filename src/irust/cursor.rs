use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
mod bounds;
use bounds::Bounds;

#[derive(PartialEq)]
pub enum Move {
    Free,
    Modify,
}

#[derive(Clone)]
pub struct Cursor {
    pub screen_pos: (usize, usize),
    pub buffer_pos: usize,
    pub bounds: Bounds,
    pub lock_pos: (usize, usize),
    origin: (usize, usize, usize),
    copy: Option<Box<Cursor>>,
}
impl Cursor {
    pub fn new(x: usize, y: usize, width: usize) -> Self {
        Self {
            screen_pos: (x, y),
            buffer_pos: 0,
            bounds: Bounds::new(y, (4, width)),
            lock_pos: (4, y),
            origin: (x, y, width),
            copy: None,
        }
    }

    fn save_position(&mut self) {
        self.copy = Some(Box::new(self.clone()));
    }

    fn reset_position(&mut self) {
        if let Some(copy) = self.copy.take() {
            *self = *copy.clone();
            self.copy = Some(copy);
        }
    }

    fn move_right(&mut self) {
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
    }

    fn move_left(&mut self, move_type: Move) {
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
}

impl IRust {
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
            self.internal_cursor.screen_pos.0
        };

        let y = if y.is_some() {
            y.unwrap()
        } else {
            self.internal_cursor.screen_pos.1
        };

        self.cursor.goto(x as u16, y as u16)?;

        Ok(())
    }

    pub fn goto_cursor(&mut self) -> Result<(), IRustError> {
        self.cursor.goto(
            self.internal_cursor.screen_pos.0 as u16,
            self.internal_cursor.screen_pos.1 as u16,
        )?;
        Ok(())
    }

    pub fn at_line_end(&self) -> bool {
        self.buffer.is_empty() || self.internal_cursor.buffer_pos == self.buffer.len()
    }

    pub fn move_cursor_left(&mut self, move_type: Move) -> Result<(), IRustError> {
        self.internal_cursor.move_left(move_type);
        self.goto_cursor()?;

        Ok(())
    }

    pub fn move_cursor_right(&mut self) -> Result<(), IRustError> {
        self.internal_cursor.move_right();
        self.goto_cursor()?;

        Ok(())
    }

    pub fn screen_height_overflow_by_str(&self, out: &str) -> usize {
        let new_lines =
            (StringTools::chars_count(out) + self.internal_cursor.screen_pos.0) / self.size.0;

        self.screen_height_overflow_by_new_lines(new_lines)
    }

    pub fn screen_height_overflow_by_new_lines(&self, new_lines: usize) -> usize {
        // if corrected y  + new lines < self.size.1 there is no overflow so unwrap to 0
        (new_lines + self.internal_cursor.screen_pos.1).saturating_sub(self.size.1)
    }

    pub fn save_cursor_position(&mut self) -> Result<(), IRustError> {
        self.cursor.save_position()?;
        self.internal_cursor.save_position();
        Ok(())
    }

    pub fn reset_cursor_position(&mut self) -> Result<(), IRustError> {
        self.cursor.reset_position()?;
        self.internal_cursor.reset_position();
        Ok(())
    }
}
