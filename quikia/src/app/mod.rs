mod app;
mod page;
mod desktop_impl;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
pub use app::*;
pub use page::*;
pub use desktop_impl::*;

pub struct Timer {
    inner: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    is_running: Arc<Mutex<bool>>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            inner: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start<F>(&self, duration: Duration, callback: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            panic!("Timer is already running!");
        }
        *is_running = true;
        drop(is_running);

        let inner = Arc::clone(&self.inner);
        let is_running = Arc::clone(&self.is_running);
        let handle = thread::spawn(move || {
            thread::park_timeout(duration);
            let mut is_running = is_running.lock().unwrap();
            if *is_running {
                *is_running = false;
                drop(is_running);
                callback();
            }
        });

        let mut guard = inner.lock().unwrap();
        *guard = Some(handle);
    }

    pub fn cancel(&self) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(handle) = guard.take() {
            let mut is_running = self.is_running.lock().unwrap();
            *is_running = false;
            drop(guard);
            handle.thread().unpark();
        }
    }
}