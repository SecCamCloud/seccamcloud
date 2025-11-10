# SecCamCloud - YouTube Upload Guide

## Overview

The `youtube` module provides automated video uploading to YouTube with OAuth 2.0 authentication. It uses OpenCV for video validation and Google's YouTube Data API v3 for uploads.

**Version:** 1.0.0  
**Author:** Michael Lauzon  
**License:** GPLv2

---

## Features

✅ **OAuth 2.0 Authentication**
- Secure token management
- Automatic token refresh
- Credential persistence

✅ **Video Validation with OpenCV**
- Verify video format and codec
- Check resolution and duration
- Validate file size limits
- Extract video metadata

✅ **Full YouTube API Integration**
- Video uploads with metadata
- Custom thumbnail uploads
- Privacy settings (public/unlisted/private)
- Category selection
- Tags and descriptions

✅ **Advanced Features**
- Batch uploading
- Progress tracking
- GUI message integration
- Error recovery
- Rate limiting

---

## Prerequisites

### 1. Google Cloud Project Setup

You need a Google Cloud project with YouTube Data API v3 enabled:

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable YouTube Data API v3
4. Create OAuth 2.0 credentials
5. Download client credentials

**Detailed Steps:**

#### Create Project
```
1. Visit https://console.cloud.google.com/
2. Click "New Project"
3. Name: "SecCamCloud YouTube Uploader"
4. Click "Create"
```

#### Enable API
```
1. Go to "APIs & Services" > "Library"
2. Search for "YouTube Data API v3"
3. Click "Enable"
```

#### Create Credentials
```
1. Go to "APIs & Services" > "Credentials"
2. Click "+ CREATE CREDENTIALS" > "OAuth client ID"
3. Application type: "Desktop app"
4. Name: "SecCamCloud Desktop"
5. Click "Create"
6. Download JSON (save as client_secret.json)
```

#### Configure OAuth Consent Screen
```
1. Go to "APIs & Services" > "OAuth consent screen"
2. User Type: "External" (or "Internal" if using Google Workspace)
3. Fill in application name and support email
4. Add scope: .../auth/youtube.upload
5. Add test users (your YouTube account email)
6. Save and continue
```

### 2. Get OAuth Tokens

You need to authorize the application once to get tokens:

**Option 1: Use Google's OAuth Playground**
1. Visit https://developers.google.com/oauthplayground/
2. Click settings (gear icon)
3. Check "Use your own OAuth credentials"
4. Enter your Client ID and Client Secret
5. Select YouTube Data API v3 scope
6. Authorize and get tokens

**Option 2: Use SecCamCloud's built-in auth helper** (coming soon)

### 3. Install Dependencies

```bash
# OpenCV (for video validation)
# See VIDEO_RECORDING.md for installation instructions

# Build with YouTube support
cargo build --release --features "video,youtube"
```

---

## Quick Start

### 1. Create Credentials File

Create `youtube_credentials.json`:

```json
{
  "client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
  "client_secret": "YOUR_CLIENT_SECRET",
  "access_token": "YOUR_ACCESS_TOKEN",
  "refresh_token": "YOUR_REFRESH_TOKEN",
  "token_expiry": null
}
```

### 2. Basic Upload

```rust
use seccamcloud::{
    YouTubeUploader, YouTubeCredentials, VideoMetadata, VideoPrivacy
};

fn main() {
    // Load credentials
    let creds = YouTubeCredentials::from_file("youtube_credentials.json")
        .expect("Failed to load credentials");
    
    // Create uploader
    let uploader = YouTubeUploader::new(creds);
    
    // Create metadata
    let metadata = VideoMetadata::new(
        "My Security Camera Recording",
        "Recorded on 2024-11-08"
    )
    .with_privacy(VideoPrivacy::Private);
    
    // Upload
    match uploader.upload_video("recording.mp4", metadata) {
        Ok(video_id) => {
            println!("Uploaded! Video ID: {}", video_id);
            println!("URL: https://www.youtube.com/watch?v={}", video_id);
        }
        Err(e) => eprintln!("Upload failed: {}", e),
    }
}
```

