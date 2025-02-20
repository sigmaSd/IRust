use std::fs::OpenOptions;
use std::io::Result;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

pub static ACTIVE_LOGGER: AtomicBool = AtomicBool::new(false);
static LOG_FILE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init_log(file_path: impl Into<PathBuf>, env: &str) {
    if std::env::var(env).is_err() {
        return;
    }
    let mut log_file_path = LOG_FILE_PATH.lock().unwrap();
    *log_file_path = Some(file_path.into());
    ACTIVE_LOGGER.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn log_to_file(message: &str) -> Result<()> {
    let file_path = LOG_FILE_PATH.lock().unwrap();
    if let Some(ref path) = *file_path {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", message)?;
    }
    Ok(())
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        use std::fmt::Write as FmtWrite;
        if crate::log::ACTIVE_LOGGER.load(std::sync::atomic::Ordering::Relaxed) {
            let mut message = String::new();
            write!(&mut message, $($arg)*).unwrap();
            crate::log::log_to_file(&message).unwrap();
        }
    }};
}
