# SecCamCloud - YouTube Upload Integration Guide

## Overview

This guide walks you through integrating YouTube upload functionality into SecCamCloud. The module uses OpenCV for video validation and Google's YouTube Data API v3 for secure uploads.

---

## Files Delivered

1. **youtube.rs** - Complete YouTube upload module
2. **Cargo.toml** - Updated with reqwest dependency
3. **lib.rs** - Updated module declarations
4. **YOUTUBE_UPLOAD.md** - Complete API documentation

---

## Installation Steps

### Step 1: Copy Files

```bash
# Copy youtube module to src/
cp youtube.rs src/

# Update Cargo.toml (or merge manually)
cp Cargo.toml .

# Update lib.rs (or merge manually)
cp lib.rs src/
```

### Step 2: Install Dependencies

The YouTube module requires OpenCV (already installed for video recording):

```bash
# If not already installed, see VIDEO_RECORDING.md
```

### Step 3: Build with YouTube Feature

```bash
# Build with YouTube support (includes video feature automatically)
cargo build --release --features youtube

# Or with all features
cargo build --release --features "screenshots,video,youtube"
```

---

## Google Cloud Setup

### Prerequisites

Before using YouTube uploads, you need:

1. Google account with YouTube access
2. Google Cloud Project
3. YouTube Data API v3 enabled
4. OAuth 2.0 credentials
5. Access and refresh tokens

### Detailed Setup Process

#### 1. Create Google Cloud Project

```
1. Visit https://console.cloud.google.com/
2. Click "New Project"
3. Project name: "SecCamCloud YouTube"
4. Click "Create"
5. Wait for project creation (30 seconds)
```

#### 2. Enable YouTube Data API v3

```
1. In your project, go to "APIs & Services" > "Library"
2. Search for "YouTube Data API v3"
3. Click on it
4. Click "Enable"
5. Wait for activation
```

#### 3. Configure OAuth Consent Screen

```
1. Go to "APIs & Services" > "OAuth consent screen"
2. User Type: Select "External" (or "Internal" for Workspace)
3. Click "Create"
4. Fill in required fields:
   - App name: "SecCamCloud"
   - User support email: (your email)
   - Developer contact: (your email)
5. Click "Save and Continue"
6. Scopes: Click "Add or Remove Scopes"
   - Search for "youtube"
   - Check: ".../auth/youtube.upload"
   - Click "Update"
   - Click "Save and Continue"
7. Test users: Add your Google account email
8. Click "Save and Continue"
9. Review and click "Back to Dashboard"
```

#### 4. Create OAuth 2.0 Credentials

```
1. Go to "APIs & Services" > "Credentials"
2. Click "+ CREATE CREDENTIALS"
3. Select "OAuth client ID"
4. Application type: "Desktop app"
5. Name: "SecCamCloud Desktop"
6. Click "Create"
7. You'll see client ID and secret
8. Click "Download JSON"
9. Save as "client_secret.json"
```

#### 5. Get Access and Refresh Tokens

**Option A: Using OAuth Playground (Easiest)**

```
1. Visit https://developers.google.com/oauthplayground/
2. Click settings icon (top right)
3. Check "Use your own OAuth credentials"
4. Enter your Client ID and Client Secret
5. Close settings
6. Select "YouTube Data API v3"
7. Check "https://www.googleapis.com/auth/youtube.upload"
8. Click "Authorize APIs"
9. Sign in with your Google account
10. Click "Allow"
11. Click "Exchange authorization code for tokens"
12. Copy the "Access token" and "Refresh token"
```

**Option B: Using curl (Advanced)**

```bash
# 1. Get authorization code (paste URL in browser)
echo "https://accounts.google.com/o/oauth2/v2/auth?client_id=YOUR_CLIENT_ID&redirect_uri=urn:ietf:wg:oauth:2.0:oob&response_type=code&scope=https://www.googleapis.com/auth/youtube.upload"

# 2. After authorizing, you'll get a code. Use it here:
curl -X POST https://oauth2.googleapis.com/token \
  -d "code=YOUR_AUTH_CODE" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "redirect_uri=urn:ietf:wg:oauth:2.0:oob" \
  -d "grant_type=authorization_code"

# 3. Response contains access_token and refresh_token
```

#### 6. Create Credentials File

Create `youtube_credentials.json`:

```json
{
  "client_id": "123456789-abcdefg.apps.googleusercontent.com",
  "client_secret": "YOUR_CLIENT_SECRET_HERE",
  "access_token": "ya29.a0AfH6SMC...",
  "refresh_token": "1//0gK4r...",
  "token_expiry": null
}
```

**⚠️ Security:**
- Never commit this file to git
- Add to `.gitignore`
- Store securely
- Regenerate if compromised

---

## Quick Test

