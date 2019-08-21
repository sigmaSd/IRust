use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
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
    NewLine,
    Welcome,
    Custom(Color),
}

impl Default for PrinterItemType {
    fn default() -> Self {
        PrinterItemType::NewLine
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
            out_type: PrinterItemType::NewLine,
        }
    }
}

impl PrinterItem {
    pub fn new(string: String, out_type: PrinterItemType) -> Self {
        Self { string, out_type }
    }
}

impl IRust {
    pub fn write_input(&mut self) -> Result<(), IRustError> {
        // scroll if needed before writing the input
        let input_last_row = self.cursor.input_last_pos(&self.buffer).1;
        let height_overflow = input_last_row.saturating_sub(self.cursor.bound.height - 1);
        if height_overflow > 0 {
            self.scroll_up(height_overflow);
        }

        self.cursor.save_position()?;
        self.cursor.goto_start();
        self.raw_terminal.clear(ClearType::FromCursorDown)?;
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
                    let _ = self.raw_terminal.set_fg(color);

                    for c in elem.string.chars() {
                        self.write(&c.to_string(), color)?;
                        if self.cursor.is_at_col(super::INPUT_START_COL) {
                            self.write_from_terminal_start("..: ", Color::Yellow)?;
                        }
                    }
                }
                PrinterItemType::NewLine => {
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
        // check if need to scroll
        let new_lines = printer
            .iter()
            .filter(|p| p.out_type == PrinterItemType::NewLine)
            .count();

        let height_overflow = self.cursor.screen_height_overflow_by_new_lines(new_lines);
        if height_overflow > 0 {
            self.scroll_up(height_overflow);
        }

        for output in printer {
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
                PrinterItemType::NewLine => {
                    self.cursor.goto(0, self.cursor.pos.current_pos.1 + 1);
                    self.cursor.use_current_row_as_starting_row();
                    continue;
                }
                PrinterItemType::Custom(color) => color,
            };

            self.raw_terminal.set_fg(color)?;
            if !output.string.is_empty() {
                if StringTools::is_multiline(&output.string) {
                    self.cursor.goto(0, self.cursor.pos.current_pos.1 + 1);

                    output.string.split('\n').for_each(|line| {
                        let _ = self.raw_terminal.write(line);
                        let _ = self.raw_terminal.write("\r\n");
                    });

                    // check if we did scroll automatically
                    // TODO: maybe convert all output.string newlines to printer NewLine
                    // So I can remove this check
                    let new_lines = StringTools::new_lines_count(&output.string);
                    let height_overflow =
                        self.cursor.screen_height_overflow_by_new_lines(new_lines);
                    if height_overflow > 0 {
                        self.raw_terminal.scroll_up(1)?;
                        self.cursor.pos.current_pos.1 = self.cursor.bound.height - 1;
                    } else {
                        self.cursor.pos.current_pos.1 += new_lines;
                    }
                } else {
                    self.raw_terminal.write(&output.string)?;
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
