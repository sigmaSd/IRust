use once_cell::sync::Lazy;
use printer::{
    buffer::Buffer,
    printer::{PrintQueue, PrinterItem},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};
use theme::Theme;

pub mod theme;

// Load these once at the start of your program
static PS: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static SYNTAX: Lazy<&SyntaxReference> = Lazy::new(|| PS.find_syntax_by_extension("rs").unwrap());

fn syntect_color_to_crossterm(color: syntect::highlighting::Color) -> crossterm::style::Color {
    crossterm::style::Color::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

pub fn highlight(buffer: &Buffer, theme: &Theme) -> PrintQueue {
    let mut h = HighlightLines::new(&SYNTAX, &TS.themes["base16-ocean.dark"]);
    let mut print_queue = PrintQueue::default();

    let buf = buffer.to_string();
    for line in LinesWithEndings::from(&buf) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &PS).unwrap();
        for (style, text) in ranges {
            print_queue.push(PrinterItem::String(
                text.to_string(),
                syntect_color_to_crossterm(style.foreground),
            ));
        }
    }

    print_queue
}
