pub fn stdout_and_stderr(out: std::process::Output) -> String {
    let out = if !out.stdout.is_empty() {
        out.stdout
    } else {
        out.stderr
    };

    String::from_utf8(out).unwrap_or_default()
}

pub fn remove_main(script: &mut String) {
    let main_start = match script.find("fn main() {") {
        Some(idx) => idx,
        None => return,
    };

    let open_tag = main_start + 11;

    // corrupted script `..fn main() {`
    if open_tag == script.len() - 1 {
        return;
    }

    let mut close_tag = None;

    // look for closing tag
    let mut tag_score = 1;
    for (idx, character) in script[open_tag + 1..].chars().enumerate() {
        if character == '{' {
            tag_score += 1;
        }
        if character == '}' {
            tag_score -= 1;
            if tag_score == 0 {
                close_tag = Some(idx);
            }
        }
    }
    if let Some(close_tag) = close_tag {
        script.remove(open_tag + close_tag + 1);
        script.replace_range(main_start..=open_tag, "");
    }
}

pub struct StringTools {}

impl StringTools {
    pub fn insert_at_char_idx(buffer: &mut String, idx: usize, character: char) {
        let mut buffer_chars: Vec<char> = buffer.chars().collect();
        buffer_chars.insert(idx, character);
        *buffer = buffer_chars.into_iter().collect();
    }

    pub fn remove_at_char_idx(buffer: &mut String, idx: usize) {
        let mut buffer_chars: Vec<char> = buffer.chars().collect();
        buffer_chars.remove(idx);
        *buffer = buffer_chars.into_iter().collect();
    }

    pub fn chars_count(buffer: &str) -> usize {
        buffer.chars().count()
    }

    pub fn is_multiline(string: &str) -> bool {
        string.chars().filter(|c| *c == '\n').count() > 1
    }

    pub fn strings_unique(s1: &str, s2: &mut String) {
        let mut idx = s2.len();
        loop {
            if !s2[..idx].is_empty() && s1.ends_with(&s2[..idx]) {
                for _ in 0..idx {
                    s2.remove(0);
                }
                break;
            }
            if idx == 0 {
                if let Some(last_char) = s1.chars().last() {
                    if last_char.is_alphanumeric() {
                        s2.clear();
                    }
                }
                break;
            }

            idx -= 1;
        }
    }

    pub fn nl_to_space(s: &str) -> String {
        s.replace("\n", " ")
    }

    pub fn unmatched_brackets(s: &str) -> bool {
        let mut braces = std::collections::HashMap::new();
        braces.insert('(', 0);
        braces.insert('[', 0);
        braces.insert('{', 0);

        let mut quote = false;
        let mut double_quote = false;
        let mut previous_char = ' ';
        for character in s.chars() {
            match character {
                '(' => {
                    if !quote && !double_quote {
                        *braces.get_mut(&'(').unwrap() += 1;
                    }
                }
                ')' => {
                    if !quote && !double_quote {
                        *braces.get_mut(&'(').unwrap() -= 1;
                    }
                }
                '[' => {
                    if !quote && !double_quote {
                        *braces.get_mut(&'[').unwrap() += 1;
                    }
                }
                ']' => {
                    if !quote && !double_quote {
                        *braces.get_mut(&'[').unwrap() -= 1;
                    }
                }
                '{' => {
                    if !quote && !double_quote {
                        *braces.get_mut(&'{').unwrap() += 1;
                    }
                }
                '}' => {
                    if !quote && !double_quote {
                        *braces.get_mut(&'{').unwrap() -= 1;
                    }
                }
                '"' => {
                    if previous_char != '\\' {
                        double_quote = !double_quote;
                    }
                }
                '\'' => {
                    if previous_char != '\\' {
                        quote = !quote;
                    }
                }
                _ => (),
            }
            previous_char = character;
        }

        braces[&'('] != 0 || braces[&'['] != 0 || braces[&'{'] != 0
    }
}

pub struct VecTools {}

impl VecTools {
    pub fn index(vector: &[String], item: &str) -> Vec<usize> {
        let mut indices = vec![];

        for (idx, elem) in vector.iter().enumerate() {
            if elem.starts_with(item) {
                indices.push(idx);
            }
        }

        indices
    }
}

pub fn read_until_bytes<R: std::io::BufRead + ?Sized>(
    r: &mut R,
    delim: &[u8],
    buf: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let mut read = 0;
    let mut count = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            match available.iter().position(|b| *b == delim[count]) {
                Some(i) => {
                    buf.extend_from_slice(&available[..=i]);

                    count += 1;
                    if count == delim.len() {
                        (true, i + 1)
                    } else {
                        (false, i + 1)
                    }
                }
                None => {
                    count = 0;
                    buf.extend_from_slice(available);
                    (false, available.len())
                }
            }
        };
        r.consume(used);
        read += used;
        if done || used == 0 {
            return Ok(read);
        }
    }
}
