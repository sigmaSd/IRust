pub mod theme;

use printer::{buffer::Buffer, printer::PrintQueue};
use theme::Theme;

#[derive(Debug)]
pub struct Highlight {
    pub engine: String,
    pub theme: String,
}

impl Highlight {
    pub fn new(engine: &str, theme: &str) -> Self {
        Self {
            engine: engine.into(),
            theme: theme.into(),
        }
    }
}

#[cfg(feature = "rustc_lexer")]
mod rustc_lexer_imp;

#[cfg(feature = "rustc_lexer")]
#[cfg(not(feature = "change_highlight"))]
impl Highlight {
    pub fn highlight(&self, buffer: &Buffer, theme: &Theme) -> PrintQueue {
        rustc_lexer_imp::highlight(buffer, theme)
    }
}

#[cfg(feature = "syntect")]
mod syntect_imp;

#[cfg(feature = "syntect")]
#[cfg(not(feature = "change_highlight"))]
impl Highlight {
    pub fn highlight(&self, buffer: &Buffer, _theme: &Theme) -> PrintQueue {
        let h = syntect_imp::get_highlighter(&self.theme);
        syntect_imp::highlight(&h, buffer)
    }
}

#[cfg(feature = "change_highlight")]
impl Highlight {
    pub fn highlight(&self, buffer: &Buffer, theme: &Theme) -> PrintQueue {
        match self.engine.as_str() {
            "syntect" => {
                let h = syntect_imp::get_highlighter(&self.theme);
                syntect_imp::highlight(h, buffer)
            }
            _ => rustc_lexer_imp::highlight(buffer, theme),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "nightly")]
    extern crate test;

    use super::*;

    #[cfg(feature = "nightly")]
    #[cfg(feature = "syntect")]
    #[bench]
    fn bench_syntect_highlight(b: &mut test::Bencher) {
        let h = Highlight::new("syntect", "default");
        let buffer = r#"\
        fn default() -> Self {
        crossterm::terminal::enable_raw_mode().expect("failed to enable raw_mode");
        let raw = Rc::new(RefCell::new(std::io::stdout()));
        Self {
            printer: Default::default(),
            writer: writer::Writer::new(raw.clone()),
            cursor: cursor::Cursor::new(raw),
        }
        "#
        .into();

        b.iter(|| h.highlight(&buffer, &Theme::default()));
    }

    #[cfg(feature = "nightly")]
    #[cfg(feature = "syntect")]
    #[bench]
    fn bench_rustc_lexer_highlight(b: &mut test::Bencher) {
        let h = Highlight::new("default", "default");
        let buffer = r#"\
        fn default() -> Self {
        crossterm::terminal::enable_raw_mode().expect("failed to enable raw_mode");
        let raw = Rc::new(RefCell::new(std::io::stdout()));
        Self {
            printer: Default::default(),
            writer: writer::Writer::new(raw.clone()),
            cursor: cursor::Cursor::new(raw),
        }
        "#
        .into();

        b.iter(|| h.highlight(&buffer, &Theme::default()));
    }
}
