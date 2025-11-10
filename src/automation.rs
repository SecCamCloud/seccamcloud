// ============================================================================
// SecCamCloud - Automation Module
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use chrono::Local;
use log::{info, error};
use enigo::{Enigo, Button, Direction, Coordinate, Settings, Keyboard, Mouse};

use crate::config::ClickPoint;
use crate::watchdog::WatchdogTimer;

// ============================================================================
// AUTOMATION MESSAGES
// ============================================================================

/// Messages sent from automation thread to GUI
#[derive(Debug, Clone)]
pub enum AutomationMessage {
    Log(String),
    Status(String),
    UpdateTimer(i32),
    ErrorPopup(String),
    Stop,
}

// ============================================================================
// AUTOMATION THREAD
// ============================================================================

/// Main automation execution thread
pub struct AutomationThread {
    points: Vec<ClickPoint>,
    total_seconds: i32,
    step_delay: i32,
    max_retries: i32,
    step4_wait_sec: i32,
    dry_run: bool,
    tx_to_gui: Sender<AutomationMessage>,
    rx_stop: Receiver<()>,
    stop_flag: Arc<AtomicBool>,
}

impl AutomationThread {
    pub fn new(
        points: Vec<ClickPoint>,
        total_seconds: i32,
        step_delay: i32,
        max_retries: i32,
        step4_wait_sec: i32,
        dry_run: bool,
        tx_to_gui: Sender<AutomationMessage>,
        rx_stop: Receiver<()>,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            points,
            total_seconds: total_seconds.max(0),
            step_delay: step_delay.max(0),
            max_retries: max_retries.max(1),
            step4_wait_sec: step4_wait_sec.max(0),
            dry_run,
            tx_to_gui,
            rx_stop,
            stop_flag,
        }
    }
    
    fn log(&self, msg: impl AsRef<str>) {
        let msg = msg.as_ref();
        info!("{}", msg);
        let _ = self.tx_to_gui.send(AutomationMessage::Log(msg.to_string()));
    }
    
    fn update_status(&self, status: impl AsRef<str>) {
        let _ = self.tx_to_gui.send(AutomationMessage::Status(status.as_ref().to_string()));
    }
    
    fn update_timer(&self, remaining: i32) {
        let _ = self.tx_to_gui.send(AutomationMessage::UpdateTimer(remaining));
    }
    
    fn error_popup(&self, msg: impl AsRef<str>) {
        let msg = msg.as_ref();
        error!("{}", msg);
        let _ = self.tx_to_gui.send(AutomationMessage::ErrorPopup(msg.to_string()));
    }
    
    fn is_running(&self) -> bool {
        !self.stop_flag.load(Ordering::SeqCst) && self.rx_stop.try_recv().is_err()
    }
    
    fn sleep_with_check(&self, seconds: i32) -> bool {
        for _ in 0..seconds {
            if !self.is_running() {
                self.log("Interrupted during sleep");
                return false;
            }
            thread::sleep(Duration::from_secs(1));
        }
        true
    }
    
    fn execute_click(&self, point: &ClickPoint, watchdog: &WatchdogTimer) -> bool {
        for attempt in 1..=self.max_retries {
            if !self.is_running() {
                return false;
            }
            
            self.log(format!("[{}] Attempt {}/{}", point.name, attempt, self.max_retries));
            
            if !self.dry_run {
                match Enigo::new(&Settings::default()) {
                    Ok(mut enigo) => {
                        // Move mouse
                        if let Err(e) = enigo.move_mouse(point.x, point.y, Coordinate::Abs) {
                            error!("Mouse move failed: {}", e);
                            continue;
                        }
                        thread::sleep(Duration::from_millis(50));
                        
                        // Click
                        if let Err(e) = enigo.button(Button::Left, Direction::Click) {
                            error!("Mouse click failed: {}", e);
                            continue;
                        }
                    }
                    Err(e) => {
                        error!("Enigo creation failed: {}", e);
                        continue;
                    }
                }
            } else {
                self.log(format!("[DRY RUN] Would click {} at ({}, {})", point.name, point.x, point.y));
            }
            
            // Success - wait step delay
            watchdog.cancel();
            if !self.sleep_with_check(self.step_delay) {
                return false;
            }
            watchdog.reset();
            
            return true;
        }
        
        error!("Failed after {} retries: {}", self.max_retries, point.name);
        false
    }
    
    fn type_text(&self, text: &str) -> Result<(), String> {
        if self.dry_run {
            self.log(format!("[DRY RUN] Would type: {}", text));
            return Ok(());
        }
        
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                enigo.text(text).map_err(|e| format!("Type failed: {}", e))
            }
            Err(e) => Err(format!("Enigo creation failed: {}", e)),
        }
    }
    
    pub fn run(mut self) {
        info!("Automation thread started");
        self.update_status("Status: Running");
        
        // Setup watchdog
        let tx_clone = self.tx_to_gui.clone();
        let stop_clone = self.stop_flag.clone();
        let watchdog = WatchdogTimer::new(
            (self.max_retries as u64 * 3).max(30),
            move || {
                error!("Watchdog timeout - automation unresponsive");
                let _ = tx_clone.send(AutomationMessage::Log("⚠ Watchdog timeout".to_string()));
                let _ = tx_clone.send(AutomationMessage::Status("Status: Error - Timeout".to_string()));
                stop_clone.store(true, Ordering::SeqCst);
            },
        );
        
        // Run automation
        if let Err(e) = self.automation_loop(&watchdog) {
            error!("Automation error: {}", e);
            self.error_popup(format!("Automation Error: {}", e));
        }
        
        // Cleanup
        self.stop_flag.store(true, Ordering::SeqCst);
        let _ = self.tx_to_gui.send(AutomationMessage::Stop);
        info!("Automation thread stopped");
    }
    
    fn automation_loop(&mut self, watchdog: &WatchdogTimer) -> Result<(), String> {
        let mut iteration = 0;
        
        while self.is_running() {
            iteration += 1;
            self.log(format!("===== Iteration {} =====", iteration));
            
            // Step 1
            watchdog.reset();
            if !self.execute_click(&self.points[0], watchdog) {
                return Err(format!("Failed: {}", self.points[0].name));
            }
            
            // Step 2 - Click date field and enter date in DD-MM-YYYY format
            watchdog.reset();
            if !self.execute_click(&self.points[1], watchdog) {
                return Err(format!("Failed: {}", self.points[1].name));
            }
            
            let date = Local::now().format("%d-%m-%Y").to_string();
            self.type_text(&date)?;
            self.log(format!("Entered date: {}", date));
            
            if !self.sleep_with_check(2) {
                break;
            }
            
            // Step 3
            watchdog.reset();
            if !self.execute_click(&self.points[2], watchdog) {
                return Err(format!("Failed: {}", self.points[2].name));
            }
            
            // Step 4 - Short wait
            self.log(format!("Step 4: Waiting {} seconds", self.step4_wait_sec));
            watchdog.cancel();
            if !self.sleep_with_check(self.step4_wait_sec) {
                break;
            }
            watchdog.reset();
            
            // Step 5
            watchdog.reset();
            if !self.execute_click(&self.points[3], watchdog) {
                return Err(format!("Failed: {}", self.points[3].name));
            }
            
            // Step 6 - Long wait
            let hours = self.total_seconds / 3600;
            let minutes = (self.total_seconds % 3600) / 60;
            self.log(format!("Step 6: Long wait {}h {}m", hours, minutes));
            watchdog.cancel();
            
            let mut remaining = self.total_seconds;
            while remaining > 0 && self.is_running() {
                self.update_timer(remaining);
                thread::sleep(Duration::from_secs(1));
                remaining -= 1;
            }
            
            if !self.is_running() {
                break;
            }
            
            self.log("Long wait completed");
            watchdog.reset();
            
            if !self.sleep_with_check(2) {
                break;
            }
            
            // Step 7
            watchdog.reset();
            if !self.execute_click(&self.points[4], watchdog) {
                return Err(format!("Failed: {}", self.points[4].name));
            }
            
            // Step 8
            watchdog.reset();
            if !self.execute_click(&self.points[5], watchdog) {
                return Err(format!("Failed: {}", self.points[5].name));
            }
            
            self.log(format!("===== Iteration {} complete =====", iteration));
            
            watchdog.cancel();
            if !self.sleep_with_check(5) {
                break;
            }
            watchdog.reset();
        }
        
        Ok(())
    }
}
