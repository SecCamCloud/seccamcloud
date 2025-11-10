# SecCamCloud - Video Recording Integration Guide

## Overview

This guide explains how to integrate the `vidrec.rs` module into your SecCamCloud project for video recording capabilities.

---

## Files Delivered

1. **vidrec.rs** - Complete video recording module
2. **Cargo.toml** - Updated with OpenCV dependencies
3. **VIDEO_RECORDING.md** - Comprehensive usage documentation

---

## Installation Steps

### Step 1: Install OpenCV

**Windows:**
```bash
# Option 1: vcpkg (recommended)
vcpkg install opencv4[core,videoio,highgui,imgcodecs]:x64-windows

# Option 2: Download from opencv.org and set OPENCV_DIR
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install libopencv-dev clang libclang-dev pkg-config
```

**macOS:**
```bash
brew install opencv pkg-config
```

### Step 2: Add Files to Project

```bash
# Copy vidrec.rs to your src directory
cp vidrec.rs src/

# Replace Cargo.toml (or manually merge changes)
cp Cargo.toml .
```

### Step 3: Verify lib.rs

Your `lib.rs` should already have these lines (they're in the uploaded version):

```rust
// Module declaration
pub mod vidrec;

// Re-exports
pub use vidrec::{VideoRecorder, VideoConfig, VideoFormat, CameraInfo, VideoMessage};
```

### Step 4: Build with Video Feature

```bash
# Build with video support
cargo build --release --features video

# Or with all features
cargo build --release --features "screenshots,video"
```

---

## What Changed in Cargo.toml

### Added Dependency

```toml
# Video recording (optional feature)
opencv = { version = "0.92", optional = true, default-features = false, features = ["videoio", "highgui", "imgcodecs"] }
```

### Updated Features

```toml
[features]
default = []
screenshots = ["scrap", "image"]
video = ["opencv"]  # Changed from "nokhwa" to "opencv"
```

**Why OpenCV?**
- More mature and stable than nokhwa
- Better support for IP cameras (RTSP/HTTP)
- Wider platform compatibility
- Professional-grade video encoding
- Industry standard for computer vision

---

## Quick Test

### Test 1: Basic Webcam Recording

Create `examples/test_webcam.rs`:

```rust
use seccamcloud::{CameraInfo, CameraSource, VideoConfig, VideoRecorder};
use std::time::Duration;
use std::thread;

fn main() {
    println!("Testing webcam recording...");
    
    let camera = CameraInfo::new("Test Webcam", CameraSource::Webcam(0))
        .with_resolution(1280, 720)
        .with_fps(30.0);
    
    let config = VideoConfig::new()
        .with_output_dir("test_recordings");
    
    let mut recorder = VideoRecorder::new(camera, config);
    
    println!("Starting 10 second recording...");
    recorder.start_recording().unwrap();
    
    thread::sleep(Duration::from_secs(10));
    
    println!("Stopping recording...");
    recorder.stop_recording().unwrap();
    
    println!("Done! Check test_recordings/ directory");
}
```

Run with:
```bash
cargo run --release --features video --example test_webcam
```

### Test 2: IP Camera Recording

Create `examples/test_ipcam.rs`:

```rust
use seccamcloud::{CameraInfo, CameraSource, VideoConfig, VideoRecorder};
use std::time::Duration;
use std::thread;

fn main() {
    // Replace with your camera's URL
    let rtsp_url = "rtsp://admin:password@192.168.1.100:554/stream";
    
    let camera = CameraInfo::new("IP Camera", CameraSource::RtspStream(rtsp_url.to_string()));
    let config = VideoConfig::new();
    
    let mut recorder = VideoRecorder::new(camera, config);
    
    println!("Recording for 30 seconds...");
    recorder.start_recording().unwrap();
    thread::sleep(Duration::from_secs(30));
    recorder.stop_recording().unwrap();
    
    println!("Done!");
}
```

---

## Integration with GUI (main.rs)

### Add to AppState

```rust
use seccamcloud::{VideoRecorder, VideoMessage, CameraInfo, VideoConfig};
use std::sync::mpsc::{Sender, Receiver, channel};

struct AppState {
    // ... existing fields ...
    
    // Video recording
    video_recorder: Option<VideoRecorder>,
    video_rx: Option<Receiver<VideoMessage>>,
    video_log: Vec<String>,
    video_status: String,
}
```

### Initialize in new()

```rust
impl AppState {
    fn new(cc: &eframe::CreationContext) -> Self {
        Self {
            // ... existing initialization ...
            
            video_recorder: None,
            video_rx: None,
            video_log: Vec::new(),
            video_status: "Not recording".to_string(),
        }
    }
}
```

### Add UI Panel

```rust
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process video messages
        if let Some(rx) = &self.video_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    VideoMessage::Log(text) => {
                        self.video_log.push(format!("[{}] {}", 
                            chrono::Local::now().format("%H:%M:%S"), text));
                    }
                    VideoMessage::Status(status) => {
                        self.video_status = status;
                    }
                    VideoMessage::RecordingStarted { camera, filename } => {
                        self.video_log.push(format!("Started: {} -> {}", camera, filename));
                    }
                    VideoMessage::RecordingStopped { camera, duration_sec } => {
                        self.video_log.push(format!("Stopped: {} ({}s)", camera, duration_sec));
                    }
                    VideoMessage::Error(err) => {
                        self.video_log.push(format!("ERROR: {}", err));
                    }
                    VideoMessage::FramesCaptured(count) => {
                        // Update progress
                    }
                }
            }
        }
        
        // Add video recording panel
        egui::Window::new("📹 Video Recording")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Camera Recording");
                
                ui.separator();
                
                // Status
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label(&self.video_status);
                });
                
                ui.separator();
                
                // Controls
                ui.horizontal(|ui| {
                    if ui.button("▶ Start Recording").clicked() {
                        self.start_video_recording();
                    }
                    
                    if ui.button("⏹ Stop Recording").clicked() {
                        self.stop_video_recording();
                    }
                });
                
                ui.separator();
                
                // Log
                ui.label("Activity Log:");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for msg in &self.video_log {
                            ui.label(msg);
                        }
                    });
            });
        
        ctx.request_repaint();
    }
}
```

### Add Helper Methods

```rust
impl AppState {
    fn start_video_recording(&mut self) {
        if self.video_recorder.is_some() {
            self.video_log.push("Already recording".to_string());
            return;
        }
        
        // Create message channel
        let (tx, rx) = channel();
        self.video_rx = Some(rx);
        
        // Create camera info (webcam 0)
        let camera = CameraInfo::new("Webcam", CameraSource::Webcam(0))
            .with_resolution(1920, 1080)
            .with_fps(30.0);
        
        // Create config
        let config = VideoConfig::new()
            .with_output_dir("recordings")
            .with_format(VideoFormat::MP4)
            .with_max_duration(3600); // 1 hour
        
        // Create recorder
        let mut recorder = VideoRecorder::new(camera, config)
            .with_gui_sender(tx);
        
        // Start recording
        match recorder.start_recording() {
            Ok(_) => {
                self.video_status = "Recording...".to_string();
                self.video_recorder = Some(recorder);
            }
            Err(e) => {
                self.video_log.push(format!("Failed to start: {}", e));
            }
        }
    }
    
    fn stop_video_recording(&mut self) {
        if let Some(mut recorder) = self.video_recorder.take() {
            match recorder.stop_recording() {
                Ok(_) => {
                    self.video_status = "Stopped".to_string();
                }
                Err(e) => {
                    self.video_log.push(format!("Error stopping: {}", e));
                }
            }
        }
        self.video_recorder = None;
    }
}
```

---

## Architecture Integration

The video recording module fits perfectly into SecCamCloud's modular architecture:

```
src/
├── lib.rs          (declares vidrec module)
├── main.rs         (GUI with video controls)
├── config.rs       (configuration)
├── automation.rs   (automation)
├── watchdog.rs     (safety)
├── telemetry.rs    (logging)
├── screenshot.rs   (screenshots)
└── vidrec.rs       (video recording) ← NEW!
```

**Benefits:**
- ✅ Follows existing module pattern
- ✅ Same GPLv2 license
- ✅ Consistent code style
- ✅ Same author and versioning
- ✅ Independent and reusable
- ✅ Optional feature (doesn't bloat core)

---

## Features Comparison

| Feature | Automation | Screenshots | Video Recording |
|---------|-----------|-------------|-----------------|
| **Purpose** | Click automation | Still images | Video capture |
| **Cargo Feature** | (default) | `screenshots` | `video` |
| **Dependencies** | enigo | scrap, image | opencv |
| **Output** | N/A | PNG files | MP4/AVI/MKV |
| **Multi-source** | No | No | Yes (webcam, IP cam) |
| **Background Thread** | Yes | No | Yes |
| **GUI Messages** | Yes | No | Yes |

---

## Common Issues & Solutions

### Issue: OpenCV not found

**Error:**
```
Could not find OpenCV. Please install OpenCV and set OPENCV_DIR
```

**Solutions:**
1. Install OpenCV for your platform (see step 1)
2. Set environment variable:
   - Windows: `set OPENCV_DIR=C:\opencv\build`
   - Linux/Mac: Usually auto-detected

### Issue: Camera won't open

**Error:**
```
Failed to open camera source
```

**Solutions:**
1. Check camera index (try 0, 1, 2 for different cameras)
2. Verify no other application is using the camera
3. For IP cameras, test URL in VLC first
4. Check camera permissions (Linux: add user to video group)

### Issue: Build takes forever

**Reason:** OpenCV is a large library

**Solutions:**
1. Use `default-features = false` in Cargo.toml (already done)
2. Only build with video feature when needed
3. First build will be slow, subsequent builds are fast
4. Use `cargo build --release` for faster runtime

### Issue: Video file not created

**Solutions:**
1. Check output directory exists and is writable
2. Verify sufficient disk space
3. Check logs for error messages
4. Ensure recording was actually started

---

## Performance Tips

### Optimal Settings for Different Use Cases

**Security Camera (24/7 recording):**
```rust
CameraInfo::new("Security", source)
    .with_resolution(1280, 720)   // 720p sufficient
    .with_fps(15.0);               // 15fps for storage savings

VideoConfig::new()
    .with_format(VideoFormat::MP4)
    .with_max_duration(1800)       // 30 min segments
    .with_auto_restart(true);      // Continuous recording
```

**High Quality Recording:**
```rust
CameraInfo::new("HQ Camera", source)
    .with_resolution(1920, 1080)   // Full HD
    .with_fps(30.0);               // Smooth motion

VideoConfig::new()
    .with_format(VideoFormat::MKV)
    .with_max_duration(3600);      // 1 hour segments
```

**Low Resource Recording:**
```rust
CameraInfo::new("Low Power", source)
    .with_resolution(640, 480)     // VGA
    .with_fps(10.0);               // Minimal fps

VideoConfig::new()
    .with_format(VideoFormat::AVI);
```

---

## Next Steps

1. ✅ Install OpenCV
2. ✅ Copy files to project
3. ✅ Build with video feature
4. ✅ Test with webcam
5. ✅ Test with IP camera (if available)
6. ✅ Integrate into GUI
7. ✅ Configure for your needs

---

## Resources

- **OpenCV Documentation:** https://docs.opencv.org/
- **RTSP Testing:** Use VLC Media Player
- **opencv-rust crate:** https://github.com/twistedfall/opencv-rust

---

## Support

If you encounter issues:
1. Check VIDEO_RECORDING.md for detailed API docs
2. Review troubleshooting section above
3. Test camera with VLC first
4. Check application logs
5. Verify OpenCV installation

---

**Your video recording module is ready to use!** 🎥
