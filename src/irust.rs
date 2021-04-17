mod art;
mod cargo_cmds;
mod events;
mod format;
mod global_variables;
mod help;
pub mod highlight;
mod history;
pub mod options;
mod parser;
pub mod printer;
mod racer;
mod repl;
use crossterm::event::*;
use crossterm::style::Color;
use history::History;
use options::Options;
use racer::Racer;
use repl::Repl;
pub mod buffer;
use buffer::Buffer;
use crossterm::event::KeyModifiers;
use global_variables::GlobalVariables;
use highlight::theme::Theme;
use once_cell::sync::Lazy;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const IN: &str = "In: ";
const OUT: &str = "Out: ";
pub const CTRL_KEYMODIFIER: KeyModifiers = KeyModifiers::CONTROL;
const ALT_KEYMODIFIER: KeyModifiers = KeyModifiers::ALT;
const SHIFT_KEYMODIFIER: KeyModifiers = KeyModifiers::SHIFT;
pub const NO_MODIFIER: KeyModifiers = KeyModifiers::empty();

static SOUT: Lazy<std::io::Stdout> = Lazy::new(std::io::stdout);

pub struct IRust {
    buffer: Buffer,
    repl: Repl,
    printer: printer::Printer<std::io::StdoutLock<'static>>,
    options: Options,
    racer: Option<Racer>,
    global_variables: GlobalVariables,
    theme: Theme,
    history: History,
}

impl IRust {
    pub fn new(options: Options) -> Self {
        let out = SOUT.lock();
        let printer = printer::Printer::new(out);

        let global_variables = GlobalVariables::new();

        let repl = Repl::new();
        let racer = if options.enable_racer {
            Racer::start()
        } else {
            None
        };
        let buffer = Buffer::new();
        let theme = highlight::theme::theme().unwrap_or_default();
        let history = History::new().unwrap_or_default();

        IRust {
            repl,
            printer,
            options,
            racer,
            buffer,
            global_variables,
            theme,
            history,
        }
    }

    fn prepare(&mut self) -> Result<()> {
        // title is optional
        self.printer.writer.raw.set_title(&format!(
            "IRust: {}",
            self.global_variables.get_cwd().display()
        ))?;
        self.repl.prepare_ground(self.options.toolchain)?;
        self.welcome()?;
        self.printer.write_from_terminal_start(IN, Color::Yellow)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        self.prepare()?;

        loop {
            // flush queued output after each key
            // some events that have an inner input loop like ctrl-r/ ctrl-d require flushing inside their respective handler function
            std::io::Write::flush(&mut self.printer.writer.raw)?;

            match crossterm::event::read() {
                Ok(ev) => {
                    let exit = self.handle_input_event(ev)?;
                    if exit {
                        break Ok(());
                    }
                }
                Err(e) => break Err(format!("failed to read input. error: {}", e).into()),
            }
        }
    }

    fn handle_input_event(&mut self, ev: crossterm::event::Event) -> Result<bool> {
        // handle input event
        match ev {
            Event::Mouse(_) => (),
            Event::Resize(width, height) => {
                self.printer.cursor.update_dimensions(width, height);
                //Hack
                self.handle_ctrl_c()?;
            }
            Event::Key(key_event) => match key_event {
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: NO_MODIFIER,
                }
                | KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: SHIFT_KEYMODIFIER,
                } => {
                    self.handle_character(c)?;
                }
                KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_e()?;
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: ALT_KEYMODIFIER,
                } => {
                    self.handle_alt_enter()?;
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    self.handle_enter(false)?;
                }
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => {
                    self.handle_tab()?;
                }
                KeyEvent {
                    code: KeyCode::BackTab,
                    ..
                } => {
                    self.handle_back_tab()?;
                }
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: NO_MODIFIER,
                } => {
                    self.handle_left()?;
                }
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: NO_MODIFIER,
                } => {
                    self.handle_right()?;
                }
                KeyEvent {
                    code: KeyCode::Up, ..
                } => {
                    self.handle_up()?;
                }
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => {
                    self.handle_down()?;
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    self.handle_backspace()?;
                }
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_c()?;
                }
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    return self.handle_ctrl_d();
                }
                KeyEvent {
                    code: KeyCode::Char('z'),
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_z()?;
                }
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_l()?;
                }
                KeyEvent {
                    code: KeyCode::Char('r'),
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_r()?;
                }
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => {
                    self.handle_home_key()?;
                }
                KeyEvent {
                    code: KeyCode::End, ..
                } => {
                    self.handle_end_key()?;
                }
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_left()?;
                }
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: CTRL_KEYMODIFIER,
                } => {
                    self.handle_ctrl_right()?;
                }
                KeyEvent {
                    code: KeyCode::Delete,
                    ..
                } => {
                    self.handle_del()?;
                }
                keyevent => {
                    // Handle AltGr on windows
                    if keyevent
                        .modifiers
                        .contains(CTRL_KEYMODIFIER | ALT_KEYMODIFIER)
                    {
                        if let KeyCode::Char(c) = keyevent.code {
                            self.handle_character(c)?;
                        }
                    }
                }
            },
        }
        Ok(false)
    }
}

impl Drop for IRust {
    fn drop(&mut self) {
        // ignore errors on drop with let _
        let _ = self.exit();
        if std::thread::panicking() {
            let _ = self.printer.writer.raw.write("IRust panicked, to log the error you can redirect stderror to a file, example irust 2>log");
        }
    }
}
