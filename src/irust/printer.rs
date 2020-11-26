use super::highlight::highlight;
use crate::irust::{IRust, IRustError};
use crossterm::{style::Color, terminal::ClearType};
use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct Printer {
    items: VecDeque<PrinterItem>,
}
impl Printer {
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

impl Iterator for Printer {
    type Item = PrinterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_front()
    }
}

#[derive(Debug, Clone)]
pub enum PrinterItem {
    Char(char, Color),
    String(String, Color),
    NewLine,
}

impl IRust {
    pub fn print_input(&mut self) -> Result<(), IRustError> {
        self.cursor.hide();
        // scroll if needed before writing the input
        self.scroll_if_needed_for_input();
        self.cursor.save_position();
        self.cursor.goto_start();
        self.raw_terminal.clear(ClearType::FromCursorDown)?;

        self.write_from_terminal_start(super::IN, Color::Yellow)?;
        self.print_inner(highlight(self.buffer.to_string(), &self.theme))?;

        self.cursor.restore_position();
        self.cursor.show();

        Ok(())
    }

    fn print_inner(&mut self, printer: Printer) -> Result<(), IRustError> {
        for item in printer {
            match item {
                PrinterItem::String(string, color) => {
                    for c in string.chars() {
                        self.print_char(c, color)?;
                    }
                }
                PrinterItem::Char(c, color) => {
                    self.print_char(c, color)?;
                }
                PrinterItem::NewLine => {
                    self.cursor.bound_current_row_at_current_col();
                    self.cursor.goto_next_row_terminal_start();
                    self.write("..: ", Color::Yellow)?;
                }
            }
        }

        Ok(())
    }

    fn print_char(&mut self, c: char, color: Color) -> Result<(), IRustError> {
        if c == '\n' {
            self.cursor.bound_current_row_at_current_col();
            self.cursor.goto_next_row_terminal_start();
            self.write("..: ", Color::Yellow)?;
        } else {
            self.write_char_with_color(c, color)?;
            if self.cursor.is_at_last_terminal_col() {
                self.cursor.bound_current_row_at_current_col();
            }
            if self.cursor.is_at_col(super::INPUT_START_COL) {
                self.write_from_terminal_start("..: ", Color::Yellow)?;
            }
        }
        Ok(())
    }

    pub fn print_output(&mut self, printer: Printer) -> Result<(), IRustError> {
        for item in printer {
            match item {
                PrinterItem::Char(c, color) => {
                    self.raw_terminal.set_fg(color)?;
                    self.raw_terminal.write(c)?;
                }
                PrinterItem::String(string, color) => {
                    self.raw_terminal.set_fg(color)?;
                    self.raw_terminal.write(&string.replace('\n', "\r\n"))?;
                    let rows = string.match_indices('\n').count();
                    self.cursor.pos.current_pos.1 += rows;
                }
                PrinterItem::NewLine => {
                    if self.cursor.pos.current_pos.1 >= self.cursor.bound.height - 1 {
                        self.raw_terminal.scroll_up(1)?;
                    }
                    self.cursor.goto_next_row_terminal_start();
                    self.cursor.use_current_row_as_starting_row();
                }
            }
        }
        self.readjust_cursor_pos()?;

        Ok(())
    }

    // helper fns

    fn scroll_if_needed_for_input(&mut self) {
        let input_last_row = self.cursor.input_last_pos(&self.buffer).1;
        let height_overflow = input_last_row.saturating_sub(self.cursor.bound.height - 1);
        if height_overflow > 0 {
            self.scroll_up(height_overflow);
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
}
