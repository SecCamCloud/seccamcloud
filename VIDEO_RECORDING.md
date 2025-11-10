# SecCamCloud - Video Recording Guide

## Overview

The `vidrec` module provides comprehensive video recording capabilities for SecCamCloud using OpenCV. It supports multiple camera sources including webcams, IP cameras (RTSP/HTTP), and video files.

**Version:** 1.0.0  
**Author:** Michael Lauzon  
**License:** GPLv2

---

## Features

✅ **Multiple Camera Sources**
- Local webcams (USB, built-in)
- IP cameras via RTSP streams
- HTTP/MJPEG streams
- Video files (for testing)

✅ **Professional Recording**
- Configurable resolution and frame rate
- Multiple output formats (MP4, AVI, MKV)
- Automatic file management
- Duration and file size limits

✅ **Multi-Camera Support**
- Record from multiple cameras simultaneously
- Individual configuration per camera
- Centralized management

✅ **Safety Features**
- Graceful shutdown handling
- Automatic cleanup
- Error recovery
- Thread-safe operations

---

## Installation

### 1. Install OpenCV

**Windows:**
```bash
# Download OpenCV from opencv.org
# Or use vcpkg:
vcpkg install opencv4[core,videoio,highgui,imgcodecs]:x64-windows
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install libopencv-dev clang libclang-dev
```

**macOS:**
```bash
brew install opencv
```

### 2. Build SecCamCloud with Video Support

```bash
# Build with video feature
cargo build --release --features video

# Or with all features
cargo build --release --features "screenshots,video"
```

---

## Quick Start

### Basic Webcam Recording

```rust
use seccamcloud::{CameraInfo, CameraSource, VideoConfig, VideoRecorder};

fn main() {
    // Create camera info for default webcam
    let camera = CameraInfo::new("My Webcam", CameraSource::Webcam(0))
        .with_resolution(1920, 1080)
        .with_fps(30.0);
    
    // Create configuration
    let config = VideoConfig::new()
        .with_output_dir("videos")
        .with_max_duration(3600); // 1 hour
    
    // Create recorder
    let mut recorder = VideoRecorder::new(camera, config);
    
    // Start recording
    recorder.start_recording().unwrap();
    
    // ... do other work ...
    
    // Stop recording
    recorder.stop_recording().unwrap();
}
```

### IP Camera Recording (RTSP)

```rust
use seccamcloud::{CameraInfo, CameraSource, VideoConfig, VideoRecorder};

fn main() {
    // RTSP camera
    let camera = CameraInfo::new(
        "Front Door Camera",
        CameraSource::RtspStream("rtsp://192.168.1.100:554/stream".to_string())
    )
    .with_resolution(1920, 1080)
    .with_fps(25.0);
    
    let config = VideoConfig::new()
        .with_output_dir("recordings")
        .with_format(VideoFormat::MP4);
    
    let mut recorder = VideoRecorder::new(camera, config);
    recorder.start_recording().unwrap();
    
    // Recording runs in background thread
    std::thread::sleep(std::time::Duration::from_secs(60));
    
    recorder.stop_recording().unwrap();
}
```

### Multi-Camera Recording

```rust
use seccamcloud::{
    CameraInfo, CameraSource, VideoConfig, VideoFormat,
    MultiCameraRecorder
};

fn main() {
    let mut manager = MultiCameraRecorder::new();
    
    // Add webcam
    let webcam = CameraInfo::new("Webcam", CameraSource::Webcam(0));
    let config1 = VideoConfig::new().with_output_dir("recordings/webcam");
    manager.add_camera(webcam, config1);
    
    // Add IP camera
    let ipcam = CameraInfo::new(
        "IP Camera",
        CameraSource::RtspStream("rtsp://192.168.1.100:554/stream".to_string())
    );
    let config2 = VideoConfig::new().with_output_dir("recordings/ipcam");
    manager.add_camera(ipcam, config2);
    
    // Start all cameras
    manager.start_all().unwrap();
    
    println!("Recording from {} cameras", manager.recording_count());
    
    // ... wait ...
    
    // Stop all cameras
    manager.stop_all().unwrap();
}
```

---

## API Reference

### CameraSource

Defines the source of video input:

