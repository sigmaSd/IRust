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