---

## API Reference

### YouTubeCredentials

Manages OAuth 2.0 authentication:

```rust
pub struct YouTubeCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: Option<i64>,
}
```

**Methods:**
```rust
// Create new credentials
let creds = YouTubeCredentials::new(
    client_id,
    client_secret,
    access_token,
    refresh_token
);

// Load from JSON file
let creds = YouTubeCredentials::from_file("credentials.json")?;

// Save to file
creds.save_to_file("credentials.json")?;

// Check if token expired
if creds.is_expired() {
    // Token will be auto-refreshed on next upload
}
```

### VideoMetadata

Video information for upload:

```rust
pub struct VideoMetadata {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub category_id: VideoCategory,
    pub privacy_status: VideoPrivacy,
    pub made_for_kids: bool,
    pub thumbnail_path: Option<PathBuf>,
}
```

**Builder Methods:**
```rust
let metadata = VideoMetadata::new("Title", "Description")
    .with_tags(vec!["security".to_string(), "camera".to_string()])
    .with_category(VideoCategory::ScienceTechnology)
    .with_privacy(VideoPrivacy::Unlisted)
    .with_thumbnail("thumbnail.jpg")
    .for_kids(false);
```

### VideoPrivacy

Privacy settings:

```rust
pub enum VideoPrivacy {
    Public,    // Anyone can watch
    Unlisted,  // Only people with link
    Private,   // Only you and people you choose
}
```

### VideoCategory

YouTube categories:

```rust
pub enum VideoCategory {
    FilmAnimation = 1,
    AutosVehicles = 2,
    Music = 10,
    PetsAnimals = 15,
    Sports = 17,
    Travel = 19,
    Gaming = 20,
    PeopleBlogs = 22,
    Comedy = 23,
    Entertainment = 24,
    NewsPolitic = 25,
    HowtoStyle = 26,
    Education = 27,
    ScienceTechnology = 28,
}
```

### YouTubeUploader

Main upload interface:

```rust
pub struct YouTubeUploader { /* ... */ }
```

**Methods:**
```rust
// Create uploader
let uploader = YouTubeUploader::new(credentials);

// Connect to GUI
let uploader = uploader.with_gui_sender(tx);

// Upload video
let video_id = uploader.upload_video("video.mp4", metadata)?;

// Get current status
let status = uploader.get_status();
```

### VideoValidator

Validates videos using OpenCV:

```rust
// Validate before upload
let info = VideoValidator::validate(Path::new("video.mp4"))?;

println!("Resolution: {}x{}", info.width, info.height);
println!("FPS: {}", info.fps);
println!("Duration: {}", info.format_duration());
println!("Size: {}", info.format_size());
```

### UploadMessage

Messages sent to GUI:

```rust
pub enum UploadMessage {
    Log(String),
    Progress { bytes_uploaded: u64, total_bytes: u64, percentage: f64 },
    VideoProcessing(String),
    UploadStarted { filename: String },
    UploadComplete { video_id: String, url: String },
    Error(String),
    ThumbnailUploaded,
}
```

### BatchUploader

Upload multiple videos:

```rust
pub struct BatchUploader { /* ... */ }
```

**Methods:**
```rust
let mut batch = BatchUploader::new(credentials)
    .with_gui_sender(tx);

// Add videos to queue
batch.add_video(path1, metadata1);
batch.add_video(path2, metadata2);
batch.add_video(path3, metadata3);

// Upload all
let video_ids = batch.upload_all()?;

println!("Uploaded {} videos", video_ids.len());
```

---

## Complete Examples

### Example 1: Upload with Custom Metadata

