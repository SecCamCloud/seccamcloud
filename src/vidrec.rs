// ============================================================================
// SecCamCloud - Video Recording Module
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};

use chrono::Local;
use log::{info, error, warn};

#[cfg(feature = "video")]
use opencv::{
    prelude::*,
    videoio::{self, VideoCapture, VideoWriter, CAP_ANY},
    core::{Size, Vector},
    Result as CvResult,
};

// ============================================================================
// CONSTANTS
// ============================================================================

const DEFAULT_FPS: f64 = 30.0;
const DEFAULT_WIDTH: i32 = 1920;
const DEFAULT_HEIGHT: i32 = 1080;
const DEFAULT_OUTPUT_DIR: &str = "recordings";

// ============================================================================
// VIDEO FORMATS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    MP4,
    AVI,
    MKV,
}

impl VideoFormat {
    pub fn extension(&self) -> &str {
        match self {
            VideoFormat::MP4 => "mp4",
            VideoFormat::AVI => "avi",
            VideoFormat::MKV => "mkv",
        }
    }

    #[cfg(feature = "video")]
    pub fn fourcc(&self) -> i32 {
        match self {
            VideoFormat::MP4 => VideoWriter::fourcc('m' as i8, 'p' as i8, '4' as i8, 'v' as i8).unwrap(),
            VideoFormat::AVI => VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap(),
            VideoFormat::MKV => VideoWriter::fourcc('X' as i8, '2' as i8, '6' as i8, '4' as i8).unwrap(),
        }
    }

    #[cfg(not(feature = "video"))]
    pub fn fourcc(&self) -> i32 {
        0
    }
}

// ============================================================================
// CAMERA TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
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

impl CameraSource {
    #[cfg(feature = "video")]
    fn to_opencv_string(&self) -> String {
        match self {
            CameraSource::Webcam(idx) => idx.to_string(),
            CameraSource::RtspStream(url) => url.clone(),
            CameraSource::HttpStream(url) => url.clone(),
            CameraSource::VideoFile(path) => path.clone(),
        }
    }

    pub fn source_type(&self) -> &str {
        match self {
            CameraSource::Webcam(_) => "Webcam",
            CameraSource::RtspStream(_) => "RTSP",
            CameraSource::HttpStream(_) => "HTTP",
            CameraSource::VideoFile(_) => "File",
        }
    }
}

// ============================================================================
// CAMERA INFO
// ============================================================================

#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub name: String,
    pub source: CameraSource,
    pub width: i32,
    pub height: i32,
    pub fps: f64,
}

impl CameraInfo {
    pub fn new(name: impl Into<String>, source: CameraSource) -> Self {
        Self {
            name: name.into(),
            source,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            fps: DEFAULT_FPS,
        }
    }

    pub fn with_resolution(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_fps(mut self, fps: f64) -> Self {
        self.fps = fps;
        self
    }
}

// ============================================================================
// VIDEO CONFIGURATION
// ============================================================================

#[derive(Debug, Clone)]
pub struct VideoConfig {
    pub output_dir: PathBuf,
    pub format: VideoFormat,
    pub max_duration_sec: Option<u64>,
    pub max_file_size_mb: Option<u64>,
    pub auto_restart: bool,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR),
            format: VideoFormat::MP4,
            max_duration_sec: Some(3600), // 1 hour
            max_file_size_mb: Some(2048), // 2GB
            auto_restart: true,
        }
    }
}

impl VideoConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    pub fn with_format(mut self, format: VideoFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_max_duration(mut self, seconds: u64) -> Self {
        self.max_duration_sec = Some(seconds);
        self
    }

    pub fn with_max_file_size(mut self, megabytes: u64) -> Self {
        self.max_file_size_mb = Some(megabytes);
        self
    }

    pub fn with_auto_restart(mut self, restart: bool) -> Self {
        self.auto_restart = restart;
        self
    }
}

// ============================================================================
// VIDEO MESSAGES
// ============================================================================

#[derive(Debug, Clone)]
pub enum VideoMessage {
    Log(String),
    Status(String),
    RecordingStarted { camera: String, filename: String },
    RecordingStopped { camera: String, duration_sec: u64 },
    Error(String),
    FramesCaptured(u64),
}

// ============================================================================
// RECORDING STATE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordingState {
    Idle,
    Recording,
    Stopping,
    Error,
}

// ============================================================================
// VIDEO RECORDER
// ============================================================================

pub struct VideoRecorder {
    camera_info: CameraInfo,
    config: VideoConfig,
    state: Arc<Mutex<RecordingState>>,
    tx_to_gui: Option<Sender<VideoMessage>>,
    thread_handle: Option<JoinHandle<()>>,
    stop_tx: Option<Sender<()>>,
}

impl VideoRecorder {
    /// Create a new video recorder
    pub fn new(camera_info: CameraInfo, config: VideoConfig) -> Self {
        // Ensure output directory exists
        if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
            error!("Failed to create output directory: {}", e);
        }

