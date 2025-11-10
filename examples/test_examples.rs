// ============================================================================
// SecCamCloud - Video Recording Test Examples
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

// These examples show how to use the vidrec module
// Save these in your project's examples/ directory

// ============================================================================
// EXAMPLE 1: Basic Webcam Recording (examples/test_webcam.rs)
// ============================================================================

/*
use seccamcloud::{CameraInfo, CameraSource, VideoConfig, VideoRecorder, VideoFormat};
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== SecCamCloud Webcam Test ===\n");
    
    // Create camera info for default webcam
    let camera = CameraInfo::new("Test Webcam", CameraSource::Webcam(0))
        .with_resolution(1280, 720)
        .with_fps(30.0);
    
    println!("Camera: {}", camera.name);
    println!("Source: {:?}", camera.source);
    println!("Resolution: {}x{}", camera.width, camera.height);
    println!("FPS: {}\n", camera.fps);
    
    // Create configuration
    let config = VideoConfig::new()
        .with_output_dir("test_recordings")
        .with_format(VideoFormat::MP4);
    
    println!("Output: {}", config.output_dir.display());
    println!("Format: {:?}\n", config.format);
    
    // Create recorder
    let mut recorder = VideoRecorder::new(camera, config);
    
    // Start recording
    println!("Starting 10 second recording...");
    match recorder.start_recording() {
        Ok(_) => println!("Recording started successfully!"),
        Err(e) => {
            eprintln!("Failed to start recording: {}", e);
            return;
        }
    }
    
    // Show countdown
    for i in (1..=10).rev() {
        println!("Recording... {}s remaining", i);
        thread::sleep(Duration::from_secs(1));
    }
    
    // Stop recording
    println!("\nStopping recording...");
    match recorder.stop_recording() {
        Ok(_) => println!("Recording stopped successfully!"),
        Err(e) => eprintln!("Error stopping: {}", e),
    }
    
    println!("\n=== Test Complete ===");
    println!("Check test_recordings/ directory for output file");
}
*/

// To run this example:
// cargo run --release --features video --example test_webcam

// ============================================================================
// EXAMPLE 2: IP Camera Recording (examples/test_ipcam.rs)
// ============================================================================

/*
use seccamcloud::{CameraInfo, CameraSource, VideoConfig, VideoRecorder, VideoFormat};
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== SecCamCloud IP Camera Test ===\n");
    
    // IMPORTANT: Replace with your camera's actual RTSP URL
    let rtsp_url = "rtsp://admin:password@192.168.1.100:554/stream";
    
    println!("Connecting to: {}\n", rtsp_url);
    
    // Create camera info
    let camera = CameraInfo::new(
        "Test IP Camera",
        CameraSource::RtspStream(rtsp_url.to_string())
    )
    .with_resolution(1920, 1080)
    .with_fps(25.0);
    
    // Create configuration
    let config = VideoConfig::new()
        .with_output_dir("ipcam_recordings")
        .with_format(VideoFormat::MP4)
        .with_max_duration(30); // 30 second test
    
    // Create recorder
    let mut recorder = VideoRecorder::new(camera, config);
    
    // Start recording
    println!("Starting recording...");
    match recorder.start_recording() {
        Ok(_) => println!("Recording started!"),
        Err(e) => {
            eprintln!("Failed to start: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Check RTSP URL is correct");
            eprintln!("2. Verify camera is accessible on network");
            eprintln!("3. Test URL in VLC first");
            return;
        }
    }
    
    // Recording will auto-stop after 30 seconds (max_duration)
    println!("Recording for 30 seconds...");
    println!("(Will auto-stop at max duration)");
    
    thread::sleep(Duration::from_secs(35));
    
    println!("\n=== Test Complete ===");
    println!("Check ipcam_recordings/ directory");
}
*/

// To run this example:
// cargo run --release --features video --example test_ipcam

// ============================================================================
// EXAMPLE 3: Multi-Camera Recording (examples/test_multicam.rs)
// ============================================================================

