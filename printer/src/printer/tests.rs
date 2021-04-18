use super::default_process_fn;
use super::Printer;
use crossterm::style::Color;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn write_from_terminal_start_cursor_pos_correct() -> Result<()> {
    let mut p = Printer::new(std::io::sink());

    let origin_pos = p.cursor.pos;
    p.write_from_terminal_start("hello", Color::Red)?;
    assert_eq!(p.cursor.pos.current_pos.0, 5);
    assert_eq!(p.cursor.pos.current_pos.1, origin_pos.current_pos.1);

    Ok(())
}

#[test]
fn writenew_line_no_scroll() {
    let mut p = Printer::new(std::io::sink());

    let b = "Hello world".into();

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = 0;
    p.cursor.goto_start();
    assert_eq!(p.cursor.pos.current_pos, p.cursor.pos.starting_pos);

    let origin_pos = p.cursor.pos;
    p.write_newline(&b);

    assert_eq!(origin_pos.starting_pos.1 + 1, p.cursor.pos.starting_pos.1);
    assert_eq!(origin_pos.current_pos.1 + 1, p.cursor.pos.current_pos.1);
}

#[test]
fn writenew_line_with_scroll() {
    let mut p = Printer::new(std::io::sink());
    let b = "Hello world".into();

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = p.cursor.bound.height - 1;
    p.cursor.goto_start();

    assert_eq!(p.cursor.pos.current_pos, p.cursor.pos.starting_pos);

    let origin_pos = p.cursor.pos;
    p.write_newline(&b);

    assert_eq!(origin_pos.starting_pos.1, p.cursor.pos.starting_pos.1);
    assert_eq!(origin_pos.current_pos.1, p.cursor.pos.current_pos.1);
}

#[test]
fn scroll_up() -> Result<()> {
    let mut p = Printer::new(std::io::sink());

    let origin_pos = p.cursor.pos;
    p.scroll_up(3);

    assert_eq!(
        origin_pos.starting_pos.1.saturating_sub(3),
        p.cursor.pos.starting_pos.1
    );
    assert_eq!(
        origin_pos.current_pos.1.saturating_sub(3),
        p.cursor.pos.current_pos.1
    );

    Ok(())
}

#[test]
fn scroll_because_input_needs_scroll() -> Result<()> {
    let mut p = Printer::new(std::io::sink());
    let b = "\n\n\n".into();

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = p.cursor.bound.height - 1;
    p.cursor.goto_start();

    let original_pos = p.cursor.pos;
    p.scroll_if_needed_for_input(&b);

    assert_eq!(original_pos.starting_pos.1 - 3, p.cursor.pos.starting_pos.1);
    Ok(())
}

#[test]
fn dont_scroll_because_input_doesent_need_scroll() -> Result<()> {
    let mut p = Printer::new(std::io::sink());
    let b = "\n\n\n".into();

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = 0;
    p.cursor.goto_start();

    let original_pos = p.cursor.pos;
    p.scroll_if_needed_for_input(&b);

    assert_eq!(original_pos.starting_pos.1, p.cursor.pos.starting_pos.1);
    Ok(())
}

#[test]
fn calculate_bounds_correctly() -> Result<()> {
    const INPUT_START: usize = 4;
    let mut p = Printer::new(std::io::sink());
    let width = p.cursor.bound.width;
    let height = p.cursor.bound.height;
    let queue = default_process_fn(&"alloc\nprint".into());

    // 1
    move_to_and_modify_start(&mut p, 0, 0);
    p.recalculate_bounds(queue.clone())?;

    let expected_bound = {
        let mut v = vec![width - 1; height];
        v[0] = INPUT_START + 5;
        v[1] = INPUT_START + 5;
        v
    };
    assert_eq!(expected_bound, p.cursor.bound.bound);
    Ok(())
}

#[test]
pub fn calculate_bounds_correctly2() -> Result<()> {
    const INPUT_START: usize = 4;
    let mut p = Printer::new(std::io::sink());
    let width = p.cursor.bound.width;
    let height = p.cursor.bound.height;
    let queue = default_process_fn(&"A\tz\nBC\n".into());
    // 2
    move_to_and_modify_start(&mut p, 0, height - 5);
    p.recalculate_bounds(queue)?;

    let expected_bound = {
        let mut v = vec![width - 1; height];
        v[height - 5] = INPUT_START + 3;
        v[height - 4] = INPUT_START + 2;
        v[height - 3] = INPUT_START;
        v
    };
    assert_eq!(expected_bound, p.cursor.bound.bound);

    Ok(())
}

// helper
fn move_to_and_modify_start(printer: &mut Printer<impl Write>, x: usize, y: usize) {
    printer.cursor.pos.starting_pos.0 = x;
    printer.cursor.pos.starting_pos.1 = y;
    printer.cursor.goto_start();
}
