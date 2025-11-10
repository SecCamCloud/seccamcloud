// ============================================================================
// SecCamCloud - Core Library
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use log::{info, LevelFilter};
use simplelog::{ConfigBuilder, WriteLogger, TermLogger, TerminalMode, ColorChoice, CombinedLogger};

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================

pub mod config;
pub mod watchdog;
pub mod telemetry;
pub mod screenshot;
pub mod automation;
pub mod vidrec;
pub mod youtube;

// ============================================================================
// RE-EXPORTS
// ============================================================================

// Configuration
pub use config::{ClickPoint, AppConfig, DEFAULT_POINTS, load_points, save_points};

// Watchdog
pub use watchdog::WatchdogTimer;

// Telemetry
pub use telemetry::Telemetry;

// Screenshot
pub use screenshot::ScreenshotManager;

// Automation
pub use automation::{AutomationThread, AutomationMessage};

// Video Recording
pub use vidrec::{VideoRecorder, VideoConfig, VideoFormat, CameraInfo, VideoMessage};

// YouTube Upload
pub use youtube::{
    YouTubeUploader, YouTubeCredentials, VideoMetadata, VideoPrivacy, VideoCategory,
    VideoInfo, VideoValidator, UploadMessage, UploadStatus, BatchUploader,
};

// ============================================================================
// PUBLIC CONSTANTS
// ============================================================================

pub const APP_TITLE: &str = "SecCamCloud";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LOG_FILE: &str = "automation_log.txt";

// Log rotation settings
const MAX_LOG_BYTES: u64 = 5_000_000;  // 5MB
const MAX_LOG_BACKUPS: usize = 3;

// ============================================================================
// PLATFORM UTILITIES
// ============================================================================

/// Check if running on Windows platform
#[inline]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Check if a Windows virtual key is currently pressed
#[cfg(target_os = "windows")]
pub fn key_pressed(vk_code: i32) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    unsafe { 
        (GetAsyncKeyState(vk_code) & 0x8000u16 as i16) != 0 
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn key_pressed(_vk_code: i32) -> bool {
    false
}

// ============================================================================
// LOG ROTATION
// ============================================================================

/// Rotate log files when they exceed size limit
fn rotate_logs() {
    if !Path::new(LOG_FILE).exists() {
        return;
    }
    
    // Check file size
    let meta = match std::fs::metadata(LOG_FILE) {
        Ok(m) => m,
        Err(_) => return,
    };
    
    if meta.len() < MAX_LOG_BYTES {
        return;
    }
    
    info!("Rotating log files (size: {} bytes)", meta.len());
    
    // Rotate backups: .3 -> delete, .2 -> .3, .1 -> .2, current -> .1
    for i in (1..=MAX_LOG_BACKUPS).rev() {
        let old_name = if i == 1 {
            LOG_FILE.to_string()
        } else {
            format!("{}.{}", LOG_FILE, i - 1)
        };
        
        let new_name = format!("{}.{}", LOG_FILE, i);
        
        if Path::new(&old_name).exists() {
            if i == MAX_LOG_BACKUPS {
                let _ = std::fs::remove_file(&old_name);
            } else {
                let _ = std::fs::rename(&old_name, &new_name);
            }
        }
    }
    
    // Create fresh log file
    let _ = File::create(LOG_FILE);
}

// ============================================================================
// LOGGING INITIALIZATION
// ============================================================================

/// Initialize logging system with rotation and dual output
pub fn setup_logging() {
    rotate_logs();
    
    let config = ConfigBuilder::new()
        .set_time_format_str("%Y-%m-%d %H:%M:%S%.3f")
        .set_time_to_local(true)
        .set_target_level(LevelFilter::Error)
        .set_location_level(LevelFilter::Error)
        .build();
    
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = Vec::new();
    
    // File logger
    if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
    {
        loggers.push(WriteLogger::new(LevelFilter::Debug, config.clone(), file));
    } else {
        eprintln!("⚠ Warning: Could not open log file {}", LOG_FILE);
    }
    
    // Terminal logger
    if let Some(term_logger) = TermLogger::new(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto
    ) {
        loggers.push(term_logger);
    }
    
    if let Err(e) = CombinedLogger::init(loggers) {
        eprintln!("Failed to initialize logger: {}", e);
        return;
    }
    
    info!("========================================");
    info!("{} v{}", APP_TITLE, APP_VERSION);
    info!("Session started");
    info!("========================================");
}
