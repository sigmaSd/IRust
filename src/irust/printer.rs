use crate::irust::{IRust, IRustError, IN};
use crossterm::{ClearType, Color};
use std::iter::FromIterator;

#[derive(Clone)]
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

#[derive(Default, Clone)]
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

    pub fn append(&mut self, other: &mut Self) {
        self.inner.append(&mut other.inner);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
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

#[derive(Clone)]
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
    pub fn write_out(&mut self) -> Result<(), IRustError> {
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
                    self.write_newline()?;
                    continue;
                }
                PrinterItemType::Custom(color) => color,
            };
            self.color.set_fg(color)?;
            if !output.string.is_empty() {
                if crate::utils::StringTools::is_multiline(&output.string) {
                    let _ = self.write_newline();
                    output.string.split('\n').for_each(|o| {
                        let _ = self.terminal.write(o);
                        let _ = self.write_newline();
                    });
                } else {
                    self.terminal.write(&output.string)?;
                }
            }
        }

        Ok(())
    }

    pub fn write_in(&mut self) -> Result<(), IRustError> {
        self.internal_cursor.screen_pos.0 = 0;
        self.goto_cursor()?;
        self.terminal.clear(ClearType::FromCursorDown)?;
        self.color.set_fg(self.options.input_color)?;
        self.write(IN)?;
        //self.internal_cursor.screen_pos.0 = 4;
        *self.internal_cursor.current_bounds_mut() = (4, self.size.0);
        self.internal_cursor.buffer_pos = 0;
        self.internal_cursor.lock_pos = (4, self.internal_cursor.screen_pos.1);
        self.color.reset()?;
        Ok(())
    }

    pub fn write_insert(&mut self, c: Option<&str>) -> Result<(), IRustError> {
        // Clear from cursor down
        self.terminal.clear(ClearType::FromCursorDown)?;

        // Set input color
        self.color.set_fg(self.options.insert_color)?;

        // Write the new input character
        if let Some(c) = c {
            // insert
            self.write(c)?;
        }

        // If the new character is not in the last position
        // rewrite the buffer from the character and on
        if !self.at_line_end() {
            self.save_cursor_position()?;
            for character in self
                .buffer
                .chars()
                .skip(self.internal_cursor.buffer_pos)
                .collect::<Vec<char>>()
                .iter()
            {
                self.write(&character.to_string())?;
            }
            self.reset_cursor_position()?;
        }

        // Reset color
        self.color.reset()?;

        // Unlock racer suggestions update
        let _ = self.unlock_racer_update();

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
