use super::printer::{Printer, PrinterItem, PrinterItemType};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub fn highlight(c: &str) -> Printer {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut printer = Printer::default();

    for line in LinesWithEndings::from(c) {
        h.highlight(line, &ps)
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
