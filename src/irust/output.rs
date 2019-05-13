use crate::irust::IRust;
use crossterm::Color;

#[derive(Default)]
pub struct Output {
    strings_and_colors: Vec<(String, Color)>,
}

impl Output {
    pub fn new<P: Into<Option<Color>>>(string: String, color: P) -> Self {
        let color = color.into().unwrap_or(Color::White);

        Self {
            strings_and_colors: vec![(string, color)],
        }
    }

    pub fn push<P: Into<Option<Color>>>(&mut self, string: String, color: P) {
        let color = color.into().unwrap_or(Color::White);

        self.strings_and_colors.push((string, color));
    }

    pub fn clear(&mut self) {
        self.strings_and_colors.clear();
    }
}

impl IRust {
    pub fn write_out(&mut self) -> std::io::Result<()> {
        for (string, color) in self.output.strings_and_colors.clone() {
            self.color.set_fg(color);
            self.write(&string);
            self.color.reset();
        }
        Ok(())
    }
}
