use crate::irust::IRust;
use crossterm::Color;

#[derive(Clone)]
pub enum OutputType {
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
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::Empty
    }
}

#[derive(Default, Clone)]
pub struct Outputs {
    inner: Vec<Output>,
}
impl Outputs {
    pub fn new(output: Output) -> Self {
        Self {
            inner: vec![output],
        }
    }

    pub fn add_new_line(&mut self, num: usize) {
        for _ in 0..num {
            self.inner.push(Output::default());
        }
    }

    pub fn push(&mut self, output: Output) {
        self.inner.push(output);
    }

    pub fn append(&mut self, other: &mut Self) {
        self.inner.append(&mut other.inner);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Iterator for Outputs {
    type Item = Output;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.inner.is_empty() {
            Some(self.inner.remove(0))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Output {
    string: String,
    out_type: OutputType,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            string: String::new(),
            out_type: OutputType::Empty,
        }
    }
}

impl Output {
    pub fn new(string: String, out_type: OutputType) -> Self {
        Self { string, out_type }
    }
}

impl IRust {
    pub fn write_out(&mut self) -> std::io::Result<()> {
        for output in self.output.clone() {
            let color = match output.out_type {
                OutputType::Eval => self.options.eval_color,
                OutputType::Ok => self.options.ok_color,
                OutputType::Show => self.options.show_color,
                OutputType::IRust => self.options.irust_color,
                OutputType::Warn => self.options.warn_color,
                OutputType::Out => self.options.out_color,
                OutputType::Shell => self.options.shell_color,
                OutputType::Err => self.options.err_color,
                OutputType::Help => Color::White,
                OutputType::Empty => {
                    self.write_newline()?;
                    continue;
                }
            };
            self.color.set_fg(color)?;
            self.write(&output.string)?;
            self.color.reset()?;
        }

        Ok(())
    }
}

pub trait ColoredOutput {
    fn to_output(&self, _color: Color) -> Output;
}

impl<T: ToString> ColoredOutput for T {
    fn to_output(&self, _color: Color) -> Output {
        Output::new(self.to_string(), OutputType::Help)
    }
}