```rust
pub enum CameraSource {
    /// Local webcam by index (0 = default)
    Webcam(i32),
    
    /// IP camera RTSP stream
    RtspStream(String),
    
    /// HTTP/MJPEG stream
    HttpStream(String),
    
    /// Video file path (for testing)
    VideoFile(String),
}
```

**Examples:**
```rust
// Default webcam
CameraSource::Webcam(0)

// Second webcam
CameraSource::Webcam(1)

// RTSP IP camera
CameraSource::RtspStream("rtsp://admin:password@192.168.1.100:554/stream1".to_string())

// HTTP MJPEG stream
CameraSource::HttpStream("http://192.168.1.100:8080/video".to_string())

// Video file (for testing)
CameraSource::VideoFile("/path/to/test.mp4".to_string())
```

### CameraInfo

Camera configuration:

```rust
pub struct CameraInfo {
    pub name: String,
    pub source: CameraSource,
    pub width: i32,
    pub height: i32,
    pub fps: f64,
}
```

**Methods:**
```rust
// Create with defaults (1920x1080 @ 30fps)
let camera = CameraInfo::new("Camera Name", source);

// Set custom resolution
let camera = camera.with_resolution(1280, 720);

// Set custom frame rate
let camera = camera.with_fps(60.0);

// Chain methods
let camera = CameraInfo::new("HD Camera", source)
    .with_resolution(1920, 1080)
    .with_fps(30.0);
```

### VideoFormat

Output video format:

```rust
pub enum VideoFormat {
    MP4,   // H.264 in MP4 container
    AVI,   // MJPEG in AVI container
    MKV,   // H.264 in MKV container
}
```

**Codec Details:**
- **MP4:** Uses H.264 codec (mp4v fourcc) - Best compatibility
- **AVI:** Uses MJPEG codec - Larger files but simpler
- **MKV:** Uses H.264 codec (x264 fourcc) - Good compression

### VideoConfig

Recording configuration:

```rust
pub struct VideoConfig {
    pub output_dir: PathBuf,
    pub format: VideoFormat,
    pub max_duration_sec: Option<u64>,
    pub max_file_size_mb: Option<u64>,
    pub auto_restart: bool,
}
```

**Methods:**
```rust
// Create with defaults
let config = VideoConfig::new();

// Set output directory
let config = config.with_output_dir("my_recordings");

// Set video format
let config = config.with_format(VideoFormat::MP4);

// Set max duration (in seconds)
let config = config.with_max_duration(3600); // 1 hour

// Set max file size (in megabytes)
let config = config.with_max_file_size(2048); // 2GB

// Enable/disable auto restart when limits reached
let config = config.with_auto_restart(true);

// Chain methods
let config = VideoConfig::new()
    .with_output_dir("recordings")
    .with_format(VideoFormat::MP4)
    .with_max_duration(3600)
    .with_max_file_size(2048);
```

### VideoRecorder

Main recording interface:

```rust
pub struct VideoRecorder { /* ... */ }
```

**Methods:**
```rust
// Create recorder
let recorder = VideoRecorder::new(camera_info, config);

// Connect to GUI (optional)
let recorder = recorder.with_gui_sender(tx);

// Start recording (non-blocking)
recorder.start_recording()?;

// Check if recording
if recorder.is_recording() {
    println!("Recording in progress");
}

// Stop recording (blocks until stopped)
recorder.stop_recording()?;
```

### VideoMessage

Messages sent to GUI:

```rust
pub enum VideoMessage {
    Log(String),
    Status(String),
    RecordingStarted { camera: String, filename: String },
    RecordingStopped { camera: String, duration_sec: u64 },
    Error(String),
    FramesCaptured(u64),
}
```

### MultiCameraRecorder

Manage multiple cameras:

```rust
pub struct MultiCameraRecorder { /* ... */ }
```

**Methods:**
```rust
// Create manager
let mut manager = MultiCameraRecorder::new();

// Connect to GUI (optional)
let manager = manager.with_gui_sender(tx);

// Add cameras
manager.add_camera(camera_info, config);

// Start all cameras
manager.start_all()?;

// Get recording count
let count = manager.recording_count();

// Stop all cameras
manager.stop_all()?;
```

---

## Common Camera URLs

### RTSP URL Formats