### Test 1: Validate Credentials

Create `examples/test_youtube_auth.rs`:

```rust
use seccamcloud::YouTubeCredentials;

fn main() {
    println!("Testing YouTube authentication...\n");
    
    match YouTubeCredentials::from_file("youtube_credentials.json") {
        Ok(creds) => {
            println!("✓ Credentials loaded successfully");
            println!("  Client ID: {}...", &creds.client_id[..20]);
            println!("  Has refresh token: {}", !creds.refresh_token.is_empty());
            
            if creds.is_expired() {
                println!("  ⚠ Access token expired (will auto-refresh)");
            } else {
                println!("  ✓ Access token valid");
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load credentials: {}", e);
            eprintln!("\nMake sure youtube_credentials.json exists");
        }
    }
}
```

Run:
```bash
cargo run --release --features youtube --example test_youtube_auth
```

### Test 2: Validate Video

Create `examples/test_youtube_validate.rs`:

```rust
use seccamcloud::VideoValidator;
use std::path::Path;

fn main() {
    let video_path = "test_video.mp4"; // Change to your video
    
    println!("Validating video: {}\n", video_path);
    
    match VideoValidator::validate(Path::new(video_path)) {
        Ok(info) => {
            println!("✓ Video is valid!");
            println!("  Resolution: {}x{}", info.width, info.height);
            println!("  FPS: {:.1}", info.fps);
            println!("  Duration: {}", info.format_duration());
            println!("  Size: {}", info.format_size());
            println!("  Frames: {}", info.frame_count);
        }
        Err(e) => {
            eprintln!("✗ Validation failed: {}", e);
        }
    }
}
```

Run:
```bash
cargo run --release --features youtube --example test_youtube_validate
```

### Test 3: Upload Video

Create `examples/test_youtube_upload.rs`:

```rust
use seccamcloud::*;

fn main() {
    println!("=== YouTube Upload Test ===\n");
    
    // Load credentials
    let creds = match YouTubeCredentials::from_file("youtube_credentials.json") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load credentials: {}", e);
            return;
        }
    };
    
    // Create uploader
    let uploader = YouTubeUploader::new(creds);
    
    // Create metadata
    let metadata = VideoMetadata::new(
        "SecCamCloud Test Upload",
        "This is a test upload from SecCamCloud\n\nTesting automated YouTube uploads."
    )
    .with_tags(vec!["test".to_string(), "seccamcloud".to_string()])
    .with_privacy(VideoPrivacy::Private)
    .with_category(VideoCategory::ScienceTechnology);
    
    println!("Uploading test_video.mp4...\n");
    
    // Upload
    match uploader.upload_video("test_video.mp4", metadata) {
        Ok(video_id) => {
            println!("\n✓ Upload successful!");
            println!("  Video ID: {}", video_id);
            println!("  URL: https://www.youtube.com/watch?v={}", video_id);
            println!("\nNote: Video is private. View it in YouTube Studio.");
        }
        Err(e) => {
            eprintln!("\n✗ Upload failed: {}", e);
        }
    }
}
```

Run:
```bash
cargo run --release --features youtube --example test_youtube_upload
```

---

## Integration with Main Application

### Add to AppState

```rust
use seccamcloud::{
    YouTubeUploader, YouTubeCredentials, VideoMetadata,
    VideoPrivacy, VideoCategory, UploadMessage
};
use std::sync::mpsc::{Sender, Receiver, channel};

struct AppState {
    // ... existing fields ...
    
    // YouTube upload
    youtube_uploader: Option<YouTubeUploader>,
    upload_rx: Option<Receiver<UploadMessage>>,
    upload_log: Vec<String>,
    upload_progress: f64,
    upload_video_path: String,
    upload_title: String,
    upload_description: String,
    upload_privacy: VideoPrivacy,
}
```

### Initialize in new()

```rust
impl AppState {
    fn new(cc: &eframe::CreationContext) -> Self {
        // Try to load YouTube credentials
        let youtube_uploader = match YouTubeCredentials::from_file("youtube_credentials.json") {
            Ok(creds) => {
                info!("YouTube credentials loaded");
                Some(YouTubeUploader::new(creds))
            }
            Err(e) => {
                warn!("YouTube credentials not found: {}", e);
                None
            }
        };
        
        Self {
            // ... existing initialization ...
            
            youtube_uploader,
            upload_rx: None,
            upload_log: Vec::new(),
            upload_progress: 0.0,
            upload_video_path: String::new(),
            upload_title: String::new(),
            upload_description: String::new(),
            upload_privacy: VideoPrivacy::Private,
        }
    }
}
```

### Process Messages