        Self {
            camera_info,
            config,
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            tx_to_gui: None,
            thread_handle: None,
            stop_tx: None,
        }
    }

    /// Set GUI message sender
    pub fn with_gui_sender(mut self, tx: Sender<VideoMessage>) -> Self {
        self.tx_to_gui = Some(tx);
        self
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        *self.state.lock().unwrap() == RecordingState::Recording
    }

    /// Get current state
    pub fn get_state(&self) -> RecordingState {
        *self.state.lock().unwrap()
    }

    /// Start recording
    pub fn start_recording(&mut self) -> Result<(), String> {
        // Check if already recording
        {
            let state = self.state.lock().unwrap();
            if *state == RecordingState::Recording {
                return Err("Already recording".to_string());
            }
        }

        #[cfg(not(feature = "video"))]
        {
            return Err("Video recording feature not enabled. Build with --features video".to_string());
        }

        #[cfg(feature = "video")]
        {
            info!("Starting recording for camera: {}", self.camera_info.name);
            self.send_message(VideoMessage::Log(format!(
                "Starting recording: {} ({})",
                self.camera_info.name,
                self.camera_info.source.source_type()
            )));

            // Create stop channel
            let (stop_tx, stop_rx) = channel();
            self.stop_tx = Some(stop_tx);

            // Clone data for thread
            let camera_info = self.camera_info.clone();
            let config = self.config.clone();
            let state = self.state.clone();
            let tx_gui = self.tx_to_gui.clone();

            // Update state
            *self.state.lock().unwrap() = RecordingState::Recording;

            // Spawn recording thread
            let handle = thread::spawn(move || {
                Self::recording_thread(camera_info, config, state, tx_gui, stop_rx);
            });

            self.thread_handle = Some(handle);

            Ok(())
        }
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Result<(), String> {
        let current_state = *self.state.lock().unwrap();
        
        if current_state != RecordingState::Recording {
            return Err("Not currently recording".to_string());
        }

        info!("Stopping recording for camera: {}", self.camera_info.name);
        self.send_message(VideoMessage::Log(format!(
            "Stopping recording: {}",
            self.camera_info.name
        )));

        // Signal stop
        if let Some(stop_tx) = &self.stop_tx {
            let _ = stop_tx.send(());
        }

        // Update state
        *self.state.lock().unwrap() = RecordingState::Stopping;

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        *self.state.lock().unwrap() = RecordingState::Idle;
        self.stop_tx = None;

        Ok(())
    }

    /// Send message to GUI
    fn send_message(&self, msg: VideoMessage) {
        if let Some(tx) = &self.tx_to_gui {
            let _ = tx.send(msg);
        }
    }

    /// Generate output filename
    fn generate_filename(camera_name: &str, format: VideoFormat) -> String {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let safe_name = camera_name.replace(' ', "_").replace('/', "_");
        format!("{}_{}.{}", safe_name, timestamp, format.extension())
    }

    /// Recording thread implementation
    #[cfg(feature = "video")]
    fn recording_thread(
        camera_info: CameraInfo,
        config: VideoConfig,
        state: Arc<Mutex<RecordingState>>,
        tx_gui: Option<Sender<VideoMessage>>,
        stop_rx: Receiver<()>,
    ) {
        let send_msg = |msg: VideoMessage| {
            if let Some(tx) = &tx_gui {
                let _ = tx.send(msg);
            }
        };

        let send_log = |msg: String| {
            info!("{}", msg);
            send_msg(VideoMessage::Log(msg));
        };

        let send_error = |msg: String| {
            error!("{}", msg);
            send_msg(VideoMessage::Error(msg.clone()));
            *state.lock().unwrap() = RecordingState::Error;
        };

        // Open camera
        send_log(format!("Opening camera source: {:?}", camera_info.source));
        
        let mut camera = match camera_info.source {
            CameraSource::Webcam(idx) => {
                match VideoCapture::new(idx, CAP_ANY) {
                    Ok(cam) => cam,
                    Err(e) => {
                        send_error(format!("Failed to open webcam {}: {}", idx, e));
                        return;
                    }
                }
            }
            _ => {
                match VideoCapture::from_file(&camera_info.source.to_opencv_string(), CAP_ANY) {
                    Ok(cam) => cam,
                    Err(e) => {
                        send_error(format!("Failed to open camera source: {}", e));
                        return;
                    }
                }
            }
        };

        // Check if camera opened
        match camera.is_opened() {
            Ok(true) => send_log("Camera opened successfully".to_string()),
            Ok(false) => {
                send_error("Camera failed to open".to_string());
                return;
            }
            Err(e) => {
                send_error(format!("Error checking camera status: {}", e));
                return;
            }
        }

        // Set camera properties
        let _ = camera.set(videoio::CAP_PROP_FRAME_WIDTH, camera_info.width as f64);
        let _ = camera.set(videoio::CAP_PROP_FRAME_HEIGHT, camera_info.height as f64);
        let _ = camera.set(videoio::CAP_PROP_FPS, camera_info.fps);

        // Get actual camera properties
        let actual_width = camera.get(videoio::CAP_PROP_FRAME_WIDTH).unwrap_or(camera_info.width as f64) as i32;
        let actual_height = camera.get(videoio::CAP_PROP_FRAME_HEIGHT).unwrap_or(camera_info.height as f64) as i32;
        let actual_fps = camera.get(videoio::CAP_PROP_FPS).unwrap_or(camera_info.fps);

        send_log(format!(
            "Camera properties: {}x{} @ {:.1} fps",
            actual_width, actual_height, actual_fps
        ));

        // Generate output filename
        let filename = Self::generate_filename(&camera_info.name, config.format);
        let output_path = config.output_dir.join(&filename);

        send_log(format!("Output file: {}", output_path.display()));

        // Create video writer
        let fourcc = config.format.fourcc();
        let frame_size = Size::new(actual_width, actual_height);
        
        let mut writer = match VideoWriter::new(
            output_path.to_str().unwrap(),
            fourcc,
            actual_fps,
            frame_size,
            true,
        ) {
            Ok(w) => w,
            Err(e) => {
                send_error(format!("Failed to create video writer: {}", e));
                return;
            }
        };

        // Check if writer opened
        match writer.is_opened() {
            Ok(true) => send_log("Video writer ready".to_string()),
            Ok(false) => {
                send_error("Video writer failed to open".to_string());
                return;
            }
            Err(e) => {
                send_error(format!("Error checking writer status: {}", e));
                return;
            }
        }

        // Notify recording started
        send_msg(VideoMessage::RecordingStarted {
            camera: camera_info.name.clone(),
            filename: filename.clone(),
        });

        // Recording loop
        let start_time = Instant::now();
        let mut frame_count: u64 = 0;
        let mut frame = Mat::default();

        send_log("Recording started".to_string());

        loop {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                send_log("Stop signal received".to_string());
                break;
            }

            // Check duration limit
            if let Some(max_dur) = config.max_duration_sec {
                if start_time.elapsed().as_secs() >= max_dur {
                    send_log(format!("Max duration reached: {}s", max_dur));
                    break;
                }
            }

            // Read frame
            match camera.read(&mut frame) {
                Ok(true) => {
                    if frame.empty() {
                        warn!("Empty frame received");
                        continue;
                    }

                    // Write frame
                    if let Err(e) = writer.write(&frame) {
                        send_error(format!("Failed to write frame: {}", e));
                        break;
                    }

                    frame_count += 1;

                    // Send progress update every 100 frames
                    if frame_count % 100 == 0 {
                        send_msg(VideoMessage::FramesCaptured(frame_count));
                    }
                }
                Ok(false) => {
                    warn!("Failed to read frame from camera");
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    send_error(format!("Error reading frame: {}", e));
                    break;
                }
            }

            // Small delay to prevent CPU spinning
            thread::sleep(Duration::from_millis(1));
        }

        // Cleanup
        let duration = start_time.elapsed().as_secs();
        send_log(format!(
            "Recording stopped. Duration: {}s, Frames: {}",
            duration, frame_count
        ));

        let _ = writer.release();
        let _ = camera.release();

        send_msg(VideoMessage::RecordingStopped {
            camera: camera_info.name.clone(),
            duration_sec: duration,
        });

        *state.lock().unwrap() = RecordingState::Idle;
    }
}

