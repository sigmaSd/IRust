use super::highlight::highlight;
use crate::irust::IRustError;
use crossterm::{style::Color, terminal::ClearType};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

mod cursor;
mod writer;

#[derive(Debug, Clone)]
pub struct Printer<W: std::io::Write> {
    printer: PrintQueue,
    pub writer: writer::Writer<W>,
    pub cursor: cursor::Cursor<W>,
}

impl Default for Printer<std::io::Stdout> {
    fn default() -> Self {
        crossterm::terminal::enable_raw_mode().expect("failed to enable raw_mode");
        Self {
            printer: Default::default(),
            writer: writer::Writer::default(),
            cursor: cursor::Cursor::default(),
        }
    }
}

impl<W: std::io::Write> Printer<W> {
    // for tests
    pub fn _new(raw: W) -> Printer<W> {
        crossterm::terminal::enable_raw_mode().expect("failed to enable raw_mode");
        let raw = Rc::new(RefCell::new(raw));
        Self {
            printer: PrintQueue::default(),
            writer: writer::Writer::_new(raw.clone()),
            cursor: cursor::Cursor::_new(raw),
        }
    }
}

impl<W: std::io::Write> Drop for Printer<W> {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

#[derive(Debug, Default, Clone)]
pub struct PrintQueue {
    items: VecDeque<PrinterItem>,
}

impl PrintQueue {
    pub fn add_new_line(&mut self, num: usize) {
        for _ in 0..num {
            self.items.push_back(PrinterItem::NewLine);
        }
    }

    pub fn push(&mut self, item: PrinterItem) {
        self.items.push_back(item);
    }

    pub fn append(&mut self, other: &mut Self) {
        self.items.append(&mut other.items);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Iterator for PrintQueue {
    type Item = PrinterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_front()
    }
}

#[derive(Debug, Clone)]
pub enum PrinterItem {
    Char(char, Color),
    String(String, Color),
    Str(&'static str, Color),
    NewLine,
}

impl<W: std::io::Write> Printer<W> {
    pub fn print_input(
        &mut self,
        buffer: &crate::irust::Buffer,

        theme: &super::Theme,
    ) -> Result<(), IRustError> {
        if self.check_for_offscreen_render_hack(&buffer)? {
            return Ok(());
        }

        self.cursor.hide();
        // scroll if needed before writing the input
        self.scroll_if_needed_for_input(&buffer);
        self.cursor.save_position();
        self.cursor.goto_start();
        self.writer.raw.clear(ClearType::FromCursorDown)?;

        self.writer
            .write_from_terminal_start(super::IN, Color::Yellow, &mut self.cursor)?;

        self.print_input_inner(highlight(&buffer.buffer, theme))?;

        self.cursor.restore_position();
        self.cursor.show();

        Ok(())
    }

    fn print_input_inner(&mut self, printer: PrintQueue) -> Result<(), IRustError> {
        for item in printer {
            match item {
                PrinterItem::String(string, color) => {
                    self.print_input_str(&string, color)?;
                }
                PrinterItem::Str(string, color) => {
                    self.print_input_str(&string, color)?;
                }
                PrinterItem::Char(c, color) => {
                    self.print_input_char(c, color)?;
                }
                PrinterItem::NewLine => {
                    self.cursor.bound_current_row_at_current_col();
                    self.cursor.goto_next_row_terminal_start();
                    self.writer.write("..: ", Color::Yellow, &mut self.cursor)?;
                }
            }
        }

        Ok(())
    }

    fn print_input_str(&mut self, string: &str, color: Color) -> Result<(), IRustError> {
        for c in string.chars() {
            self.print_input_char(c, color)?;
        }
        Ok(())
    }

    fn print_input_char(&mut self, c: char, color: Color) -> Result<(), IRustError> {
        self.writer
            .write_char_with_color(c, color, &mut self.cursor)?;
        if self.cursor.is_at_last_terminal_col() {
            self.cursor.bound_current_row_at_current_col();
        }
        if self.cursor.is_at_col(cursor::INPUT_START_COL) {
            self.writer
                .write_from_terminal_start("..: ", Color::Yellow, &mut self.cursor)?;
        }
        Ok(())
    }

    pub fn print_output(&mut self, printer: PrintQueue) -> Result<(), IRustError> {
        for item in printer {
            match item {
                PrinterItem::Char(c, color) => {
                    self.writer.raw.set_fg(color)?;
                    self.writer.raw.write(c)?;
                }
                PrinterItem::String(string, color) => {
                    self.print_out_str(&string, color)?;
                }
                PrinterItem::Str(string, color) => {
                    self.print_out_str(&string, color)?;
                }
                PrinterItem::NewLine => {
                    if self.cursor.pos.current_pos.1 >= self.cursor.bound.height - 1 {
                        self.writer.raw.scroll_up(1)?;
                    }
                    self.cursor.goto_next_row_terminal_start();
                    self.cursor.use_current_row_as_starting_row();
                }
            }
        }
        self.readjust_cursor_pos()?;

        Ok(())
    }

