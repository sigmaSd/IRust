use crate::irust::Result;
use crossterm::{style::Color, terminal::ClearType};
mod raw;
use raw::Raw;
use std::{cell::RefCell, rc::Rc};

use super::{cursor::INPUT_START_COL, ONE_LENGTH_CHAR};

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
    ) -> Result<()> {
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
    ) -> Result<()> {
        // Performance: set_fg only when needed
        if self.last_color != Some(color) {
            self.raw.set_fg(color)?;
        }
        self.write_char(c, cursor)?;
        self.last_color = Some(color);
        Ok(())
    }

    pub fn write_char(&mut self, c: char, cursor: &mut super::cursor::Cursor<W>) -> Result<()> {
        let current_char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        //FIXME
        dbg!(cursor.pos, current_char_width, &cursor.bound.width);
        let h_overshoot = (cursor.pos.current_pos.0 as isize + current_char_width as isize)
            - (cursor.bound.width as isize);

        if h_overshoot == 0 {
            self.raw.write(c)?;
            cursor.pos.current_pos.0 += current_char_width;
            //cursor.bound_current_row_at_current_col();
            cursor.bound.set_bound(
                cursor.pos.current_pos.1,
                cursor.bound.width - current_char_width,
            );
            cursor.pos.current_pos.0 = INPUT_START_COL;
            cursor.pos.current_pos.1 += 1;
        } else if h_overshoot > 0 {
            cursor.bound_current_row_at_current_col();
            cursor.pos.current_pos.0 = INPUT_START_COL;
            cursor.pos.current_pos.1 += 1;
            cursor.goto_internal_pos();
            self.raw.write(c)?;
            cursor.pos.current_pos.0 += current_char_width;
        } else {
            self.raw.write(c)?;
            cursor.pos.current_pos.0 += current_char_width;
        }
        cursor.goto_internal_pos();

        Ok(())
    }

    pub fn write_at(
        &mut self,
        s: &str,
        x: usize,
        y: usize,
        cursor: &mut super::cursor::Cursor<W>,
    ) -> Result<()> {
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
    ) -> Result<()> {
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
    ) -> Result<()> {
        cursor.goto(0, cursor.pos.current_pos.1);
        self.write(out, color, cursor)?;
        Ok(())
    }

    pub fn write_newline(
        &mut self,
        cursor: &mut super::cursor::Cursor<W>,
        buffer: &crate::irust::buffer::Buffer,
    ) -> Result<()> {
        cursor.move_to_input_last_row(buffer);

        // check for scroll
        if cursor.is_at_last_terminal_row() {
            self.scroll_up(1, cursor);
        }

        cursor.move_down(1);
        cursor.use_current_row_as_starting_row();

        Ok(())
    }

    pub fn clear(&mut self, cursor: &mut super::cursor::Cursor<W>) -> Result<()> {
        self.raw.clear(ClearType::All)?;

        cursor.pos.starting_pos = (0, 0);
        cursor.goto(4, 0);
        cursor.bound.reset();
        //self.print_input()?;
        Ok(())
    }

    pub fn clear_last_line(&mut self, cursor: &mut super::cursor::Cursor<W>) -> Result<()> {
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