impl Drop for VideoRecorder {
    fn drop(&mut self) {
        if self.is_recording() {
            let _ = self.stop_recording();
        }
    }
}

// ============================================================================
// MULTI-CAMERA MANAGER
// ============================================================================

pub struct MultiCameraRecorder {
    recorders: Vec<VideoRecorder>,
    tx_to_gui: Option<Sender<VideoMessage>>,
}

impl MultiCameraRecorder {
    pub fn new() -> Self {
        Self {
            recorders: Vec::new(),
            tx_to_gui: None,
        }
    }

    pub fn with_gui_sender(mut self, tx: Sender<VideoMessage>) -> Self {
        self.tx_to_gui = Some(tx);
        self
    }

    pub fn add_camera(&mut self, camera_info: CameraInfo, config: VideoConfig) {
        let mut recorder = VideoRecorder::new(camera_info, config);
        
        if let Some(tx) = &self.tx_to_gui {
            recorder = recorder.with_gui_sender(tx.clone());
        }
        
        self.recorders.push(recorder);
    }

    pub fn start_all(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        
        for recorder in &mut self.recorders {
            if let Err(e) = recorder.start_recording() {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Failed to start some cameras: {}", errors.join(", ")))
        }
    }

    pub fn stop_all(&mut self) -> Result<(), String> {
        for recorder in &mut self.recorders {
            let _ = recorder.stop_recording();
        }
        Ok(())
    }

    pub fn recording_count(&self) -> usize {
        self.recorders.iter().filter(|r| r.is_recording()).count()
    }
}

impl Default for MultiCameraRecorder {
    fn default() -> Self {
        Self::new()
    }
}