    fn print_out_str(&mut self, string: &str, color: Color) -> Result<(), IRustError> {
        self.writer.raw.set_fg(color)?;
        self.writer.raw.write(&string.replace('\n', "\r\n"))?;
        let rows = string.match_indices('\n').count();
        self.cursor.pos.current_pos.1 += rows;
        Ok(())
    }

    pub fn scroll_if_needed_for_input(&mut self, buffer: &crate::irust::Buffer) {
        let input_last_row = self.cursor.input_last_pos(&buffer).1;

        let height_overflow = input_last_row.saturating_sub(self.cursor.bound.height - 1);
        if height_overflow > 0 {
            self.writer.scroll_up(height_overflow, &mut self.cursor);
        }
    }

    fn readjust_cursor_pos(&mut self) -> Result<(), IRustError> {
        // check if we did scroll automatically
        // if we did update current_pos.1  and starting_pos.1 to the height of the terminal (-1)
        if self.cursor.pos.current_pos.1 > self.cursor.bound.height - 1 {
            self.cursor.pos.current_pos.1 = self.cursor.bound.height - 1;
            self.cursor.pos.starting_pos.1 = self.cursor.bound.height - 1;
        }
        Ok(())
    }

    fn check_for_offscreen_render_hack(
        &mut self,
        buffer: &crate::irust::Buffer,
    ) -> Result<bool, IRustError> {
        // Hack
        if self.cursor.buffer_pos_to_cursor_pos(&buffer).1 >= self.cursor.bound.height {
            let mut p = PrintQueue::default();
            p.push(PrinterItem::Str("\rIt looks like the input is larger then the termnial, either use the `:edit` command or enlarge the terminal. hit ctrl-c to continue", Color::Red));
            self.print_output(p)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn adjust(&mut self) {
        self.cursor.move_right_unbounded();
        if self.cursor.is_at_last_terminal_col() {
            self.cursor.bound_current_row_at_current_col();
        }
        if self.cursor.is_at_col(cursor::INPUT_START_COL) {
            for _ in 0..4 {
                self.cursor.move_right_unbounded();
            }
        }
    }
    pub fn recalculate_bounds(&mut self, printer: PrintQueue) -> Result<(), IRustError> {
        self.cursor.save_position();
        self.cursor.goto_start();
        for _ in 0..4 {
            self.cursor.move_right_unbounded();
        }
        for item in printer {
            match item {
                PrinterItem::String(string, _) => {
                    for _ in string.chars() {
                        self.adjust();
                    }
                }
                PrinterItem::Str(string, _) => {
                    for _ in string.chars() {
                        self.adjust();
                    }
                }
                PrinterItem::Char(_, _) => {
                    self.adjust();
                }
                PrinterItem::NewLine => {
                    self.cursor.bound_current_row_at_current_col();
                    self.cursor.goto_next_row_terminal_start();
                    for _ in 0..4 {
                        self.cursor.move_right_unbounded();
                    }
                }
            }
        }
        self.cursor.restore_position();

        Ok(())
    }
}

impl<W: std::io::Write> Printer<W> {
    pub fn write_from_terminal_start(&mut self, out: &str, color: Color) -> Result<(), IRustError> {
        self.writer
            .write_from_terminal_start(out, color, &mut self.cursor)
    }
    pub fn clear(&mut self) -> Result<(), IRustError> {
        self.writer.clear(&mut self.cursor)
    }
    pub fn clear_last_line(&mut self) -> Result<(), IRustError> {
        self.writer.clear_last_line(&mut self.cursor)
    }

    pub fn write_newline(
        &mut self,
        buffer: &crate::irust::buffer::Buffer,
    ) -> Result<(), IRustError> {
        self.writer.write_newline(&mut self.cursor, buffer)
    }

    pub fn write(&mut self, out: &str, color: Color) -> Result<(), IRustError> {
        self.writer.write(out, color, &mut self.cursor)
    }

    pub fn write_at(&mut self, s: &str, x: usize, y: usize) -> Result<(), IRustError> {
        self.writer.write_at(s, x, y, &mut self.cursor)
    }
    pub fn write_at_no_cursor(
        &mut self,
        s: &str,
        color: Color,
        x: usize,
        y: usize,
    ) -> Result<(), IRustError> {
        self.writer
            .write_at_no_cursor(s, color, x, y, &mut self.cursor)
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.writer.scroll_up(n, &mut self.cursor)
    }

    pub fn update_dimensions(&mut self, width: u16, height: u16) {
        self.cursor.bound.width = width as usize;
        self.cursor.bound.height = height as usize;
    }
}