```rust
use seccamcloud::*;

fn main() {
    let creds = YouTubeCredentials::from_file("credentials.json")
        .expect("Failed to load credentials");
    
    let uploader = YouTubeUploader::new(creds);
    
    let metadata = VideoMetadata::new(
        "Security Camera - Front Door - 2024-11-08",
        "24-hour security footage from front door camera.\n\
         Location: Main entrance\n\
         Date: November 8, 2024"
    )
    .with_tags(vec![
        "security".to_string(),
        "surveillance".to_string(),
        "home".to_string(),
    ])
    .with_category(VideoCategory::ScienceTechnology)
    .with_privacy(VideoPrivacy::Private)
    .with_thumbnail("front_door_thumbnail.jpg");
    
    match uploader.upload_video("recordings/front_door_20241108.mp4", metadata) {
        Ok(video_id) => {
            println!("✓ Upload successful!");
            println!("  Video ID: {}", video_id);
            println!("  URL: https://www.youtube.com/watch?v={}", video_id);
        }
        Err(e) => {
            eprintln!("✗ Upload failed: {}", e);
        }
    }
}
```

### Example 2: Batch Upload Multiple Cameras

```rust
use seccamcloud::*;
use std::path::PathBuf;

fn main() {
    let creds = YouTubeCredentials::from_file("credentials.json").unwrap();
    
    let mut batch = BatchUploader::new(creds);
    
    // Front door camera
    batch.add_video(
        PathBuf::from("recordings/front_door.mp4"),
        VideoMetadata::new("Front Door Camera", "Front entrance footage")
            .with_privacy(VideoPrivacy::Private)
    );
    
    // Back door camera
    batch.add_video(
        PathBuf::from("recordings/back_door.mp4"),
        VideoMetadata::new("Back Door Camera", "Rear entrance footage")
            .with_privacy(VideoPrivacy::Private)
    );
    
    // Garage camera
    batch.add_video(
        PathBuf::from("recordings/garage.mp4"),
        VideoMetadata::new("Garage Camera", "Garage area footage")
            .with_privacy(VideoPrivacy::Private)
    );
    
    println!("Uploading {} videos...", batch.queue_len());
    
    match batch.upload_all() {
        Ok(video_ids) => {
            println!("✓ All uploads complete!");
            for (i, id) in video_ids.iter().enumerate() {
                println!("  Video {}: {}", i + 1, id);
            }
        }
        Err(e) => {
            eprintln!("✗ Batch upload failed: {}", e);
        }
    }
}
```

### Example 3: Upload with GUI Integration

```rust
use seccamcloud::*;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (tx, rx) = channel();
    
    let creds = YouTubeCredentials::from_file("credentials.json").unwrap();
    let uploader = YouTubeUploader::new(creds).with_gui_sender(tx);
    
    let metadata = VideoMetadata::new("Camera Upload", "Test upload");
    
    // Upload in background thread
    let upload_thread = thread::spawn(move || {
        uploader.upload_video("video.mp4", metadata)
    });
    
    // Monitor progress in main thread
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(UploadMessage::Log(msg)) => {
                println!("[LOG] {}", msg);
            }
            Ok(UploadMessage::Progress { percentage, .. }) => {
                println!("[PROGRESS] {:.1}%", percentage);
            }
            Ok(UploadMessage::UploadComplete { video_id, url }) => {
                println!("[SUCCESS] Video ID: {}", video_id);
                println!("[SUCCESS] URL: {}", url);
                break;
            }
            Ok(UploadMessage::Error(err)) => {
                eprintln!("[ERROR] {}", err);
                break;
            }
            _ => {}
        }
        
        // Check if upload thread finished
        if upload_thread.is_finished() {
            break;
        }
    }
    
    let _ = upload_thread.join();
}
```

### Example 4: Automatic Upload After Recording

