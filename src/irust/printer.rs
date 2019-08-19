use crate::irust::{IRust, IRustError};
use crossterm::{ClearType, Color};
use std::iter::FromIterator;

/// input is shown with x in this example
/// |In: x
/// |    x
/// |    x
pub const INPUT_START_COL: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum PrinterItemType {
    Eval,
    Ok,
    _IRust,
    Warn,
    Out,
    Shell,
    Err,
    Empty,
    Welcome,
    Custom(Color),
}

impl Default for PrinterItemType {
    fn default() -> Self {
        PrinterItemType::Empty
    }
}

#[derive(Debug, Default, Clone)]
pub struct Printer {
    inner: Vec<PrinterItem>,
}
impl Printer {
    pub fn new(output: PrinterItem) -> Self {
        Self {
            inner: vec![output],
        }
    }

    pub fn add_new_line(&mut self, num: usize) {
        for _ in 0..num {
            self.inner.push(PrinterItem::default());
        }
    }

    pub fn push(&mut self, output: PrinterItem) {
        self.inner.push(output);
    }

    pub fn pop(&mut self) -> Option<PrinterItem> {
        self.inner.pop()
    }

    pub fn append(&mut self, other: &mut Self) {
        self.inner.append(&mut other.inner);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PrinterItem> {
        self.inner.iter()
    }
}

impl Iterator for Printer {
    type Item = PrinterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.inner.is_empty() {
            Some(self.inner.remove(0))
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
    out_type: PrinterItemType,
}

impl Default for PrinterItem {
    fn default() -> Self {
        Self {
            string: String::new(),
            out_type: PrinterItemType::Empty,
        }
    }
}

impl PrinterItem {
    pub fn new(string: String, out_type: PrinterItemType) -> Self {
        Self { string, out_type }
    }
}

impl IRust {
    pub fn input_last_pos(&mut self) -> (usize, usize) {
        let relative_pos = self.buffer.end_to_relative_current_pos();
        let x = relative_pos.0 + INPUT_START_COL;
        let y = relative_pos.1 + self.cursor.pos.starting_pos.1;

        (x, y)
    }

    pub fn move_screen_cursor_to_last_line(&mut self) {
        let input_last_row = self.input_last_pos().1;

        let height_overflow = input_last_row.saturating_sub(self.cursor.bound.height - 1);

        if height_overflow > 0 {
            self.scroll_up(height_overflow);
        }

        self.cursor.goto(0, input_last_row);
    }

    pub fn write_input(&mut self) -> Result<(), IRustError> {
        // scroll if needed before writing the input
        let input_last_row = self.input_last_pos().1;
        let height_overflow = input_last_row.saturating_sub(self.cursor.bound.height - 1);
        if height_overflow > 0 {
            self.scroll_up(height_overflow);
        }

        self.cursor.save_position()?;
        self.cursor.goto_start();
        self.terminal.clear(ClearType::FromCursorDown)?;
        self.write_from_terminal_start("In: ", Color::Yellow)?;

        let input = super::highlight::highlight(&self.buffer.to_string());
        self.print_inner(input)?;
        self.cursor.reset_position()?;

        Ok(())
    }

    pub fn print_inner(&mut self, printer: Printer) -> Result<(), IRustError> {
        for elem in printer {
            match elem.out_type {
                PrinterItemType::Custom(color) => {
                    let _ = self.color.set_fg(color);

                    for c in elem.string.chars() {
                        self.write(&c.to_string(), color)?;
                        if self.cursor.is_at_col(INPUT_START_COL) {
                            self.write_from_terminal_start("..: ", Color::Yellow)?;
                        }
                    }
                }
                PrinterItemType::Empty => {
                    self.cursor.bound_current_row_at_current_col();
                    self.cursor.goto(0, self.cursor.pos.current_pos.1 + 1);
                    self.write_from_terminal_start("..: ", Color::Yellow)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn write_output(&mut self, printer: Printer) -> Result<(), IRustError> {
        let new_lines = printer
            .iter()
            .filter(|p| p.out_type == PrinterItemType::Empty)
            .count();

        let overflow = (new_lines + self.cursor.pos.current_pos.1)
            .saturating_sub(self.cursor.bound.height - 1);
        if overflow > 0 {
            self.scroll_up(overflow);
        }

        for output in printer.clone() {
            let color = match output.out_type {
                PrinterItemType::Eval => self.options.eval_color,
                PrinterItemType::Ok => self.options.ok_color,
                PrinterItemType::_IRust => self.options.irust_color,
                PrinterItemType::Warn => self.options.irust_warn_color,
                PrinterItemType::Out => self.options.out_color,
                PrinterItemType::Shell => self.options.shell_color,
                PrinterItemType::Err => self.options.err_color,
                PrinterItemType::Welcome => {
                    let msg = if !self.options.welcome_msg.is_empty() {
                        self.fit_msg(&self.options.welcome_msg.clone())
                    } else {
                        self.fit_msg(&output.string)
                    };
                    self.write(&msg, self.options.welcome_color)?;
                    continue;
                }
                PrinterItemType::Empty => {
                    self.cursor.goto(0, self.cursor.pos.current_pos.1 + 1);
                    self.cursor.pos.starting_pos.1 = self.cursor.pos.current_pos.1;
                    continue;
                }
                PrinterItemType::Custom(color) => color,
            };

            self.color.set_fg(color)?;
            if !output.string.is_empty() {
                if crate::utils::StringTools::is_multiline(&output.string) {
                    self.cursor.goto(0, self.cursor.pos.current_pos.1 + 1);

                    output.string.split('\n').for_each(|o| {
                        let _ = self.terminal.write(o);
                        let _ = self.terminal.write("\r\n");
                    });

                    // check if we scrolled
                    let new_lines = (output.string.chars().filter(|c| *c == '\n')).count();
                    let overflow = (new_lines + self.cursor.pos.current_pos.1)
                        .saturating_sub(self.cursor.bound.height - 1);
                    if overflow > 0 {
                        self.terminal.scroll_up(1)?;
                        self.cursor.pos.current_pos.1 = self.cursor.bound.height - 1;
                    } else {
                        self.cursor.pos.current_pos.1 += new_lines;
                    }
                } else {
                    self.terminal.write(&output.string)?;
                }
            }
        }

        Ok(())
    }
}

pub trait ColoredPrinterItem {
    fn to_output(&self, _color: Color) -> PrinterItem;
}

impl<T: ToString> ColoredPrinterItem for T {
    fn to_output(&self, color: Color) -> PrinterItem {
        PrinterItem::new(self.to_string(), PrinterItemType::Custom(color))
    }
}
