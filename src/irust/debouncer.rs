use std::time::{Duration, Instant};

const WAIT_TIMEOUT: u64 = 50;

pub struct Debouncer {
    timer: Instant,
}

impl Debouncer {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
        }
    }

    pub fn run(&mut self, mut function: impl FnMut() -> std::io::Result<()>) {
        if self.timer.elapsed() >= Duration::from_millis(WAIT_TIMEOUT) {
            let _ = function();
        }
        self.timer = Instant::now();
    }
}
