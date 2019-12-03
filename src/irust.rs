use crossterm::{
    input::{input, InputEvent, KeyEvent},
    screen::RawScreen,
    style::Color,
};

mod art;
mod cargo_cmds;
mod cursor;
mod debouncer;
mod events;
mod format;
mod help;
mod highlight;
mod history;
mod irust_error;
pub mod options;
mod parser;
mod printer;
mod racer;
mod repl;
mod writer;
use cursor::{Cursor, INPUT_START_COL};
use debouncer::Debouncer;
use history::History;
use irust_error::IRustError;
use options::Options;
use racer::Racer;
use repl::Repl;
mod buffer;
use buffer::Buffer;
mod raw_terminal;
use raw_terminal::RawTerminal;

const IN: &str = "In: ";
const OUT: &str = "Out: ";

pub struct IRust {
    raw_terminal: RawTerminal,
    buffer: Buffer,
    repl: Repl,
    cursor: Cursor,
    history: History,
    options: Options,
    racer: Result<Racer, IRustError>,
    debouncer: Debouncer,
}

impl IRust {
    pub fn new() -> Self {
        let raw_terminal = RawTerminal::new();
        let repl = Repl::new();
        let history = History::new(dirs::cache_dir().unwrap().join("irust")).unwrap_or_default();
        let options = Options::new().unwrap_or_default();
        let debouncer = Debouncer::new();
        let racer = if options.enable_racer {
            Racer::start()
        } else {
            Err(IRustError::RacerDisabled)
        };
        let size = {
            let (width, height) = raw_terminal.size().expect("Error getting terminal size");
            (width as usize, height as usize)
        };
        let cursor = Cursor::new(0, 0, size.0, size.1);
        let buffer = Buffer::new(size.0 - INPUT_START_COL);

        IRust {
            cursor,
            raw_terminal,
            repl,
            history,
            options,
            racer,
            debouncer,
            buffer,
        }
    }

    fn prepare(&mut self) -> Result<(), IRustError> {
        self.repl.prepare_ground()?;
        self.debouncer.run();
        self.welcome()?;
        self.write_from_terminal_start(IN, Color::Yellow)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), IRustError> {
        // ignore events after ctrl-r
        // new crossterm event loop will run the events twice if this is not used
        let mut ignore_events = 0;

        let _screen = RawScreen::into_raw_mode()?;
        self.prepare()?;
        let mut stdin = input().read_sync();

        loop {
            self.check_racer_callback()?;
            if let Some(key_event) = stdin.next() {
                // hack to ignore repeted events after ctrl-r
                if ignore_events > 0 {
                    ignore_events -= 1;
                    continue;
                }

                match key_event {
                    InputEvent::Keyboard(KeyEvent::Char(c)) => {
                        self.handle_character(c)?;
                    }
                    InputEvent::Keyboard(KeyEvent::Enter) => {
                        self.handle_enter()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Tab) => {
                        self.handle_tab()?;
                    }
                    InputEvent::Keyboard(KeyEvent::BackTab) => {
                        self.handle_back_tab()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Left) => {
                        self.handle_left()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Right) => {
                        self.handle_right()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Up) => {
                        self.handle_up()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Down) => {
                        self.handle_down()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Backspace) => {
                        self.handle_backspace()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('c')) => {
                        self.handle_ctrl_c()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('d')) => {
                        self.handle_ctrl_d()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('z')) => {
                        self.handle_ctrl_z()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('l')) => {
                        self.handle_ctrl_l()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('r')) => {
                        ignore_events = self.handle_ctrl_r()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Alt('\r')) => {
                        self.handle_alt_enter()?;
                    }
                    InputEvent::Keyboard(KeyEvent::Home) => {
                        self.handle_home_key()?;
                    }
                    InputEvent::Keyboard(KeyEvent::End) => {
                        self.handle_end_key()?;
                    }
                    InputEvent::Keyboard(KeyEvent::CtrlLeft) => {
                        self.handle_ctrl_left();
                    }
                    InputEvent::Keyboard(KeyEvent::CtrlRight) => {
                        self.handle_ctrl_right();
                    }
                    InputEvent::Keyboard(KeyEvent::Delete) => {
                        self.handle_del()?;
                    }
                    _ => {}
                }
            }
        }
    }
}
