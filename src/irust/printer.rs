use super::highlight::highlight;
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
use crossterm::{style::Color, terminal::ClearType};
use std::iter::FromIterator;

#[derive(Debug, Default, Clone)]
pub struct Printer {
    items: Vec<PrinterItem>,
}
impl Printer {
    pub fn new(output: PrinterItem) -> Self {
        Self {
            items: vec![output],
        }
    }

    pub fn add_new_line(&mut self, num: usize) {
        for _ in 0..num {
            self.items.push(PrinterItem::default());
        }
    }

    pub fn push(&mut self, output: PrinterItem) {
        self.items.push(output);
    }

    pub fn _pop(&mut self) -> Option<PrinterItem> {
        self.items.pop()
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
        if !self.items.is_empty() {
            Some(self.items.remove(0))
        } else {
            None
        }
    }
}

impl FromIterator<PrinterItem> for Printer {
    fn from_iter<I: IntoIterator<Item = PrinterItem>>(printer_items: I) -> Self {
        let mut printer = Printer::default();
        for printer_item in printer_items {
            printer.push(printer_item);
        }
        printer
    }
}

#[derive(Debug, Clone)]
pub struct PrinterItem {
    string: String,
    string_type: PrinterItemType,
}

impl Default for PrinterItem {
    fn default() -> Self {
        Self {
            string: String::new(),
            string_type: PrinterItemType::NewLine,
        }
    }
}

impl PrinterItem {
    pub fn new(string: String, string_type: PrinterItemType) -> Self {
        Self {
            string,
            string_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrinterItemType {
    Eval,
    Ok,
    _IRust,
    _Warn,
    Out,
    Shell,
    Err,
    NewLine,
    Custom(Color),
}

impl Default for PrinterItemType {
    fn default() -> Self {
        PrinterItemType::NewLine
    }
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
        for elem in printer {
            match elem.string_type {
                PrinterItemType::Custom(color) => {
                    for c in elem.string.chars() {
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
                    }
                }
                PrinterItemType::NewLine => {
                    self.cursor.bound_current_row_at_current_col();
                    self.cursor.goto_next_row_terminal_start();
                    self.write("..: ", Color::Yellow)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn print_output(&mut self, printer: Printer) -> Result<(), IRustError> {
        for output in printer {
            let color = match output.string_type {
                PrinterItemType::Eval => self.options.eval_color,
                PrinterItemType::Ok => self.options.ok_color,
                PrinterItemType::_IRust => self.options.irust_color,
                PrinterItemType::_Warn => self.options.irust_warn_color,
                PrinterItemType::Out => self.options.out_color,
                PrinterItemType::Shell => self.options.shell_color,
                PrinterItemType::Err => self.options.err_color,
                PrinterItemType::Custom(color) => color,
                PrinterItemType::NewLine => {
                    self.cursor.goto_next_row_terminal_start();
                    self.cursor.use_current_row_as_starting_row();
                    continue;
                }
            };

            self.raw_terminal.set_fg(color)?;
            if StringTools::is_multiline(&output.string) {
                for line in output.string.split('\n') {
                    self.raw_terminal.write(line)?;
                    self.raw_terminal.write("\r\n")?;
                    self.cursor.pos.current_pos.1 += 1;
                }
            } else if &output.string == "\n" {
                // check if we need to scroll up
                if self.cursor.pos.current_pos.1 >= self.cursor.bound.height - 1 {
                    self.raw_terminal.scroll_up(1)?;
                }
                self.cursor.goto_next_row_terminal_start();
            } else {
                self.raw_terminal.write(&output.string)?;
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
        // if we did scroll, then scroll the terminal by another row
        // and update current_pos.1  and starting_pos.1 to the height of the terminal (-1)
        if self.cursor.pos.current_pos.1 > self.cursor.bound.height - 1 {
            self.raw_terminal.scroll_up(1)?;
            self.cursor.pos.current_pos.1 = self.cursor.bound.height - 1;
            self.cursor.pos.starting_pos.1 = self.cursor.bound.height - 1;
        }
        Ok(())
    }
}
