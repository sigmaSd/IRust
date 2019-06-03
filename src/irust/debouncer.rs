use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const WAIT_TIMEOUT: u64 = 10;
const SLEEP_TIME: u64 = 300;

pub struct Debouncer {
    timer: Arc<Mutex<Instant>>,
    send: mpsc::Sender<usize>,
    pub recv: mpsc::Receiver<usize>,
}

impl Debouncer {
    pub fn new() -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            timer: Arc::new(Mutex::new(Instant::now())),
            send,
            recv,
        }
    }

    pub fn run(&mut self) {
        let send = self.send.clone();
        let timer = self.timer.clone();
        std::thread::spawn(move || loop {
            if timer.lock().unwrap().elapsed() >= Duration::from_millis(WAIT_TIMEOUT) {
                send.send(1).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(SLEEP_TIME));
        });
    }

    pub fn reset_timer(&mut self) {
        *self.timer.lock().unwrap() = Instant::now();
    }
}