```rust
use seccamcloud::*;
use std::sync::mpsc::channel;

fn main() {
    // Setup video recorder
    let camera = CameraInfo::new("Webcam", CameraSource::Webcam(0));
    let video_config = VideoConfig::new()
        .with_output_dir("recordings")
        .with_max_duration(60); // 1 minute
    
    let mut recorder = VideoRecorder::new(camera, video_config);
    
    // Record video
    println!("Recording for 1 minute...");
    recorder.start_recording().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(65));
    recorder.stop_recording().unwrap();
    
    // Upload to YouTube
    println!("Uploading to YouTube...");
    let creds = YouTubeCredentials::from_file("credentials.json").unwrap();
    let uploader = YouTubeUploader::new(creds);
    
    let metadata = VideoMetadata::new(
        format!("Recording {}", chrono::Local::now().format("%Y-%m-%d %H:%M")),
        "Automated upload from SecCamCloud"
    )
    .with_privacy(VideoPrivacy::Private);
    
    // Find the most recent recording
    let recording_path = "recordings/Webcam_*.mp4"; // Adjust pattern
    
    match uploader.upload_video(recording_path, metadata) {
        Ok(video_id) => {
            println!("✓ Uploaded! https://www.youtube.com/watch?v={}", video_id);
        }
        Err(e) => {
            eprintln!("✗ Upload failed: {}", e);
        }
    }
}
```

---

## GUI Integration

### Add to AppState

```rust
use seccamcloud::{YouTubeUploader, UploadMessage, YouTubeCredentials};
use std::sync::mpsc::{Sender, Receiver, channel};

struct AppState {
    // ... existing fields ...
    
    youtube_uploader: Option<YouTubeUploader>,
    upload_rx: Option<Receiver<UploadMessage>>,
    upload_log: Vec<String>,
    upload_progress: f64,
}
```

### Process Upload Messages

```rust
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process upload messages
        if let Some(rx) = &self.upload_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    UploadMessage::Log(text) => {
                        self.upload_log.push(text);
                    }
                    UploadMessage::Progress { percentage, .. } => {
                        self.upload_progress = percentage;
                    }
                    UploadMessage::UploadComplete { video_id, url } => {
                        self.upload_log.push(format!("✓ Upload complete: {}", url));
                    }
                    UploadMessage::Error(err) => {
                        self.upload_log.push(format!("✗ Error: {}", err));
                    }
                    _ => {}
                }
            }
        }
        
        // UI code...
    }
}
```

### Upload Panel UI

```rust
egui::Window::new("📤 YouTube Upload")
    .show(ctx, |ui| {
        ui.heading("Upload to YouTube");
        
        ui.separator();
        
        // File selection
        ui.horizontal(|ui| {
            ui.label("Video file:");
            if ui.button("Select...").clicked() {
                // Open file dialog
            }
        });
        
        // Metadata inputs
        ui.text_edit_singleline(&mut self.upload_title);
        ui.text_edit_multiline(&mut self.upload_description);
        
        ui.separator();
        
        // Upload button
        if ui.button("🚀 Upload to YouTube").clicked() {
            self.start_youtube_upload();
        }
        
        // Progress
        if self.upload_progress > 0.0 {
            ui.add(egui::ProgressBar::new(self.upload_progress / 100.0)
                .text(format!("{:.1}%", self.upload_progress)));
        }
        
        // Log
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for msg in &self.upload_log {
                    ui.label(msg);
                }
            });
    });
```

---

## YouTube Limits & Best Practices

### Upload Limits

| Limit | Value | Notes |
|-------|-------|-------|
| **Max Duration** | 12 hours | For verified accounts |
| **Max File Size** | 256 GB | Per video |
| **Daily Uploads** | Varies | Based on account history |
| **Default Duration** | 15 minutes | For unverified accounts |

### Best Practices

1. **Use Private Initially**
   - Upload as private first
   - Review before making public

2. **Add Descriptions**
   - Include date, time, location
   - Add context for footage

3. **Use Tags**
   - Help with organization
   - Make videos searchable

4. **Custom Thumbnails**
   - Generate from video frame
   - Make videos identifiable

5. **Organize with Playlists**
   - Group by camera
   - Group by date/time
   - Use via YouTube website

---

## Troubleshooting

### Authentication Errors

**Error:** `Token refresh failed: 401`

**Solutions:**
1. Re-authorize the application
2. Check client_secret.json is correct
3. Ensure OAuth consent screen is configured
4. Add your account as test user

### Upload Fails

**Error:** `Upload failed: 403 Forbidden`

