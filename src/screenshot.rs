// ============================================================================
// SecCamCloud - Screenshot Module
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use chrono::Local;

// ============================================================================
// SCREENSHOT MANAGER
// ============================================================================

pub struct ScreenshotManager {
    enabled: bool,
    output_dir: String,
}

impl ScreenshotManager {
    pub fn new(enabled: bool) -> Arc<Self> {
        let output_dir = "screenshots".to_string();

        if enabled {
            let _ = std::fs::create_dir_all(&output_dir);
        }

        Arc::new(Self {
            enabled,
            output_dir,
        })
    }

    #[allow(unused_variables)]
    pub fn capture(&self, step_name: &str, suffix: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }

        #[cfg(feature = "screenshots")]
        {
            use image::{Rgba, RgbaImage};
            use scrap::{Capturer, Display};

            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("{}/{}_{}_{}.png", self.output_dir, step_name, suffix, timestamp);

            let display = Display::primary().ok()?;
            let mut capturer = Capturer::new(display).ok()?;

            for _ in 0..5 {
                if let Ok(frame) = capturer.frame() {
                    let width = capturer.width();
                    let height = capturer.height();
                    let mut img = RgbaImage::new(width as u32, height as u32);

                    for y in 0..height {
                        for x in 0..width {
                            let idx = (y * width + x) * 4;
                            if idx + 3 < frame.len() {
                                img.put_pixel(
                                    x as u32,
                                    y as u32,
                                    Rgba([frame[idx + 2], frame[idx + 1], frame[idx], 255]),
                                );
                            }
                        }
                    }

                    if img.save(&filename).is_ok() {
                        return Some(filename);
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
        }

        None
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
