use super::printer::{Printer, PrinterItem, PrinterItemType};
use crossterm::style::Color;
pub mod theme;

const PAREN_COLORS: [&str; 4] = ["green", "red", "yellow", "blue"];
pub fn highlight(c: String, theme: &theme::Theme) -> Printer {
    let mut printer = Printer::default();

    for token in parse(c) {
        use Token::*;
        let (string, color) = match token {
            Keyword(s) => (s, &theme.keyword[..]),
            Keyword2(s) => (s, &theme.keyword2[..]),
            Function(s) => (s, &theme.function[..]),
            Type(s) => (s, &theme.r#type[..]),
            Number(s) => (s, &theme.number[..]),
            Symbol(c) => (c.to_string(), &theme.symbol[..]),
            Macro(s) => (s, &theme.r#macro[..]),
            StringLiteral(s) => (s, &theme.string_literal[..]),
            Character(c) => (c.to_string(), &theme.character[..]),
            LifeTime(s) => (s, &theme.lifetime[..]),
            Comment(s) => (s, &theme.comment[..]),
            Const(s) => (s, &theme.r#const[..]),
            X(s) => (s, &theme.x[..]),
            Token::LeftParen(s, idx) => (s.to_string(), PAREN_COLORS[idx.abs() as usize % 4]),
            Token::RightParen(s, idx) => (s.to_string(), PAREN_COLORS[idx.abs() as usize % 4]),
        };

        let color = theme::theme_color_to_term_color(color).unwrap_or(Color::White);
        printer.push(PrinterItem::new(string, PrinterItemType::Custom(color)));
    }
    printer
}

#[derive(Debug)]
enum Token {
    Keyword(String),
    Keyword2(String),
    Function(String),
    Type(String),
    Number(String),
    Macro(String),
    Symbol(char),
    StringLiteral(String),
    Character(char),
    LifeTime(String),
    Comment(String),
    Const(String),
    X(String),
    RightParen(char, isize),
    LeftParen(char, isize),
}

impl Token {
    fn _is_x(&self) -> bool {
        matches!(self, Token::X(_))
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
    let mut paren_idx = 0;
    while let Some(c) = s.next() {
        match c {
            // _ is accepted as variable/function name
            c if c.is_alphanumeric() || c == '_' => {
                alphanumeric.push(c);
            }
            '(' => {
                // maybe function
                if !alphanumeric.is_empty() {
                    tokens.push(Token::Function(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::LeftParen('(', paren_idx));
                paren_idx += 1;
            }
            '<' | '>' => {
                // maybe type <u8>
                if !alphanumeric.is_empty() {
                    tokens.push(Token::Type(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::Symbol(c));
            }
            '!' => {
                //maybe macro hello!
                if !alphanumeric.is_empty() {
                    tokens.push(Token::Macro(alphanumeric.drain(..).collect()));
                }
                tokens.push(Token::Symbol(c));
            }
            '\'' => {
                // maybe character || maybe lifetime
                if !alphanumeric.is_empty() {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                }
                // ' is considered Token::Character in both cases
                tokens.push(Token::Character(c));
                if s.peek().is_some() {
                    tokens.extend(parse_character_lifetime(&mut s));
                }
            }
            '"' => {
                // maybe literal
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
                // maybe const || maybe function with type annotation
                if s.peek() == Some(&':') {
                    // ::
                    // example: collect::<Vec<_>>()
                    s.next();
                } else {
                    // maybe const
                    // let HELLO: usize =
                    let token = parse_as(alphanumeric.drain(..).collect(), vec![TokenName::Const]);
                    tokens.push(token);
                    tokens.push(Token::Symbol(':'));
                    continue;
                }
                // maybe function with type annotation
                if s.peek() == Some(&'<') {
                    tokens.push(Token::Function(alphanumeric.drain(..).collect()));
                } else {
                    tokens.push(Token::X(alphanumeric.drain(..).collect()));
                }
                tokens.extend(vec![Token::Symbol(':'), Token::Symbol(':')]);
            }
            '/' => {
                // maybe division || maybe comment
                if !alphanumeric.is_empty() {
                    let token = parse_as(alphanumeric.drain(..).collect(), vec![TokenName::Number]);
                    tokens.push(token);
                }
                if s.peek() == Some(&'/') || s.peek() == Some(&'*') {
                    let end = if matches!(s.peek(), Some(&'/')) {
                        '\n'
                    } else {
                        '*'
                    };

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
                catch_all(&mut alphanumeric, &mut tokens);

                if x == ')' {
                    paren_idx -= 1;
                    tokens.push(Token::RightParen(')', paren_idx));
                } else if SYMBOLS.contains(&x) {
                    tokens.push(Token::Symbol(x));
                } else {
                    tokens.push(Token::X(x.to_string()));
                }
            }
        }
        previous_char = Some(c);
    }

    // leftover
    if !alphanumeric.is_empty() {
        catch_all(&mut alphanumeric, &mut tokens);
    }

    tokens
}

fn catch_all(alphanumeric: &mut String, tokens: &mut Vec<Token>) {
    // catch all: parse the alphanumeric buffer
    if !alphanumeric.is_empty() {
        let token = parse_as(
            alphanumeric.drain(..).collect(),
            vec![
                TokenName::Const,
                TokenName::Keyword,
                TokenName::Keyword2,
                TokenName::Number,
                TokenName::Type,
            ],
        );
        tokens.push(token);
    }
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
            // safe unwrap
            let end = characters.pop().unwrap();
            return vec![Token::LifeTime(characters), Token::Symbol(end)];
        }
    }

    while let Some(c) = s.peek() {
        if !c.is_alphabetic() {
            break;
        }
        // safe unwrap
        let c = s.next().unwrap();
        characters.push(c);
    }
    vec![Token::LifeTime(characters)]
}

fn parse_string_literal(s: &mut impl Iterator<Item = char>) -> Vec<Token> {
    let mut previous_char = None;
    let mut string_literal = String::new();
    for c in s {
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
    Keyword2,
    Type,
    Number,
    Const,
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
            TokenName::Keyword2 => {
                if is_keyword2(&p) {
                    return Token::Keyword2(p.unparsed_string());
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
            TokenName::Const => {
                if is_const(&p) {
                    return Token::Const(p.unparsed_string());
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

fn is_keyword2(p: &Token) -> bool {
    KEYWORDS2.contains(&p.unparsed_str())
}

fn is_type(p: &Token) -> bool {
    TYPES.contains(&p.unparsed_str())
}

fn is_const(p: &Token) -> bool {
    p.unparsed_str()
        .chars()
        .all(|c| c.is_uppercase() || c == '_')
}

// Splitting keywords for a nicer coloring
//      red blue  green     blue red white
// exp: pub fn    hello()   let  mut var
const KEYWORDS: &[&str] = &[
    "async", "while", "use", "super", "self", "Self", "for", "impl", "trait", "type", "pub", "in",
    "const", "static", "match", "use", "mut", "continue", "loop", "break", "if", "else",
];
const KEYWORDS2: &[&str] = &["fn", "let", "struct", "enum", "dyn"];

const SYMBOLS: &[char] = &[':', '&', '?', '+', '-', '*', '/', '=', '!', ',', ';'];
const TYPES: &[&str] = &[
    "bool", "char", "usize", "isize", "u8", "i8", "u32", "i32", "u64", "i64", "u128", "i128",
    "str", "String",
];
