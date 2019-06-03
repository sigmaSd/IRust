use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const WAIT_TIMEOUT: u64 = 110;

pub struct Debouncer {
    timer: Arc<Mutex<Instant>>,
    _function: Option<fn()>,
    send: mpsc::Sender<usize>,
    pub recv: mpsc::Receiver<usize>,
    pub lock: bool,
}

impl Debouncer {
    pub fn new() -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            timer: Arc::new(Mutex::new(Instant::now())),
            _function: None,
            send,
            recv,
            lock: false,
        }
    }

    pub fn _schedule_fn(&mut self, _f: fn()) {
        //self.function = Some(f);
    }

    pub fn run(&mut self) {
        let send = self.send.clone();
        let timer = self.timer.clone();
        std::thread::spawn(move || loop {
            if timer.lock().unwrap().elapsed() >= Duration::from_millis(WAIT_TIMEOUT) {
                send.send(1).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        });
    }

    pub fn _check(&mut self) {
        if self.recv.try_recv().is_ok() {
            //self.function;
            self.reset_timer();
        }
        // if self.timer.elapsed() >= Duration::from_millis(WAIT_TIMEOUT) {
        //     self.reset_timer();
        //     Ok(())
        // } else {
        //     self.reset_timer();
        //     Err(())
        // }
    }

    pub fn reset_timer(&mut self) {
        *self.timer.lock().unwrap() = Instant::now();
    }
}
