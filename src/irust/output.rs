use crate::irust::IRust;
use crossterm::Color;

#[derive(Default, Clone)]
pub struct Output {
    strings_and_colors: Vec<(String, Color)>,
    new_lines_idx: Vec<usize>,
}

impl Output {
    pub fn new<P: Into<Option<Color>>>(string: String, color: P) -> Self {
        let color = color.into().unwrap_or(Color::White);

        Self {
            strings_and_colors: vec![(string.trim_end_matches('\n').to_string(), color)],
            new_lines_idx: vec![],
        }
    }

    pub fn _push<P: Into<Option<Color>>>(&mut self, string: String, color: P) {
        let color = color.into().unwrap_or(Color::White);

        self.strings_and_colors
            .push((string.trim_end_matches('\n').to_string(), color));
    }

    pub fn add_new_line(&mut self) -> &mut Self {
        self.new_lines_idx.push(self.strings_and_colors.len());
        self
    }

    pub fn finish(&mut self) -> Output {
        self.clone()
    }

    pub fn append(&mut self, mut other: Self) {
        self.strings_and_colors
            .append(&mut other.strings_and_colors);
        self.new_lines_idx.append(&mut other.new_lines_idx);
    }

    pub fn is_empty(&self) -> bool {
        self.strings_and_colors.is_empty()
    }
}

impl IRust {
    pub fn write_out(&mut self) -> std::io::Result<()> {
        for (idx, (string, color)) in self.output.strings_and_colors.clone().iter().enumerate() {
            if self.output.new_lines_idx.contains(&idx) {
                self.write_newline()?;
            }
            self.color.set_fg(*color)?;
            self.write(&string)?;
            self.color.reset()?;
        }
        // check for a final new line
        if self
            .output
            .new_lines_idx
            .contains(&self.output.strings_and_colors.len())
        {
            self.write_newline()?;
        }

        Ok(())
    }
}

pub trait ColoredOutput {
    fn to_output(&self, color: Color) -> Output;
}

impl<T: ToString> ColoredOutput for T {
    fn to_output(&self, color: Color) -> Output {
        Output::new(self.to_string(), color)
    }
}
