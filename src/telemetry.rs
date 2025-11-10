// ============================================================================
// SecCamCloud - Telemetry Module
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use chrono::Local;

// ============================================================================
// TELEMETRY SYSTEM
// ============================================================================

pub struct Telemetry {
    enabled: bool,
}

impl Telemetry {
    pub fn new(enabled: bool) -> Arc<Self> {
        let telemetry = Arc::new(Self { enabled });

        if enabled {
            let _ = std::fs::create_dir_all("logs");
            telemetry.log("Telemetry initialized");
        }

        telemetry
    }

    pub fn log(&self, event: impl AsRef<str>) {
        if !self.enabled {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}] {}\n", timestamp, event.as_ref());

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/telemetry.log")
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}
