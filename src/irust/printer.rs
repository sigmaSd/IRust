use crate::irust::{IRust, IN};
use crossterm::{ClearType, Color};

#[derive(Clone)]
pub enum PrinterItemType {
    Eval,
    Ok,
    Show,
    IRust,
    Warn,
    Out,
    Shell,
    Err,
    Help,
    Empty,
    Welcome,
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
    pub fn write_out(&mut self) -> std::io::Result<()> {
        for output in self.printer.clone() {
            let color = match output.out_type {
                PrinterItemType::Eval => self.options.eval_color,
                PrinterItemType::Ok => self.options.ok_color,
                PrinterItemType::Show => self.options.show_color,
                PrinterItemType::IRust => self.options.irust_color,
                PrinterItemType::Warn => self.options.irust_warn_color,
                PrinterItemType::Out => self.options.out_color,
                PrinterItemType::Shell => self.options.shell_color,
                PrinterItemType::Err => self.options.err_color,
                PrinterItemType::Help => Color::Cyan,
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
            };
            self.color.set_fg(color)?;
            self.write(&output.string)?;
        }

        Ok(())
    }

    pub fn write_in(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(0, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        self.color.set_fg(self.options.input_color)?;
        self.terminal.write(IN)?;
        self.color.reset()?;
        Ok(())
    }

    pub fn write_insert(&mut self, c: Option<&str>) -> std::io::Result<()> {
        self.terminal.clear(ClearType::UntilNewLine)?;

        self.color.set_fg(self.options.insert_color)?;

        if let Some(c) = c {
            self.write(c)?;
        }

        self.cursor.save_position()?;

        for character in self
            .buffer
            .chars()
            .skip(self.internal_cursor.x)
            .collect::<Vec<char>>()
            .iter()
        {
            self.terminal.write(&character.to_string())?;
        }
        self.cursor.reset_position()?;
        self.color.reset()?;

        // debounce from update calls
        if self.debouncer.check().is_err() {
            return Ok(());
        }
        self.unlock_racer_update();
        self.update_suggestions()?;
        if let Some(character) = self.buffer.chars().last() {
            if character.is_alphanumeric() {
                self.write_next_suggestion()?;
            }
        }

        Ok(())
    }
}

pub trait ColoredPrinterItem {
    fn to_output(&self, _color: Color) -> PrinterItem;
}

impl<T: ToString> ColoredPrinterItem for T {
    fn to_output(&self, _color: Color) -> PrinterItem {
        PrinterItem::new(self.to_string(), PrinterItemType::Help)
    }
}
