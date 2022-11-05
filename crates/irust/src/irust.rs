mod engine;
use std::io::Read;
use std::net::{SocketAddrV4, TcpListener};
use std::sync::mpsc;
use std::time::Duration;

use engine::Engine;
mod art;
mod format;
mod help;
mod highlight;
mod history;
pub mod options;
mod parser;
mod racer;
mod script;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use highlight::{theme::Theme, Highlight};
use history::History;
use irust_api::{Command, GlobalVariables};
use irust_repl::Repl;
use options::Options;
use printer::{buffer::Buffer, printer::Printer};
use racer::Racer;
use script::Script;

pub use format::format_err;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct IRust {
    options: Options,
    buffer: Buffer,
    printer: Printer<std::io::Stdout>,
    engine: Engine,
    exit_flag: bool,
    theme: Theme,
    repl: Repl,
    global_variables: GlobalVariables,
    history: History,
    racer: Option<Racer>,
    script_mg: Option<Box<dyn Script>>,
    highlight: Highlight,
}

impl IRust {
    pub fn new(options: Options) -> Self {
        // Make sure to call Repl::new at the start so it can set `irust-repl` dir, which might be used by others (ScriptManager)
        let repl = Repl::new(
            options.toolchain,
            options.executor,
            options.main_result,
            options.edition,
            // prelude dir
            (|| Some(dirs::data_dir()?.join("irust")))(),
        )
        .expect("Could not create repl");

        let mut global_variables = GlobalVariables::new();

        let mut script_mg = Self::choose_script_mg(&options);
        let prompt = script_mg
            .as_mut()
            .map(|script_mg| {
                if let Some(prompt) = script_mg.input_prompt(&global_variables) {
                    prompt
                } else {
                    options.input_prompt.clone()
                }
            })
            .unwrap_or_else(|| options.input_prompt.clone());

        global_variables.prompt_len = prompt.chars().count();

        let printer = Printer::new(std::io::stdout(), prompt);

        let racer = if options.enable_racer {
            Racer::start()
        } else {
            None
        };

        let buffer = Buffer::new();
        let engine = Engine::default();
        let exit_flag = false;
        let theme = highlight::theme::theme().unwrap_or_default();
        let history = History::new().unwrap_or_default();
        let highlight = Highlight::new(&options.highlight_engine);

        IRust {
            options,
            buffer,
            printer,
            engine,
            exit_flag,
            theme,
            repl,
            global_variables,
            history,
            racer,
            script_mg,
            highlight,
        }
    }

    fn prepare(&mut self) -> Result<()> {
        // title is optional
        let title = if let Some(title) = self.trigger_set_title_hook() {
            title
        } else {
            format!("IRust: {}", self.global_variables.get_cwd().display())
        };
        self.printer.writer.raw.set_title(&title)?;
        self.welcome()?;
        self.warn_highlight_engine()?;
        self.printer.print_prompt_if_set()?;

        // Scripts might want run some startup commands, give them a chance here
        self.run_scripts_startup_cmds()?;

        Ok(())
    }

    /// Wrapper over printer.print_input that highlights rust code using current theme
    pub fn print_input(&mut self) -> Result<()> {
        let theme = &self.theme;
        self.printer.print_input(
            &|buffer| self.highlight.highlight(buffer, theme),
            &self.buffer,
        )?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        self.prepare()?;

        let mut server = if self.options.local_server {
            // Only one rust instance gets to act as a server
            start_server(self.options.local_server_adress).ok()
        } else {
            None
        };

        loop {
            // flush queued output after each key
            // some events that have an inner input loop like ctrl-r/ ctrl-d require flushing inside their respective handler function
            std::io::Write::flush(&mut self.printer.writer.raw)?;

            let evs = if let Some(ref mut server) = server {
                read_from_net_and_stdin(server)
            } else {
                vec![crossterm::event::read()]
            };

            for ev in evs {
                match ev {
                    Ok(ev) => {
                        self.handle_input_event(ev)?;
                        if self.exit_flag {
                            return Ok(());
                        }
                    }
                    Err(e) => return Err(format!("failed to read input. error: {}", e).into()),
                }
            }
        }
    }

