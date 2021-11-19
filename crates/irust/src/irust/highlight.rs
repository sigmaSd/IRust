use crossterm::style::Color;
use printer::buffer::Buffer;
use printer::printer::{PrintQueue, PrinterItem};
use theme::Theme;
pub mod theme;

const PAREN_COLORS: [&str; 4] = ["red", "yellow", "green", "blue"];

pub fn highlight(buffer: &Buffer, theme: &Theme) -> PrintQueue {
    let mut print_queue = PrintQueue::default();

    let buffer = buffer.to_string();
    let rc_buf = std::rc::Rc::new(buffer.clone());

    let mut token_range = 0..0;
    let tokens: Vec<_> = rustc_lexer::tokenize(&buffer).collect();
    let mut paren_idx = 0_isize;

    macro_rules! push_to_printer {
        ($color: expr) => {{
            let color = theme::theme_color_to_term_color($color).unwrap_or(Color::White);
            print_queue.push(PrinterItem::RcString(
                rc_buf.clone(),
                token_range.clone(),
                color,
            ));
        }};
    }

    for (idx, token) in tokens.iter().enumerate() {
        token_range.start = token_range.end;
        token_range.end += token.len;
        let text = &buffer[token_range.clone()];

        use rustc_lexer::TokenKind::*;
        match token.kind {
            Ident if KEYWORDS.contains(&text) => {
                push_to_printer!(&theme.keyword[..]);
            }
            Ident if KEYWORDS2.contains(&text) => {
                push_to_printer!(&theme.keyword2[..]);
            }
            Ident if TYPES.contains(&text) => {
                push_to_printer!(&theme.r#type[..]);
            }
            // const
            Ident if text.chars().all(char::is_uppercase) => {
                push_to_printer!(&theme.r#const[..]);
            }
            // macro
            Ident
                if matches!(
                    peek_first_non_white_sapce(&tokens[idx + 1..]).map(|(_, k)| k),
                    Some(Bang)
                ) =>
            {
                push_to_printer!(&theme.r#macro[..]);
            }
            // function
            Ident if is_function(&tokens[idx + 1..]) => {
                push_to_printer!(&theme.function[..]);
            }
            UnknownPrefix | Unknown | Ident | RawIdent | Whitespace => {
                push_to_printer!(&theme.ident[..]);
            }
            LineComment { .. } | BlockComment { .. } => {
                push_to_printer!(&theme.comment[..]);
            }
            Literal { .. } => {
                push_to_printer!(&theme.literal[..])
            }
            Lifetime { .. } => {
                push_to_printer!(&theme.lifetime[..])
            }
            Colon | At | Pound | Tilde | Question | Dollar | Semi | Comma | Dot | Eq | Bang
            | Lt | Gt | Minus | And | Or | Plus | Star | Slash | Caret | Percent | OpenBrace
            | OpenBracket | CloseBrace | CloseBracket => {
                push_to_printer!(&theme.symbol[..]);
            }
            OpenParen => {
                push_to_printer!(PAREN_COLORS[paren_idx.abs() as usize % 4]);
                paren_idx += 1;
            }
            CloseParen => {
                paren_idx -= 1;
                push_to_printer!(PAREN_COLORS[paren_idx.abs() as usize % 4]);
            }
        };
    }
    print_queue
}

fn peek_first_non_white_sapce(
    tokens: &[rustc_lexer::Token],
) -> Option<(usize, rustc_lexer::TokenKind)> {
    for (idx, token) in tokens.iter().enumerate() {
        if token.kind != rustc_lexer::TokenKind::Whitespace {
            return Some((idx, token.kind));
        }
    }
    None
}

fn is_function(tokens: &[rustc_lexer::Token]) -> bool {
    let (idx, kind) = match peek_first_non_white_sapce(tokens) {
        Some((i, k)) => (i, k),
        None => return false,
    };
    use rustc_lexer::TokenKind::*;
    match kind {
        OpenParen => true,
        Lt => true,
        Colon => is_function(&tokens[idx + 1..]),
        _ => false,
    }
}

// Splitting keywords for a nicer coloring
//      red blue  green     blue red white
// exp: pub fn    hello()   let  mut var
const KEYWORDS: &[&str] = &[
    "async", "await", "while", "use", "super", "self", "Self", "for", "impl", "trait", "type",
    "pub", "in", "const", "static", "match", "use", "mut", "continue", "loop", "break", "if",
    "else", "macro",
];
const KEYWORDS2: &[&str] = &["unsafe", "move", "fn", "let", "struct", "enum", "dyn"];

const TYPES: &[&str] = &[
    "bool", "char", "usize", "isize", "u8", "i8", "u32", "i32", "u64", "i64", "u128", "i128",
    "str", "String",
];
