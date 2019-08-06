use super::printer::{Printer, PrinterItem, PrinterItemType};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static PS: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static SYNTAX: Lazy<&SyntaxReference> = Lazy::new(|| PS.find_syntax_by_extension("rs").unwrap());

pub fn highlight(c: &str) -> Printer {
    let mut h = HighlightLines::new(&SYNTAX, &TS.themes["base16-ocean.dark"]);

    let mut printer = Printer::default();

    for line in LinesWithEndings::from(c) {
        h.highlight(line, &PS)
            .into_iter()
            .for_each(|(style, part)| {
                let fg_color = match style.foreground {
                    Color { r, g, b, .. } => crossterm::Color::Rgb { r, g, b },
                };
                printer.push(PrinterItem::new(
                    part.to_string(),
                    PrinterItemType::Custom(fg_color),
                ));
            });
        printer.add_new_line(1);
    }

    printer
}