    fn handle_input_event(&mut self, ev: Event) -> Result<()> {
        // update_script_state before anything else
        self.update_script_state();

        // check if a script want to act upon this event
        // if so scripts have precedence over normal flow
        if let Some(command) = self.input_event_hook(ev.clone()) {
            self.execute(command)?;
            return Ok(());
        }

        // handle input event
        match ev {
            Event::Mouse(_) => (),
            Event::Resize(width, height) => {
                self.printer.cursor.update_dimensions(width, height);
                //Hack
                self.execute(Command::HandleCtrlC)?;
            }
            Event::Key(key_event) => match key_event {
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                    ..
                } => self.execute(Command::HandleCharacter(c))?,
                KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlE)?,
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::ALT,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleAltEnter)?,
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => self.execute(Command::HandleEnter(false))?,
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => self.execute(Command::HandleTab)?,
                KeyEvent {
                    code: KeyCode::BackTab,
                    ..
                } => self.execute(Command::HandleBackTab)?,
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => self.execute(Command::HandleLeft)?,
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => self.execute(Command::HandleRight)?,
                KeyEvent {
                    code: KeyCode::Up, ..
                } => self.execute(Command::HandleUp)?,
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => self.execute(Command::HandleDown)?,
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => self.execute(Command::HandleBackSpace)?,
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlC)?,
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    self.execute(Command::HandleCtrlD)?;
                }
                KeyEvent {
                    code: KeyCode::Char('z'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlZ)?,
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlL)?,
                KeyEvent {
                    code: KeyCode::Char('r'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlR)?,
                KeyEvent {
                    code: KeyCode::Char('o'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::MacroRecordToggle)?,
                KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::MacroPlay)?,
                KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::Undo)?,
                KeyEvent {
                    code: KeyCode::Char('y'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::Redo)?,
                KeyEvent {
                    code: KeyCode::Char('x'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::Multiple(vec![
                    Command::HandleHome,
                    Command::DeleteUntilChar('\n', true),
                ]))?,
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => self.execute(Command::HandleHome)?,
                KeyEvent {
                    code: KeyCode::End, ..
                } => self.execute(Command::HandleEnd)?,
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlLeft)?,
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.execute(Command::HandleCtrlRight)?,
                KeyEvent {
                    code: KeyCode::Delete,
                    ..
                } => {
                    self.execute(Command::HandleDelete)?;
                    self.execute(Command::PrintInput)?;
                }
                keyevent => {
                    // Handle AltGr on windows
                    if keyevent
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    {
                        if let KeyCode::Char(c) = keyevent.code {
                            self.execute(Command::HandleCharacter(c))?;
                        }
                    }
                }
            },
            Event::FocusGained => (),
            Event::FocusLost => (),
            Event::Paste(_data) => (),
        }
        Ok(())
    }
}

fn start_server(adress: SocketAddrV4) -> Result<mpsc::Receiver<String>> {
    let server = TcpListener::bind(adress)?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = String::new();
        loop {
            (|| {
                let mut c = server.accept().ok()?.0;
                c.read_to_string(&mut buf).ok()?;
                tx.send(buf.clone()).ok()?;
                buf.clear();
                Some(())
            })();
        }
    });
    Ok(rx)
}
fn read_from_net_and_stdin(server: &mut mpsc::Receiver<String>) -> Vec<std::io::Result<Event>> {
    loop {
        if let Ok(e) = server.try_recv() {
            return e
                .chars()
                .map(|c| {
                    Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }))
                })
                .chain(std::iter::once(Ok(Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }))))
                .collect();
        }
        if let Ok(true) = crossterm::event::poll(Duration::from_millis(100)) {
            return vec![crossterm::event::read()];
        }
    }
}

impl Drop for IRust {
    fn drop(&mut self) {
        // ignore errors on drop with let _
        let _ = self.execute(Command::Exit);
        if std::thread::panicking() {
            let _ = self.printer.writer.raw.write("IRust panicked, to log the error you can redirect stderror to a file, example irust 2>log");
        }
    }
}
