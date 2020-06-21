use super::printer::{Printer, PrinterItem, PrinterItemType};
use crossterm::style::Color;

pub fn highlight(c: String) -> Printer {
    let mut printer = Printer::default();
    for token in parse(c) {
        use Token::*;
        let (string, color) = match token {
            Keyword(s) => (s, Color::Magenta),
            Function(s) => (s, Color::Blue),
            Type(s) => (s, Color::DarkGreen),
            Number(s) => (s, Color::Cyan),
            Symbol(s) => (s.to_string(), Color::Red),
            Macro(s) => (s, Color::Yellow),
            StringLiteral(s) => (s, Color::Green),
            X(s) => (s, Color::White),
            NewLine => {
                printer.add_new_line(1);
                continue;
            }
        };

        printer.push(PrinterItem::new(string, PrinterItemType::Custom(color)));
    }
    printer
}

#[derive(Debug)]
enum Token {
    Keyword(String),
    Function(String),
    Type(String),
    Number(String),
    Macro(String),
    Symbol(char),
    StringLiteral(String),
    NewLine,
    X(String),
}

impl Token {
    fn _is_x(&self) -> bool {
        if let Token::X(_) = self {
            true
        } else {
            false
        }
    }
    fn unparsed_string(self) -> String {
        match self {
            Token::X(s) => s,
            _ => unreachable!(),
        }
    }
    fn unparsed_str(&self) -> &str {
        match self {
            Token::X(s) => s,
            _ => unreachable!(),
        }
    }
}

fn parse(s: String) -> Vec<Token> {
    let mut s = s.chars().peekable();
    let mut alphanumeric = String::new();
    let mut tokens = vec![];
    let mut previous_char = None;
    while let Some(c) = s.next() {
        match c {
            // _ is accepted as variable/function name
            c if c.is_alphanumeric() || c == '_' => {
                alphanumeric.push(c);
            }
            '(' => {
                if !alphanumeric.is_empty() {
                    tokens.push(Token::Function(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::X('('.to_string()));
            }
            ' ' => {
                if !alphanumeric.is_empty() {
                    let token = parse_as(
                        alphanumeric.drain(..).collect(),
                        vec![TokenName::Keyword, TokenName::Type, TokenName::Number],
                    );
                    tokens.push(token);
                }
                tokens.push(Token::X(' '.to_string()));
            }
            '<' | '>' => {
                if !alphanumeric.is_empty() {
                    tokens.push(Token::Type(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::Symbol(c));
            }
            '!' => {
                if !alphanumeric.is_empty() {
                    tokens.push(Token::Macro(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::Symbol(c));
            }
            '"' => {
                if !alphanumeric.is_empty() {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                }
                if let Some('\\') = previous_char {
                    tokens.push(Token::Symbol(c));
                } else {
                    tokens.push(Token::StringLiteral('"'.to_string()));
                    tokens.extend(parse_string_literal(&mut s));
                }
            }
            ':' => {
                //collect::<Vec<_>>()
                if s.peek() == Some(&':') {
                    s.next();
                } else {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                    tokens.push(Token::Symbol(':'));
                    continue;
                }
                if s.peek() == Some(&'<') {
                    tokens.push(Token::Function(alphanumeric.drain(..).collect()));
                } else {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                }
                tokens.extend(vec![Token::Symbol(':'), Token::Symbol(':')]);
            }
            x => {
                if !alphanumeric.is_empty() {
                    let token = parse_as(
                        alphanumeric.drain(..).collect(),
                        vec![TokenName::Number, TokenName::Type],
                    );
                    tokens.push(token);
                }

                if SYMBOLS.contains(&x) {
                    tokens.push(Token::Symbol(x));
                } else if x == '\n' {
                    tokens.push(Token::NewLine)
                } else {
                    tokens.push(Token::X(x.to_string()));
                }
            }
        }
        previous_char = Some(c);
    }
    if !alphanumeric.is_empty() {
        let token = parse_as(
            alphanumeric.drain(..).collect(),
            vec![TokenName::Keyword, TokenName::Type, TokenName::Number],
        );
        tokens.push(token);
    }
    tokens
}

fn parse_string_literal(s: &mut impl Iterator<Item = char>) -> Vec<Token> {
    let mut previous_char = None;
    let mut string_literal = String::new();
    while let Some(c) = s.next() {
        if c == '"' && previous_char != Some('\\') {
            // we reached the end
            return vec![
                Token::StringLiteral(string_literal),
                Token::StringLiteral('"'.to_string()),
            ];
        } else {
            string_literal.push(c);
        }

        previous_char = Some(c);
    }

    vec![Token::StringLiteral(string_literal)]
}

enum TokenName {
    Keyword,
    Type,
    Number,
}

fn parse_as(p: String, token_names: Vec<TokenName>) -> Token {
    let p = Token::X(p);
    for token_name in token_names {
        match token_name {
            TokenName::Keyword => {
                if is_keyword(&p) {
                    return Token::Keyword(p.unparsed_string());
                }
            }
            TokenName::Type => {
                if is_type(&p) {
                    return Token::Type(p.unparsed_string());
                }
            }
            TokenName::Number => {
                if is_number(&p) {
                    return Token::Number(p.unparsed_string());
                }
            }
        }
    }
    p
}

fn is_number(p: &Token) -> bool {
    p.unparsed_str().chars().all(char::is_numeric)
}

fn is_keyword(p: &Token) -> bool {
    KEYWORDS.contains(&p.unparsed_str())
}

fn is_type(p: &Token) -> bool {
    TYPES.contains(&p.unparsed_str())
}

const KEYWORDS: &[&str] = &[
    "pub", "in", "const", "static", "match", "fn", "use", "let", "mut", "continue", "loop",
    "break", "if", "else",
];
const SYMBOLS: &[char] = &[':', '&', '?', '+', '-', '*', '/', '=', '!'];
const TYPES: &[&str] = &[
    "usize", "isize", "u8", "i8", "u32", "i32", "u64", "i64", "u128", "i128", "str", "String",
];
