use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::irust::Result;

pub fn split_args(s: String) -> Vec<String> {
    let mut args = vec![];
    let mut tmp = String::new();
    let mut quote = false;

    for c in s.chars() {
        match c {
            ' ' => {
                if !quote && !tmp.is_empty() {
                    args.push(tmp.drain(..).collect());
                } else {
                    tmp.push(' ');
                }
            }
            '"' => {
                quote = !quote;
            }
            _ => tmp.push(c),
        }
    }
    if !tmp.is_empty() {
        args.push(tmp);
    }
    args
}

#[test]
fn split_args_test() {
    let cmd = r#":add crate --no-default --features "a b c""#.to_string();
    assert_eq!(
        vec![":add", "crate", "--no-default", "--features", "a b c",]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>(),
        split_args(cmd)
    );
}

pub fn stdout_and_stderr(out: std::process::Output) -> String {
    let out = if !out.stdout.is_empty() {
        out.stdout
    } else {
        out.stderr
    };

    String::from_utf8_lossy(&out).to_string()
}

fn _remove_main(script: &str) -> String {
    const MAIN_FN: &str = "fn main() {";

    let mut script = remove_comments(script);

    let main_start = match script.find(MAIN_FN) {
        Some(idx) if _balanced_quotes(&script[..idx]) => idx,
        _ => return script,
    };

    let open_tag = main_start + MAIN_FN.len();

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
                break;
            }
        }
    }
    if let Some(close_tag) = close_tag {
        script.remove(open_tag + close_tag + 1);
        script.replace_range(main_start..=open_tag, "");
    }
    script
}

pub struct StringTools {}

impl StringTools {
    pub fn _insert_at_char_idx(buffer: &mut String, idx: usize, character: char) {
        let mut buffer_chars: Vec<char> = buffer.chars().collect();
        buffer_chars.insert(idx, character);
        *buffer = buffer_chars.into_iter().collect();
    }

    pub fn _remove_at_char_idx(buffer: &mut String, idx: usize) -> Option<char> {
        let mut buffer_chars: Vec<char> = buffer.chars().collect();

        let removed_char = if buffer_chars.len() > idx {
            Some(buffer_chars.remove(idx))
        } else {
            None
        };

        *buffer = buffer_chars.into_iter().collect();

        removed_char
    }

    pub fn chars_count(buffer: &str) -> usize {
        buffer.chars().count()
    }

    pub fn new_lines_count(buffer: &str) -> usize {
        buffer.chars().filter(|c| c == &'\n').count()
    }

    pub fn _is_multiline(string: &str) -> bool {
        string.chars().filter(|c| *c == '\n').count() > 1
    }

    pub fn strings_unique(s1: &str, s2: &mut String) {
        let mut idx = s2.len();
        loop {
            if s2.get(..idx).is_some() && !s2[..idx].is_empty() && s1.ends_with(&s2[..idx]) {
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

    pub fn unmatched_brackets(s: &str) -> bool {
        let s = remove_comments(s);
        let mut braces = std::collections::HashMap::new();
        braces.insert('(', 0);
        braces.insert('[', 0);
        braces.insert('{', 0);

        let mut quote = false;
        let mut double_quote = false;
        let mut previous_char = ' ';
        for character in s.chars() {
            // safe unwraps ahead
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

pub fn read_until_bytes<R: std::io::BufRead + ?Sized>(
    r: &mut R,
    delim: &[u8],
    buffer: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let mut nn = 0;
    let mut b = [0; 512];
    loop {
        let n = r.read(&mut b)?;
        buffer.extend(&b[..n]);
        nn += n;
        if n == 0 || buffer.ends_with(delim) {
            break Ok(nn);
        }
    }
}

fn remove_comments(s: &str) -> String {
    s.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| {
            let mut quote = false;
            let mut d_quote = false;

            let mut l = l.chars().peekable();
            let mut purged_line = String::new();

            loop {
                match (l.next(), l.peek()) {
                    (Some('/'), Some('/')) => {
                        if !quote && !d_quote {
                            break;
                        }
                    }
                    (Some('\''), _) => {
                        quote = !quote;
                        purged_line.push('\'');
                    }
                    (Some('"'), _) => {
                        d_quote = !d_quote;
                        purged_line.push('"');
                    }
                    (Some(c), _) => purged_line.push(c),
                    _ => break,
                }
            }
            purged_line + "\n"
        })
        .collect()
}

fn _balanced_quotes(s: &str) -> bool {
    s.match_indices(|p| p == '"' || p == '\'').count() % 2 == 0
}

pub fn ctrlc_cancel(process: &mut std::process::Child) -> Result<()> {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
    // Running a command as Command::new().output takes at minimum 1ms
    // So Polling should take a similar order of magnitude
    if let Ok(event) = crossterm::event::poll(std::time::Duration::from_millis(1)) {
        if event {
            if let Ok(event) = crossterm::event::read() {
                match event {
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Release,
                        ..
                    }) => (),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: crossterm::event::KeyModifiers::CONTROL,
                        ..
                    }) => {
                        process.kill()?;
                        return Err("Cancelled!".into());
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(a),
                        ..
                    }) => {
                        use std::io::Write;
                        // Ignore write errors (process might have ended)
                        let _ = process.stdin.as_mut().unwrap().write_all(&[a as u8]);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => {
                        use std::io::Write;
                        // Ignore write errors (process might have ended)
                        let _ = process.stdin.as_mut().unwrap().write_all(&[b'\n']);
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(())
}

pub fn find_workpace_root(metadata: String) -> Option<String> {
    let start = metadata.find("workspace_root")? + 17;
    let end = metadata[start..].find('"')?;
    Some(metadata[start..start + end].to_string())
}

pub fn patch_name_to(path: &Path, name: &str) -> Result<()> {
    let toml = std::fs::read_to_string(path)?;
    let mut patched: String = String::new();
    for line in toml.lines() {
        if line.starts_with("name =") {
            patched.push_str(&format!("name = \"{name}\""));
            patched.push('\n');
        } else {
            patched.push_str(line);
            patched.push('\n');
        }
    }
    std::fs::write(path, patched)?;
    Ok(())
}

pub fn copy_dir(src_path: &Path, out_path: &Path) -> Result<()> {
    if src_path.is_file() {
        panic!("Incorrect usage")
    }
    let convert_path =
        |path: &Path| -> Result<PathBuf> { Ok(out_path.join(path.strip_prefix(src_path)?)) };
    let dcb = |dp: PathBuf| Ok(std::fs::create_dir(convert_path(&dp)?)?);
    let fcb = |fp: PathBuf| Ok(std::fs::copy(fp.clone(), convert_path(&fp)?)?);
    visit_dirs(src_path, &dcb, &fcb)
}

fn visit_dirs(
    dir: &Path,
    dcb: &dyn Fn(PathBuf) -> Result<()>,
    fcb: &dyn Fn(PathBuf) -> Result<u64>,
) -> Result<()> {
    if dir.is_dir() {
        dcb(dir.to_path_buf())?;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, dcb, fcb)?;
            } else {
                fcb(entry.path())?;
            }
        }
    }
    Ok(())
}