/*
use seccamcloud::{
    CameraInfo, CameraSource, VideoConfig, VideoFormat,
    MultiCameraRecorder, VideoMessage
};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== SecCamCloud Multi-Camera Test ===\n");
    
    // Create message channel
    let (tx, rx) = channel();
    
    // Create multi-camera manager
    let mut manager = MultiCameraRecorder::new()
        .with_gui_sender(tx);
    
    // Add webcam
    println!("Adding webcam...");
    let webcam = CameraInfo::new("Webcam", CameraSource::Webcam(0))
        .with_resolution(1280, 720);
    let config1 = VideoConfig::new()
        .with_output_dir("multi_recordings/webcam")
        .with_format(VideoFormat::MP4);
    manager.add_camera(webcam, config1);
    
    // Add second webcam (if available)
    println!("Adding second webcam (if available)...");
    let webcam2 = CameraInfo::new("Webcam 2", CameraSource::Webcam(1))
        .with_resolution(640, 480);
    let config2 = VideoConfig::new()
        .with_output_dir("multi_recordings/webcam2")
        .with_format(VideoFormat::AVI);
    manager.add_camera(webcam2, config2);
    
    // Note: Add IP cameras here if you have them
    // let ipcam = CameraInfo::new("IP Cam", CameraSource::RtspStream(...));
    // manager.add_camera(ipcam, config3);
    
    println!("\nStarting all cameras...");
    match manager.start_all() {
        Ok(_) => println!("All cameras started!"),
        Err(e) => {
            eprintln!("Error: {}", e);
            println!("Note: It's OK if second camera fails (might not exist)");
        }
    }
    
    println!("Recording from {} cameras", manager.recording_count());
    println!("\nMonitoring for 20 seconds...\n");
    
    // Monitor messages for 20 seconds
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        match rx.try_recv() {
            Ok(VideoMessage::RecordingStarted { camera, filename }) => {
                println!("[{}] Recording to: {}", camera, filename);
            }
            Ok(VideoMessage::Log(msg)) => {
                println!("[LOG] {}", msg);
            }
            Ok(VideoMessage::Error(err)) => {
                eprintln!("[ERROR] {}", err);
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("\nStopping all cameras...");
    manager.stop_all().unwrap();
    
    println!("\n=== Test Complete ===");
    println!("Check multi_recordings/ directory");
}
*/

// To run this example:
// cargo run --release --features video --example test_multicam

// ============================================================================
// EXAMPLE 4: Recording with GUI Messages (examples/test_messages.rs)
// ============================================================================

/*
use seccamcloud::{
    CameraInfo, CameraSource, VideoConfig, VideoRecorder,
    VideoMessage
};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== SecCamCloud Message Test ===\n");
    
    // Create message channel
    let (tx, rx) = channel();
    
    // Create camera and config
    let camera = CameraInfo::new("Message Test Cam", CameraSource::Webcam(0));
    let config = VideoConfig::new();
    
    // Create recorder with message sender
    let mut recorder = VideoRecorder::new(camera, config)
        .with_gui_sender(tx);
    
    println!("Starting recording with message monitoring...\n");
    recorder.start_recording().unwrap();
    
    // Monitor messages for 15 seconds
    let start = std::time::Instant::now();
    let mut frame_count = 0u64;
    
    while start.elapsed() < Duration::from_secs(15) {
        match rx.try_recv() {
            Ok(VideoMessage::Log(msg)) => {
                println!("[LOG] {}", msg);
            }
            Ok(VideoMessage::Status(status)) => {
                println!("[STATUS] {}", status);
            }
            Ok(VideoMessage::RecordingStarted { camera, filename }) => {
                println!("[START] Camera: {}", camera);
                println!("        File: {}", filename);
            }
            Ok(VideoMessage::RecordingStopped { camera, duration_sec }) => {
                println!("[STOP] Camera: {}", camera);
                println!("       Duration: {}s", duration_sec);
            }
            Ok(VideoMessage::Error(err)) => {
                eprintln!("[ERROR] {}", err);
            }
            Ok(VideoMessage::FramesCaptured(count)) => {
                frame_count = count;
                print!("\r[PROGRESS] Frames captured: {}", count);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
    
    println!("\n\nStopping recording...");
    recorder.stop_recording().unwrap();
    
    println!("\n=== Test Complete ===");
    println!("Total frames captured: {}", frame_count);
}
*/