**Generic IP Camera:**
```
rtsp://username:password@ip:port/stream
rtsp://192.168.1.100:554/stream1
```

**Hikvision:**
```
rtsp://admin:password@192.168.1.100:554/Streaming/Channels/101
```

**Dahua:**
```
rtsp://admin:password@192.168.1.100:554/cam/realmonitor?channel=1&subtype=0
```

**Axis:**
```
rtsp://root:password@192.168.1.100:554/axis-media/media.amp
```

**Foscam:**
```
rtsp://username:password@192.168.1.100:88/videoMain
```

**Amcrest:**
```
rtsp://admin:password@192.168.1.100:554/cam/realmonitor?channel=1&subtype=0
```

### HTTP MJPEG URL Formats

**Generic:**
```
http://192.168.1.100:8080/video
```

**Axis:**
```
http://root:password@192.168.1.100/mjpg/video.mjpg
```

---

## Integration with GUI

### Setup Message Channel

```rust
use std::sync::mpsc::channel;
use seccamcloud::{VideoMessage, VideoRecorder, CameraInfo, VideoConfig};

// Create channel
let (tx, rx) = channel::<VideoMessage>();

// Create recorder with GUI sender
let mut recorder = VideoRecorder::new(camera_info, config)
    .with_gui_sender(tx);

// In GUI thread, receive messages
while let Ok(msg) = rx.recv() {
    match msg {
        VideoMessage::Log(text) => {
            println!("[LOG] {}", text);
        }
        VideoMessage::Status(status) => {
            // Update status label
        }
        VideoMessage::RecordingStarted { camera, filename } => {
            println!("Started: {} -> {}", camera, filename);
        }
        VideoMessage::RecordingStopped { camera, duration_sec } => {
            println!("Stopped: {} ({}s)", camera, duration_sec);
        }
        VideoMessage::Error(err) => {
            eprintln!("Error: {}", err);
        }
        VideoMessage::FramesCaptured(count) => {
            // Update progress indicator
        }
    }
}
```

### egui Integration Example

```rust
use eframe::egui;
use seccamcloud::{VideoRecorder, VideoMessage};
use std::sync::mpsc::{channel, Receiver};

struct VideoApp {
    recorder: Option<VideoRecorder>,
    rx: Option<Receiver<VideoMessage>>,
    log_messages: Vec<String>,
    status: String,
}

impl eframe::App for VideoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process video messages
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    VideoMessage::Log(text) => {
                        self.log_messages.push(text);
                    }
                    VideoMessage::Status(status) => {
                        self.status = status;
                    }
                    _ => {}
                }
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Video Recording");
            
            ui.label(&self.status);
            
            if ui.button("Start Recording").clicked() {
                if let Some(recorder) = &mut self.recorder {
                    let _ = recorder.start_recording();
                }
            }
            
            if ui.button("Stop Recording").clicked() {
                if let Some(recorder) = &mut self.recorder {
                    let _ = recorder.stop_recording();
                }
            }
            
            // Display log
            egui::ScrollArea::vertical().show(ui, |ui| {
                for msg in &self.log_messages {
                    ui.label(msg);
                }
            });
        });
        
        ctx.request_repaint();
    }
}
```

---

## Troubleshooting

### OpenCV Not Found

**Error:** `Could not find OpenCV`

**Solution:**
- **Windows:** Install OpenCV and set `OPENCV_DIR` environment variable
- **Linux:** `sudo apt-get install libopencv-dev`
- **macOS:** `brew install opencv`

### Camera Won't Open

**Error:** `Failed to open camera source`

**Solutions:**
1. Check camera index (try 0, 1, 2)
2. Verify RTSP URL is correct
3. Check network connectivity to IP camera
4. Verify camera credentials
5. Try VLC to test the stream first

### Poor Video Quality

**Solutions:**
1. Increase resolution: `.with_resolution(1920, 1080)`
2. Increase frame rate: `.with_fps(30.0)`
3. Use MP4 format instead of AVI
4. Check camera's native resolution

### High CPU Usage

**Solutions:**
1. Reduce frame rate: `.with_fps(15.0)`
2. Reduce resolution: `.with_resolution(1280, 720)`
3. Use hardware acceleration (if available)
4. Record fewer cameras simultaneously

