use crate::irust::{IRust, IRustError};
use crossterm::{ClearType, Color};
use std::iter::FromIterator;

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
    pub fn last_line_row(&mut self) -> usize {
        let relative_pos = self.buf.end_to_relative_screen_pos();
        let mut x = relative_pos.0 + 3;
        x += (x / (self.size.0 - 1)) * 3;

        let mut y = relative_pos.1 + self.cursor.pos.starting_pos.1;
        y += x / (self.size.0 - 1);

        y
    }

    pub fn move_screen_cursor_to_last_line(&mut self) {
        let y = self.last_line_row();

        let height_overflow = y.saturating_sub(self.size.1 - 1);

        if height_overflow > 0 {
            self.scroll_up(height_overflow);
        }

        self.cursor.goto(0, y);
    }

    pub fn print(&mut self) -> Result<(), IRustError> {
        let y = self.last_line_row();
        let height_overflow = y.saturating_sub(self.size.1 - 1);
        if height_overflow > 0 {
            self.scroll_up(height_overflow);
        }

        self.cursor.save_position()?;
        self.cursor.goto_start();

        self.terminal.clear(ClearType::FromCursorDown)?;

        self.color.set_fg(Color::Yellow)?;
        self.terminal.write("In: ")?;
        self.cursor.pos.screen_pos.0 = 3;
        self.color.reset()?;

        let input = super::highlight::highlight(&self.buf.to_string());

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
                        if self.cursor.is_at_last_col() {
                            self.cursor.bound_current_row_at_current_col();
                            self.cursor.goto(0, self.cursor.pos.screen_pos.1 + 1);

                            self.color.set_fg(crossterm::Color::Yellow)?;
                            self.terminal.write("..: ")?;
                            self.cursor.pos.screen_pos.0 = 3;
                            self.color.set_fg(color)?;
                        }
                        self.terminal.write(c)?;
                        self.cursor.pos.screen_pos.0 += 1;
                    }
                }
                PrinterItemType::Empty => {
                    self.cursor
                        .bound
                        .set_bound(self.cursor.pos.screen_pos.1, self.cursor.pos.screen_pos.0);
                    self.cursor.goto(0, self.cursor.pos.screen_pos.1 + 1);

                    self.color.set_fg(crossterm::Color::Yellow)?;
                    self.terminal.write("..: ")?;
                    self.cursor.pos.screen_pos.0 = 3;
                    self.color.reset()?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn write_out(&mut self) -> Result<(), IRustError> {
        let new_lines = self
            .printer
            .iter()
            .filter(|p| p.out_type == PrinterItemType::Empty)
            .count();

        let overflow = (new_lines + self.cursor.pos.screen_pos.1).saturating_sub(self.size.1 - 1);
        if overflow > 0 {
            self.scroll_up(overflow);
        }

        for output in self.printer.clone() {
            let color = match output.out_type {
                PrinterItemType::Eval => self.options.eval_color,
                PrinterItemType::Ok => self.options.ok_color,
                PrinterItemType::_IRust => self.options.irust_color,
                PrinterItemType::Warn => self.options.irust_warn_color,
                PrinterItemType::Out => self.options.out_color,
                PrinterItemType::Shell => self.options.shell_color,
                PrinterItemType::Err => self.options.err_color,
                PrinterItemType::Welcome => {
                    self.color.set_fg(self.options.welcome_color)?;
                    let msg = if !self.options.welcome_msg.is_empty() {
                        self.fit_msg(&self.options.welcome_msg.clone())
                    } else {
                        self.fit_msg(&output.string)
                    };
                    self.write(&msg)?;
                    continue;
                }
                PrinterItemType::Empty => {
                    self.cursor.goto(0, self.cursor.pos.screen_pos.1 + 1);
                    self.cursor.pos.starting_pos.1 = self.cursor.pos.screen_pos.1;
                    continue;
                }
                PrinterItemType::Custom(color) => color,
            };

            self.color.set_fg(color)?;
            if !output.string.is_empty() {
                if crate::utils::StringTools::is_multiline(&output.string) {
                    let overflow = (output.string.chars().filter(|c| *c == '\n').count()
                        + self.cursor.pos.screen_pos.1)
                        .saturating_sub(self.size.1 - 1);
                    if overflow > 0 {
                        self.scroll_up(overflow);
                    }
                    self.cursor.goto(0, self.cursor.pos.screen_pos.1 + 1);
                    output.string.split('\n').for_each(|o| {
                        let _ = self.terminal.write(o);
                        self.cursor.goto(0, self.cursor.pos.screen_pos.1 + 1);
                        //let _ = self.write_newline();
                        //                        self.cursor.goto(0, self.cursor.pos.screen_pos.1 + 1);
                        //                        self.cursor.pos.starting_pos.1 = self.cursor.pos.screen_pos.1;
                    });
                } else {
                    self.terminal.write(&output.string)?;
                    //self.cursor.goto(0, self.cursor.pos.screen_pos.1 +1);
                    //self.cursor.pos.starting_pos.1 = self.cursor.pos.screen_pos.1;
                }
            }
        }

        Ok(())
    }

    pub fn write_in(&mut self) -> Result<(), IRustError> {
        self.color.set_fg(crossterm::Color::Yellow)?;
        self.terminal.write("In: ")?;
        self.color.reset()?;
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
