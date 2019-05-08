// associated fns
pub fn remove_main(script: &mut String) {
    let main_start = match script.find("fn main() {") {
        Some(idx) => idx,
        None => return,
    };

    let open_tag = main_start + 11;
    // script == fn main() {
    if script.len() == 11 {
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