### File Size Too Large

**Solutions:**
1. Set file size limit: `.with_max_file_size(1024)` (1GB)
2. Set duration limit: `.with_max_duration(1800)` (30 min)
3. Use MP4 format (better compression)
4. Reduce resolution or frame rate

---

## Performance Considerations

### Resource Usage

**Per Camera (1080p @ 30fps):**
- CPU: 5-15%
- Memory: 50-100MB
- Disk Write: ~200MB/minute (MP4)

**Recommendations:**
- Max 4 cameras on typical desktop
- Max 2 cameras on laptop
- Use lower resolution for more cameras
- Use SSD for better write performance

### Optimization Tips

1. **Lower Resolution:**
   ```rust
   .with_resolution(1280, 720)  // Instead of 1920x1080
   ```

2. **Lower Frame Rate:**
   ```rust
   .with_fps(15.0)  // Instead of 30.0
   ```

3. **Use MP4 Format:**
   ```rust
   .with_format(VideoFormat::MP4)  // Better compression than AVI
   ```

4. **Set Limits:**
   ```rust
   .with_max_duration(1800)  // 30 minutes
   .with_max_file_size(1024)  // 1GB
   ```

---

## Examples

### Example 1: Basic Webcam Recording

```rust
use seccamcloud::*;
use std::time::Duration;
use std::thread;

fn main() {
    let camera = CameraInfo::new("Webcam", CameraSource::Webcam(0));
    let config = VideoConfig::new();
    
    let mut recorder = VideoRecorder::new(camera, config);
    
    println!("Starting recording...");
    recorder.start_recording().unwrap();
    
    thread::sleep(Duration::from_secs(30));
    
    println!("Stopping recording...");
    recorder.stop_recording().unwrap();
    
    println!("Done!");
}
```

### Example 2: IP Camera with Limits

```rust
use seccamcloud::*;

fn main() {
    let camera = CameraInfo::new(
        "Front Door",
        CameraSource::RtspStream("rtsp://192.168.1.100:554/stream".to_string())
    );
    
    let config = VideoConfig::new()
        .with_output_dir("recordings")
        .with_format(VideoFormat::MP4)
        .with_max_duration(3600)      // 1 hour max
        .with_max_file_size(2048)     // 2GB max
        .with_auto_restart(true);     // Auto-restart when limit hit
    
    let mut recorder = VideoRecorder::new(camera, config);
    recorder.start_recording().unwrap();
    
    // Recording continues until stopped or limits reached
    std::thread::park();
}
```

### Example 3: Multi-Camera Security System

```rust
use seccamcloud::*;
use std::sync::mpsc::channel;

fn main() {
    let (tx, rx) = channel();
    
    let mut manager = MultiCameraRecorder::new()
        .with_gui_sender(tx);
    
    // Front door
    manager.add_camera(
        CameraInfo::new("Front Door", CameraSource::RtspStream(
            "rtsp://192.168.1.100:554/stream".to_string()
        )),
        VideoConfig::new().with_output_dir("recordings/front")
    );
    
    // Back door
    manager.add_camera(
        CameraInfo::new("Back Door", CameraSource::RtspStream(
            "rtsp://192.168.1.101:554/stream".to_string()
        )),
        VideoConfig::new().with_output_dir("recordings/back")
    );
    
    // Garage
    manager.add_camera(
        CameraInfo::new("Garage", CameraSource::Webcam(0)),
        VideoConfig::new().with_output_dir("recordings/garage")
    );
    
    println!("Starting all cameras...");
    manager.start_all().unwrap();
    
    // Monitor messages
    loop {
        match rx.recv() {
            Ok(VideoMessage::RecordingStarted { camera, filename }) => {
                println!("[{}] Recording to: {}", camera, filename);
            }
            Ok(VideoMessage::Error(err)) => {
                eprintln!("Error: {}", err);
            }
            _ => {}
        }
    }
}
```

---

## License

This module is part of SecCamCloud and is licensed under GPLv2.

---

## Support

For issues with video recording:
1. Check OpenCV installation
2. Verify camera URLs and credentials
3. Test camera with VLC first
4. Check the logs for error messages
5. Review the troubleshooting section

---

**Happy Recording!** 🎥