**Solutions:**
1. Check YouTube Data API is enabled
2. Verify OAuth scopes include youtube.upload
3. Check daily upload quota
4. Ensure account in good standing

### Video Validation Fails

**Error:** `Failed to open video`

**Solutions:**
1. Ensure OpenCV is installed
2. Check video file isn't corrupted
3. Try re-encoding with FFmpeg
4. Verify file permissions

### Token Expired

**Error:** `Access token expired`

**Solution:**
- Token auto-refreshes on next upload
- If refresh fails, re-authorize application

---

## Security Considerations

### Credential Storage

**⚠️ Important:**
- Never commit credentials.json to git
- Use `.gitignore` to exclude it
- Store in secure location
- Use environment variables in production

**Example .gitignore:**
```
youtube_credentials.json
client_secret.json
*.token
```

### OAuth Scopes

Only request necessary scopes:
- `https://www.googleapis.com/auth/youtube.upload` - For uploads only
- `https://www.googleapis.com/auth/youtube` - Full account access (not needed)

### Privacy Settings

- Default to **Private** for security footage
- Review before changing to Public
- Use Unlisted for sharing specific videos
- Never upload sensitive content as Public

---

## Performance Tips

### Optimize Before Upload

```bash
# Compress video with FFmpeg
ffmpeg -i input.mp4 -c:v libx264 -crf 23 -c:a aac -b:a 128k output.mp4
```

### Batch Upload Timing

- Upload during off-peak hours
- Wait 5-10 seconds between uploads
- Monitor rate limits
- Use batch uploader for multiple videos

### Network Considerations

- Large files take time
- Use wired connection if possible
- Resume capability not yet implemented
- Plan for interruptions

---

## Integration with Video Recording

Complete workflow example:

```rust
use seccamcloud::*;
use std::path::PathBuf;

fn record_and_upload_workflow() {
    // 1. Record video
    let camera = CameraInfo::new("Security Cam", CameraSource::Webcam(0));
    let video_config = VideoConfig::new()
        .with_output_dir("recordings")
        .with_max_duration(3600);
    
    let mut recorder = VideoRecorder::new(camera, video_config);
    recorder.start_recording().unwrap();
    
    // ... recording happens ...
    
    recorder.stop_recording().unwrap();
    
    // 2. Find recorded file
    let recording_path = PathBuf::from("recordings/Security_Cam_20241108_120000.mp4");
    
    // 3. Upload to YouTube
    let creds = YouTubeCredentials::from_file("credentials.json").unwrap();
    let uploader = YouTubeUploader::new(creds);
    
    let metadata = VideoMetadata::new(
        "Security Recording - 2024-11-08",
        "Automated security camera upload"
    )
    .with_privacy(VideoPrivacy::Private)
    .with_category(VideoCategory::ScienceTechnology);
    
    let video_id = uploader.upload_video(recording_path, metadata).unwrap();
    
    println!("Recording uploaded: https://www.youtube.com/watch?v={}", video_id);
}
```

---

## FAQ

**Q: Do I need a YouTube account?**
A: Yes, you need a Google account with YouTube access.

**Q: Does this work with brand accounts?**
A: Yes, but you need to authorize the specific channel.

**Q: Can I schedule uploads?**
A: Not directly, but you can upload as private and schedule via YouTube website.

**Q: Are there usage costs?**
A: YouTube API is free but has daily quotas. Normal usage stays within limits.

**Q: Can I upload to multiple channels?**
A: Yes, use different credentials for each channel.

**Q: What about copyright?**
A: You're responsible for having rights to uploaded content.

---

## Resources

- **YouTube Data API:** https://developers.google.com/youtube/v3
- **OAuth 2.0:** https://developers.google.com/identity/protocols/oauth2
- **API Quotas:** https://developers.google.com/youtube/v3/getting-started#quota
- **OAuth Playground:** https://developers.google.com/oauthplayground

---

## License

This module is part of SecCamCloud and is licensed under GPLv2.

---

**Ready to upload!** 📤🎥☁️