```rust
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process YouTube messages
        if let Some(rx) = &self.upload_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    UploadMessage::Log(text) => {
                        self.upload_log.push(format!("[{}] {}", 
                            chrono::Local::now().format("%H:%M:%S"), text));
                    }
                    UploadMessage::Progress { percentage, .. } => {
                        self.upload_progress = percentage;
                    }
                    UploadMessage::UploadComplete { video_id, url } => {
                        self.upload_log.push(format!("✓ Complete: {}", url));
                        self.upload_progress = 100.0;
                    }
                    UploadMessage::Error(err) => {
                        self.upload_log.push(format!("✗ Error: {}", err));
                    }
                    _ => {}
                }
            }
        }
        
        // ... rest of update ...
    }
}
```

### Add UI Panel

```rust
// In your update() method, add YouTube upload window

egui::Window::new("📤 YouTube Upload")
    .default_width(500.0)
    .show(ctx, |ui| {
        ui.heading("Upload Video to YouTube");
        
        if self.youtube_uploader.is_none() {
            ui.colored_label(
                egui::Color32::RED,
                "⚠ YouTube credentials not found"
            );
            ui.label("Create youtube_credentials.json to enable uploads");
            return;
        }
        
        ui.separator();
        
        // File selection
        ui.horizontal(|ui| {
            ui.label("Video file:");
            ui.text_edit_singleline(&mut self.upload_video_path);
            if ui.button("📁 Browse...").clicked() {
                // File picker implementation
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Video", &["mp4", "avi", "mkv"])
                    .pick_file()
                {
                    self.upload_video_path = path.display().to_string();
                }
            }
        });
        
        ui.separator();
        
        // Metadata
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.upload_title);
        });
        
        ui.horizontal(|ui| {
            ui.label("Description:");
        });
        ui.text_edit_multiline(&mut self.upload_description);
        
        ui.horizontal(|ui| {
            ui.label("Privacy:");
            egui::ComboBox::from_id_source("privacy")
                .selected_text(format!("{:?}", self.upload_privacy))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.upload_privacy, VideoPrivacy::Private, "Private");
                    ui.selectable_value(&mut self.upload_privacy, VideoPrivacy::Unlisted, "Unlisted");
                    ui.selectable_value(&mut self.upload_privacy, VideoPrivacy::Public, "Public");
                });
        });
        
        ui.separator();
        
        // Upload button
        let can_upload = !self.upload_video_path.is_empty() 
                      && !self.upload_title.is_empty()
                      && self.upload_progress == 0.0;
        
        if ui.add_enabled(can_upload, egui::Button::new("🚀 Upload to YouTube"))
            .clicked()
        {
            self.start_youtube_upload();
        }
        
        // Progress
        if self.upload_progress > 0.0 && self.upload_progress < 100.0 {
            ui.add(
                egui::ProgressBar::new(self.upload_progress / 100.0)
                    .text(format!("{:.1}%", self.upload_progress))
            );
        }
        
        ui.separator();
        
        // Log
        ui.label("Upload Log:");
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &self.upload_log {
                    ui.label(msg);
                }
            });
    });
```

### Helper Method

```rust
impl AppState {
    fn start_youtube_upload(&mut self) {
        use std::thread;
        
        if self.youtube_uploader.is_none() {
            self.upload_log.push("✗ No YouTube credentials".to_string());
            return;
        }
        
        let video_path = self.upload_video_path.clone();
        let title = self.upload_title.clone();
        let description = self.upload_description.clone();
        let privacy = self.upload_privacy;
        
        // Create message channel
        let (tx, rx) = channel();
        self.upload_rx = Some(rx);
        
        // Setup uploader with GUI sender
        if let Some(uploader) = self.youtube_uploader.take() {
            let uploader = uploader.with_gui_sender(tx);
            
            // Upload in background thread
            thread::spawn(move || {
                let metadata = VideoMetadata::new(title, description)
                    .with_privacy(privacy)
                    .with_category(VideoCategory::ScienceTechnology);
                
                let _ = uploader.upload_video(video_path, metadata);
            });
            
            self.upload_log.push("Upload started...".to_string());
            self.upload_progress = 0.1;
        }
    }
}
```

---

## Automated Workflow

### Record and Upload Automatically

