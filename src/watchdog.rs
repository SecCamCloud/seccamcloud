// ============================================================================
// SecCamCloud - Watchdog Timer Module
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

// ============================================================================
// WATCHDOG TIMER
// ============================================================================

/// Safety timer to detect automation hangs
pub struct WatchdogTimer {
    start: Arc<Mutex<Option<Instant>>>,
    timeout_sec: u64,
    _thread: Option<JoinHandle<()>>,
}

impl WatchdogTimer {
    pub fn new<F>(timeout_sec: u64, on_timeout: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let start = Arc::new(Mutex::new(Some(Instant::now())));
        let start_clone = start.clone();
        
        let handle = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                
                let guard = start_clone.lock().unwrap();
                if let Some(s) = *guard {
                    if s.elapsed().as_secs() > timeout_sec {
                        drop(guard);
                        on_timeout();
                        break;
                    }
                } else {
                    break;
                }
            }
        });
        
        Self {
            start,
            timeout_sec,
            _thread: Some(handle),
        }
    }
    
    pub fn reset(&self) {
        let mut guard = self.start.lock().unwrap();
        *guard = Some(Instant::now());
    }
    
    pub fn cancel(&self) {
        let mut guard = self.start.lock().unwrap();
        *guard = None;
    }
}
