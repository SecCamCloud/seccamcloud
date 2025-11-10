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
use log::{info, warn};

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
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("{}/{}_{}_{}.png", self.output_dir, step_name, suffix, timestamp);

            // Try captrs first (supports both X11 and Wayland)
            if let Some(path) = self.capture_with_captrs(&filename) {
                info!("Screenshot captured with captrs: {}", path);
                return Some(path);
            }

            // Fallback to scrap (X11 only)
            if let Some(path) = self.capture_with_scrap(&filename) {
                info!("Screenshot captured with scrap: {}", path);
                return Some(path);
            }

            warn!("Failed to capture screenshot with both methods");
        }

        None
    }

    /// Capture screenshot using captrs (X11 and Wayland)
    #[cfg(feature = "screenshots")]
    fn capture_with_captrs(&self, filename: &str) -> Option<String> {
        use captrs::Capturer;

        match Capturer::new(0) {
            Ok(mut capturer) => {
                match capturer.capture_frame() {
                    Ok(frame) => {
                        // Convert frame to image
                        let (width, height) = (frame.width(), frame.height());
                        
                        // captrs returns BGRA, convert to RGBA for image crate
                        let mut rgba_data = Vec::with_capacity(width * height * 4);
                        for pixel in frame.chunks_exact(4) {
                            rgba_data.push(pixel[2]); // R
                            rgba_data.push(pixel[1]); // G
                            rgba_data.push(pixel[0]); // B
                            rgba_data.push(pixel[3]); // A
                        }

                        // Save as PNG
                        if let Ok(img) = image::RgbaImage::from_raw(width as u32, height as u32, rgba_data) {
                            if img.save(filename).is_ok() {
                                return Some(filename.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        warn!("captrs capture_frame failed: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("captrs Capturer::new failed: {}", e);
            }
        }

        None
    }

    /// Capture screenshot using scrap (X11 only, fallback)
    #[cfg(feature = "screenshots")]
    fn capture_with_scrap(&self, filename: &str) -> Option<String> {
        use image::{Rgba, RgbaImage};
        use scrap::{Capturer, Display};

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

                if img.save(filename).is_ok() {
                    return Some(filename.to_string());
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        None
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