```rust
use seccamcloud::*;
use std::path::PathBuf;

fn automated_record_and_upload() {
    // 1. Setup recording
    let camera = CameraInfo::new("Security Cam", CameraSource::Webcam(0));
    let video_config = VideoConfig::new()
        .with_output_dir("recordings")
        .with_format(VideoFormat::MP4)
        .with_max_duration(3600); // 1 hour
    
    let mut recorder = VideoRecorder::new(camera, video_config);
    
    // 2. Record
    println!("Recording...");
    recorder.start_recording().unwrap();
    
    // Wait for recording (or use messages to detect completion)
    std::thread::sleep(std::time::Duration::from_secs(3605));
    
    recorder.stop_recording().unwrap();
    
    // 3. Find recorded file (implement your logic here)
    let recorded_file = PathBuf::from("recordings/Security_Cam_20241108_120000.mp4");
    
    // 4. Upload to YouTube
    println!("Uploading to YouTube...");
    
    let creds = YouTubeCredentials::from_file("youtube_credentials.json").unwrap();
    let uploader = YouTubeUploader::new(creds);
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M");
    let metadata = VideoMetadata::new(
        format!("Security Recording {}", timestamp),
        format!("Automated upload from SecCamCloud\nRecorded: {}", timestamp)
    )
    .with_privacy(VideoPrivacy::Private)
    .with_tags(vec!["security".to_string(), "automated".to_string()])
    .with_category(VideoCategory::ScienceTechnology);
    
    match uploader.upload_video(recorded_file, metadata) {
        Ok(video_id) => {
            println!("✓ Uploaded: https://www.youtube.com/watch?v={}", video_id);
        }
        Err(e) => {
            eprintln!("✗ Upload failed: {}", e);
        }
    }
}
```

---

## Troubleshooting

### Common Issues

#### 1. Authentication Failed

**Error:** `Token refresh failed: 401`

**Solutions:**
- Re-do OAuth flow to get new tokens
- Check client_id and client_secret are correct
- Ensure OAuth consent screen is published
- Verify test user is added

#### 2. API Not Enabled

**Error:** `Upload failed: 403 Forbidden`

**Solutions:**
- Enable YouTube Data API v3 in Cloud Console
- Wait 5 minutes after enabling
- Check project has billing enabled (if required)

#### 3. Quota Exceeded

**Error:** `403 quotaExceeded`

**Solutions:**
- Daily upload limit reached
- Wait until next day (resets at midnight Pacific Time)
- Request quota increase in Cloud Console

#### 4. Invalid Video

**Error:** `Failed to open video`

**Solutions:**
- Ensure OpenCV is installed
- Check video isn't corrupted
- Verify file format is supported
- Try re-encoding with FFmpeg

#### 5. File Not Found

**Error:** `Failed to open video file`

**Solutions:**
- Check file path is correct
- Use absolute paths
- Verify file permissions
- Check file actually exists

---

## Best Practices

### Security

1. **Never commit credentials**
   ```gitignore
   youtube_credentials.json
   client_secret.json
   *.token
   ```

2. **Use environment variables in production**
   ```rust
   let client_id = std::env::var("YOUTUBE_CLIENT_ID")?;
   ```

3. **Rotate tokens periodically**

4. **Use minimum required scopes**

### Upload Strategy

1. **Default to Private**
   - Review before making public
   - Avoid accidental exposure

2. **Use Descriptive Titles**
   - Include date/time
   - Include camera location
   - Use consistent naming

3. **Add Metadata**
   - Tags for searchability
   - Descriptions with context
   - Appropriate categories

4. **Batch Processing**
   - Upload during off-peak hours
   - Wait between uploads
   - Monitor quotas

### Error Handling

1. **Validate before upload**
   ```rust
   let info = VideoValidator::validate(&path)?;
   ```

2. **Log all operations**
   ```rust
   info!("Upload started: {}", video_id);
   ```

3. **Handle token refresh**
   - Automatic in module
   - Log refresh events

4. **Retry on failure**
   - Network issues
   - Temporary API errors
   - Not for auth errors

---

## Architecture Integration

The YouTube module fits seamlessly:

```
src/
├── lib.rs          (declares youtube module)
├── main.rs         (GUI with upload panel)
├── config.rs       (configuration)
├── automation.rs   (automation)
├── watchdog.rs     (safety)
├── telemetry.rs    (logging)
├── screenshot.rs   (screenshots)
├── vidrec.rs       (video recording)
└── youtube.rs      (YouTube uploads) ← NEW!
```

**Benefits:**
- ✅ Consistent code style
- ✅ Same module pattern
- ✅ GPLv2 license
- ✅ Rust 2024 edition
- ✅ Independent and reusable
- ✅ Works with vidrec module

---

## Next Steps

1. ✅ Setup Google Cloud project
2. ✅ Get OAuth credentials
3. ✅ Copy youtube.rs to src/
4. ✅ Update Cargo.toml and lib.rs
5. ✅ Build with youtube feature
6. ✅ Test authentication
7. ✅ Test video upload
8. ✅ Integrate into GUI

---

## Resources

- **Google Cloud Console:** https://console.cloud.google.com/
- **YouTube API Docs:** https://developers.google.com/youtube/v3
- **OAuth Playground:** https://developers.google.com/oauthplayground/
- **API Quotas:** https://console.cloud.google.com/apis/api/youtube.googleapis.com/quotas

---

**Ready to upload to YouTube!** 📤☁️🎥
