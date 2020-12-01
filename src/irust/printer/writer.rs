use crate::irust::IRustError;
use crossterm::{style::Color, terminal::ClearType};
mod raw;
use raw::Raw;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct Writer<W: std::io::Write> {
    last_color: Option<Color>,
    pub raw: Raw<W>,
}

impl<W: std::io::Write> Writer<W> {
    pub fn new(raw: Rc<RefCell<W>>) -> Self {
        let raw = Raw { raw };
        Self {
            last_color: None,
            raw,
        }
    }

    pub fn write(
        &mut self,
        out: &str,
        color: Color,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        // Performance: set_fg only when needed
        if self.last_color != Some(color) {
            self.raw.set_fg(color)?;
        }

        for c in out.chars() {
            self.write_char(c, cursor)?;
        }

        self.last_color = Some(color);
        Ok(())
    }

    pub fn write_char_with_color(
        &mut self,
        c: char,
        color: Color,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        // Performance: set_fg only when needed
        if self.last_color != Some(color) {
            self.raw.set_fg(color)?;
        }
        self.write_char(c, cursor)?;
        self.last_color = Some(color);
        Ok(())
    }

    pub fn write_char(
        &mut self,
        c: char,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        self.raw.write(c)?;
        // Performance: Make sure to not move the cursor if cursor_pos = last_cursor_pos+1 because it moves automatically
        if cursor.pos.current_pos.0 == cursor.bound.width - 1 {
            cursor.pos.current_pos.0 = 4;
            cursor.pos.current_pos.1 += 1;
            cursor.goto_internal_pos();
        } else {
            cursor.pos.current_pos.0 += 1;
            // tab move the cursor by 4
            // need to adjust the screen cursor
            if c == '\t' {
                cursor.goto_internal_pos();
            }
        }
        Ok(())
    }

    pub fn write_at(
        &mut self,
        s: &str,
        x: usize,
        y: usize,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        cursor.goto(x, y);
        self.raw.write(s)?;
        Ok(())
    }

    pub fn write_at_no_cursor(
        &mut self,
        s: &str,
        color: Color,
        x: usize,
        y: usize,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        self.raw.set_fg(color)?;
        let origin_pos = cursor.pos.current_pos;
        self.write_at(s, x, y, cursor)?;
        cursor.goto(origin_pos.0, origin_pos.1);
        self.raw.reset_color()?;
        Ok(())
    }

    pub fn write_from_terminal_start(
        &mut self,
        out: &str,
        color: Color,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        cursor.goto(0, cursor.pos.current_pos.1);
        self.write(out, color, cursor)?;
        Ok(())
    }

    pub fn write_newline(
        &mut self,
        cursor: &mut super::cursor::Cursor<W>,
        buffer: &crate::irust::buffer::Buffer,
    ) -> Result<(), IRustError> {
        cursor.move_to_input_last_row(buffer);

        // check for scroll
        if cursor.is_at_last_terminal_row() {
            self.scroll_up(1, cursor);
        }

        cursor.move_down(1);
        cursor.use_current_row_as_starting_row();

        Ok(())
    }

    pub fn clear(&mut self, cursor: &mut super::cursor::Cursor<W>) -> Result<(), IRustError> {
        self.raw.clear(ClearType::All)?;

        cursor.pos.starting_pos = (0, 0);
        cursor.goto(4, 0);
        cursor.bound.reset();
        //self.print_input()?;
        Ok(())
    }

    pub fn clear_last_line(
        &mut self,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<(), IRustError> {
        let origin_pos = cursor.pos.current_pos;
        cursor.goto(0, cursor.bound.height - 1);
        self.raw.clear(ClearType::CurrentLine)?;
        cursor.goto(origin_pos.0, origin_pos.1);
        Ok(())
    }

    pub fn scroll_up(&mut self, n: usize, cursor: &mut super::cursor::Cursor<W>) {
        self.raw.scroll_up(n as u16).expect("failed to scroll-up");
        cursor.move_up(n as u16);
        cursor.pos.starting_pos.1 = cursor.pos.starting_pos.1.saturating_sub(n);
    }
}
