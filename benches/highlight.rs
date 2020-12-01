#![feature(test)]
extern crate test;

use irust::irust::buffer::Buffer;
use irust::irust::highlight::{highlight, theme::Theme};
use irust::irust::printer::Printer;
use test::Bencher;

const CODE: &str = r#"\
        fn default() -> Self {
        crossterm::terminal::enable_raw_mode().expect("failed to enable raw_mode");
        let raw = Rc::new(RefCell::new(std::io::stdout()));
        Self {
            printer: Default::default(),
            writer: writer::Writer::new(raw.clone()),
            cursor: cursor::Cursor::new(raw),
        }
        "#;

// last run 96ns/iter
#[bench]
fn bench_print_input(b: &mut Bencher) {
    let buffer = Buffer::from_string(&CODE);
    let theme = Theme::default();

    //b.iter(|| highlight(&buffer.buffer, &theme));
    b.iter(|| Printer::_new(std::io::sink()).print_input(&buffer, &theme));
}

// last run 13ns/iter
#[bench]
fn bench_highlight(b: &mut Bencher) {
    let buffer = Buffer::from_string(&CODE);
    let theme = Theme::default();

    b.iter(|| highlight(&buffer.buffer, &theme));
}
