use crossterm::style::Color;
use printer::buffer::Buffer;
use printer::printer::{PrintQueue, PrinterItem};
use theme::Theme;
pub mod theme;

const PAREN_COLORS: [&str; 4] = ["green", "red", "yellow", "blue"];
pub fn highlight(buffer: &Buffer, theme: &Theme) -> PrintQueue {
    let mut print_queue = PrintQueue::default();

    macro_rules! push_to_printer {
        ($item_type: ident, $item: expr, $color: expr) => {{
            let color = theme::theme_color_to_term_color($color).unwrap_or(Color::White);
            print_queue.push(PrinterItem::$item_type($item, color));
        }};
    }

    for token in parse(&buffer.buffer) {
        use Token::*;
        match token {
            Keyword(s) => push_to_printer!(String, s, &theme.keyword[..]),
            Keyword2(s) => push_to_printer!(String, s, &theme.keyword2[..]),
            Function(s) => push_to_printer!(String, s, &theme.function[..]),
            Type(s) => push_to_printer!(String, s, &theme.r#type[..]),
            Number(s) => push_to_printer!(String, s, &theme.number[..]),
            Symbol(c) => push_to_printer!(Char, c, &theme.symbol[..]),
            Macro(s) => push_to_printer!(String, s, &theme.r#macro[..]),
            StringLiteral(s) => push_to_printer!(String, s, &theme.string_literal[..]),
            StringLiteralC(c) => push_to_printer!(Char, c, &theme.string_literal[..]),
            Character(c) => push_to_printer!(Char, c, &theme.character[..]),
            LifeTime(s) => push_to_printer!(String, s, &theme.lifetime[..]),
            Comment(s) => push_to_printer!(String, s, &theme.comment[..]),
            CommentS(s) => push_to_printer!(Str, s, &theme.comment[..]),
            CommentC(c) => push_to_printer!(Char, c, &theme.comment[..]),
            Const(s) => push_to_printer!(String, s, &theme.r#const[..]),
            X(s) => push_to_printer!(String, s, &theme.x[..]),
            Xc(c) => push_to_printer!(Char, c, &theme.x[..]),
            Token::LeftParen(s, idx) => {
                push_to_printer!(Char, s, PAREN_COLORS[idx.abs() as usize % 4])
            }
            Token::RightParen(s, idx) => {
                push_to_printer!(Char, s, PAREN_COLORS[idx.abs() as usize % 4])
            }
            Token::NewLine => {
                print_queue.push(PrinterItem::NewLine);
            }
        };
    }
    print_queue
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
    StringLiteralC(char),
    Character(char),
    LifeTime(String),
    Comment(String),
    CommentC(char),
    CommentS(&'static str),
    Const(String),
    X(String),
    Xc(char),
    RightParen(char, isize),
    LeftParen(char, isize),
    NewLine,
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

fn parse(s: &[char]) -> Vec<Token> {
    let mut s = s.iter().peekable();
    let mut alphanumeric = String::new();
    let mut tokens = vec![];
    let mut previous_char = None;
    let mut paren_idx = 0;
    while let Some(c) = s.next() {
        let c = *c;
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
                    tokens.push(Token::StringLiteralC(c));
                } else {
                    tokens.push(Token::StringLiteralC('"'));
                    if s.peek().is_some() {
                        tokens.extend(parse_string_literal(&mut s));
                    }
                }
            }
            ':' => {
                // maybe const || maybe function with type annotation
                if s.peek() == Some(&&':') {
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
                if s.peek() == Some(&&'<') {
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
                if s.peek() == Some(&&'/') || s.peek() == Some(&&'*') {
                    let end = if matches!(s.peek(), Some(&'/')) {
                        '\n'
                    } else {
                        '*'
                    };

                    tokens.push(Token::CommentC('/'));
                    let mut comment = String::new();
                    while let Some(c) = s.next() {
                        if c == &end && end == '\n' {
                            tokens.push(Token::Comment(comment.drain(..).collect()));
                            tokens.push(Token::NewLine);
                            break;
                        } else if c == &end && s.peek() == Some(&&'/') {
                            // consume /
                            s.next();
                            tokens.push(Token::Comment(comment.drain(..).collect()));
                            tokens.push(Token::CommentS("*/"));
                            break;
                        } else {
                            comment.push(*c);
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
                } else if x == '\n' {
                    tokens.push(Token::NewLine);
                } else if SYMBOLS.contains(&x) {
                    tokens.push(Token::Symbol(x));
                } else {
                    tokens.push(Token::Xc(x));
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

fn parse_character_lifetime<'a>(
    s: &mut std::iter::Peekable<impl Iterator<Item = &'a char>>,
) -> Vec<Token> {
    // a'  b' c' d' \r' \t'
    // try as char

    let mut counter = 0;
    let mut previous_char = None;
    let mut characters = String::new();
    while let Some(c) = s.next() {
        if c == &'\'' && previous_char != Some('\\') {
            // we reached the end
            characters.push('\'');
            return characters.chars().map(Token::Character).collect();
        } else {
            characters.push(*c);
            if counter == 2 || (counter == 1 && !characters.starts_with('\\')) {
                // this is not a character
                break;
            }
        }
        previous_char = Some(*c);
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
        characters.push(*c);
    }
    vec![Token::LifeTime(characters)]
}

fn parse_string_literal<'a>(s: &mut impl Iterator<Item = &'a char>) -> Vec<Token> {
    let mut previous_char = None;
    let mut string_literal = String::new();
    for c in s {
        if c == &'"' && previous_char != Some('\\') {
            // we reached the end
            return vec![
                Token::StringLiteral(string_literal),
                Token::StringLiteralC('"'),
            ];
        } else {
            string_literal.push(*c);
        }

        previous_char = Some(*c);
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
    "async", "await", "while", "use", "super", "self", "Self", "for", "impl", "trait", "type",
    "pub", "in", "const", "static", "match", "use", "mut", "continue", "loop", "break", "if",
    "else",
];
const KEYWORDS2: &[&str] = &["unsafe", "move", "fn", "let", "struct", "enum", "dyn"];

const SYMBOLS: &[char] = &[':', '&', '?', '+', '-', '*', '/', '=', '!', ',', ';'];
const TYPES: &[&str] = &[
    "bool", "char", "usize", "isize", "u8", "i8", "u32", "i32", "u64", "i64", "u128", "i128",
    "str", "String",
];
