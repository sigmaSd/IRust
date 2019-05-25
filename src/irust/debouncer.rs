use std::time::{Duration, Instant};

const WAIT_TIMEOUT: u64 = 110;

pub struct Debouncer {
    timer: Instant,
}

impl Debouncer {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
        }
    }

    pub fn check(&mut self) -> Result<(), ()> {
        if self.timer.elapsed() >= Duration::from_millis(WAIT_TIMEOUT) {
            self.reset_timer();
            Ok(())
        } else {
            self.reset_timer();
            Err(())
        }
    }

    pub fn reset_timer(&mut self) {
        self.timer = Instant::now();
    }
}
