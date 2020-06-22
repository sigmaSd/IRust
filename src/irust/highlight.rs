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
            Symbol(c) => (c.to_string(), Color::Red),
            Macro(s) => (s, Color::Yellow),
            StringLiteral(s) => (s, Color::Green),
            Character(c) => (c.to_string(), Color::Green),
            LifeTime(s) => (s, Color::DarkMagenta),
            Comment(s) => (s, Color::DarkGrey),
            X(s) => (s, Color::White),
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
    Character(char),
    LifeTime(String),
    Comment(String),
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
            '\'' => {
                if !alphanumeric.is_empty() {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::Character(c));
                if s.peek().is_some() {
                    tokens.extend(parse_character_lifetime(&mut s));
                }
            }
            '"' => {
                if !alphanumeric.is_empty() {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                }
                if let Some('\\') = previous_char {
                    tokens.push(Token::StringLiteral(c.to_string()));
                } else {
                    tokens.push(Token::StringLiteral('"'.to_string()));
                    if s.peek().is_some() {
                        tokens.extend(parse_string_literal(&mut s));
                    }
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
            '/' => {
                if !alphanumeric.is_empty() {
                    let token = parse_as(alphanumeric.drain(..).collect(), vec![TokenName::Number]);
                    tokens.push(token);
                }
                if s.peek() == Some(&'/') || s.peek() == Some(&'*') {
                    let end = if s.peek().unwrap() == &'/' { '\n' } else { '*' };

                    tokens.push(Token::Comment('/'.to_string()));
                    let mut comment = String::new();
                    while let Some(c) = s.next() {
                        if c == end && end == '\n' {
                            tokens.push(Token::Comment(comment.drain(..).collect()));
                            tokens.push(Token::X('\n'.to_string()));
                            break;
                        } else if c == end && s.peek() == Some(&'/') {
                            // consume /
                            s.next();
                            tokens.push(Token::Comment(comment.drain(..).collect()));
                            tokens.push(Token::Comment("*/".to_string()));
                            break;
                        } else {
                            comment.push(c);
                        }
                    }
                    if !comment.is_empty() {
                        tokens.push(Token::Comment(comment));
                    }
                } else {
                    tokens.push(Token::Symbol('/'));
                }
            }
            x => {
                if !alphanumeric.is_empty() {
                    let token = parse_as(
                        alphanumeric.drain(..).collect(),
                        vec![TokenName::Keyword, TokenName::Number, TokenName::Type],
                    );
                    tokens.push(token);
                }

                if SYMBOLS.contains(&x) {
                    tokens.push(Token::Symbol(x));
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

fn parse_character_lifetime(s: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> Vec<Token> {
    // a'  b' c' d' \r' \t'
    // try as char

    let mut counter = 0;
    let mut previous_char = None;
    let mut characters = String::new();
    while let Some(c) = s.next() {
        if c == '\'' && previous_char != Some('\\') {
            // we reached the end
            characters.push('\'');
            return characters.chars().map(Token::Character).collect();
        } else {
            characters.push(c);
            if counter == 2 || (counter == 1 && !characters.starts_with('\\')) {
                // this is not a character
                break;
            }
        }
        previous_char = Some(c);
        counter += 1;
    }

    // try as lifetime

    if let Some(c) = characters.chars().last() {
        if !c.is_alphabetic() {
            let end = characters.pop().unwrap();
            return vec![Token::LifeTime(characters), Token::Symbol(end)];
        }
    }

    while let Some(c) = s.peek() {
        if !c.is_alphabetic() {
            break;
        }
        let c = s.next().unwrap();
        characters.push(c);
    }
    vec![Token::LifeTime(characters)]
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
    "self", "Self", "for", "struct", "enum", "impl", "trait", "type", "pub", "in", "const",
    "static", "match", "fn", "use", "let", "mut", "continue", "loop", "break", "if", "else",
];
const SYMBOLS: &[char] = &[':', '&', '?', '+', '-', '*', '/', '=', '!', ',', ';'];
const TYPES: &[&str] = &[
    "bool", "char", "usize", "isize", "u8", "i8", "u32", "i32", "u64", "i64", "u128", "i128",
    "str", "String",
];