// To run this example:
// cargo run --release --features video --example test_messages

// ============================================================================
// EXAMPLE 5: Long Duration Test with Limits (examples/test_limits.rs)
// ============================================================================

/*
use seccamcloud::{
    CameraInfo, CameraSource, VideoConfig, VideoFormat,
    VideoRecorder, VideoMessage
};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== SecCamCloud Limits Test ===\n");
    
    let (tx, rx) = channel();
    
    // Create camera
    let camera = CameraInfo::new("Limits Test", CameraSource::Webcam(0))
        .with_resolution(1920, 1080)
        .with_fps(30.0);
    
    // Create config with limits
    let config = VideoConfig::new()
        .with_output_dir("limit_test")
        .with_format(VideoFormat::MP4)
        .with_max_duration(60)      // 1 minute max
        .with_max_file_size(50)     // 50MB max (will hit duration first)
        .with_auto_restart(true);   // Auto-restart when limit hit
    
    println!("Testing automatic limits:");
    println!("- Max duration: 60 seconds");
    println!("- Max file size: 50MB");
    println!("- Auto-restart: enabled\n");
    
    let mut recorder = VideoRecorder::new(camera, config)
        .with_gui_sender(tx);
    
    println!("Starting recording...");
    recorder.start_recording().unwrap();
    
    // Monitor for 2 minutes to see auto-restart
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(120) {
        match rx.try_recv() {
            Ok(VideoMessage::RecordingStarted { camera, filename }) => {
                println!("\n[{}] New recording: {}", camera, filename);
            }
            Ok(VideoMessage::RecordingStopped { camera, duration_sec }) => {
                println!("\n[{}] Stopped after {}s (limit reached)", camera, duration_sec);
                println!("Auto-restarting...");
            }
            Ok(VideoMessage::Log(msg)) => {
                if msg.contains("Max duration") || msg.contains("limit") {
                    println!("[LIMIT] {}", msg);
                }
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("\n\nStopping recording...");
    recorder.stop_recording().unwrap();
    
    println!("\n=== Test Complete ===");
    println!("Check limit_test/ directory for multiple segments");
}
*/

// To run this example:
// cargo run --release --features video --example test_limits

// ============================================================================
// NOTES
// ============================================================================

// To use these examples:
//
// 1. Create examples/ directory in your project root
// 2. Save each example as a separate .rs file in examples/
// 3. Uncomment the example code you want to use
// 4. Run with: cargo run --release --features video --example <name>
//
// Example file structure:
// your_project/
// ├── src/
// │   ├── main.rs
// │   ├── vidrec.rs
// │   └── ...
// └── examples/
//     ├── test_webcam.rs
//     ├── test_ipcam.rs
//     ├── test_multicam.rs
//     ├── test_messages.rs
//     └── test_limits.rs

// ============================================================================
// COMMON CAMERA URLS FOR TESTING
// ============================================================================

// Hikvision:
// rtsp://admin:password@192.168.1.100:554/Streaming/Channels/101

// Dahua:
// rtsp://admin:password@192.168.1.100:554/cam/realmonitor?channel=1&subtype=0

// Axis:
// rtsp://root:password@192.168.1.100:554/axis-media/media.amp

// Foscam:
// rtsp://username:password@192.168.1.100:88/videoMain

// Generic RTSP:
// rtsp://username:password@ip:port/stream

// HTTP MJPEG:
// http://192.168.1.100:8080/video

// ============================================================================
