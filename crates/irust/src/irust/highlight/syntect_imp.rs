use once_cell::sync::{Lazy, OnceCell};
use printer::{
    buffer::Buffer,
    printer::{PrintQueue, PrinterItem},
};
use syntect::{
    highlighting::{HighlightIterator, HighlightState, Highlighter, ThemeSet},
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

// Load these once at the start of your program
static PS: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static SYNTAX: Lazy<&SyntaxReference> = Lazy::new(|| PS.find_syntax_by_name("Rust").unwrap());
static HIGHLIGHTER: OnceCell<Highlighter> = OnceCell::new();

pub(super) fn get_highlighter(theme: &str) -> &Highlighter {
    HIGHLIGHTER.get_or_init(|| {
        if TS.themes.contains_key(theme) {
            Highlighter::new(&TS.themes[theme])
        } else {
            Highlighter::new(&TS.themes["base16-ocean.dark"])
        }
    })
}

pub fn highlight(h: &Highlighter, buffer: &Buffer) -> PrintQueue {
    let buf = buffer.to_string();
    let rc_buf = std::rc::Rc::new(buf.clone());
    let mut range = 0..0;
    let mut print_queue = PrintQueue::default();
    macro_rules! push_to_printer {
        ($style: expr) => {{
            let color = syntect_color_to_crossterm($style.foreground);
            print_queue.push(PrinterItem::RcString(rc_buf.clone(), range.clone(), color));
        }};
    }

    let mut highlight_state = HighlightState::new(h, ScopeStack::new());
    let mut parse_state = ParseState::new(&SYNTAX);
    for line in LinesWithEndings::from(&buf) {
        let ops = parse_state.parse_line(line, &PS).unwrap();
        let iter = HighlightIterator::new(&mut highlight_state, &ops[..], line, h);
        for (style, text) in iter {
            range.start = range.end;
            range.end += text.len();
            push_to_printer!(style);
        }
    }

    print_queue
}

#[inline]
fn syntect_color_to_crossterm(color: syntect::highlighting::Color) -> crossterm::style::Color {
    crossterm::style::Color::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}
