# SecCamCloud - Modular Architecture

## Overview

SecCamCloud has been refactored into a clean, modular architecture following Rust best practices. Each module has a single, well-defined responsibility, making the codebase easier to maintain, test, and extend.

## Module Structure

### Core Modules

#### **lib.rs** - Public API & Re-exports
The main library file that:
- Declares all submodules
- Re-exports public types and functions
- Contains app constants (APP_TITLE, APP_VERSION, etc.)
- Provides platform utilities (is_windows(), key_pressed())
- Manages logging initialization and rotation

**Purpose:** Acts as the single entry point for the library, making it easy to use from main.rs

#### **config.rs** - Configuration Management
Handles all configuration-related functionality:
- `ClickPoint` struct definition
- Default click points
- `AppConfig` struct
- `load_points()` - Load from JSON or confy
- `save_points()` - Save to both JSON and confy backup

**Purpose:** Isolates all configuration logic in one place

#### **automation.rs** - Automation Logic
Contains the core automation engine:
- `AutomationMessage` enum - Messages to GUI
- `AutomationThread` - Main automation execution
- Click execution with retries
- Text typing
- 8-step automation sequence
- Error handling

**Purpose:** All automation-specific logic separated from GUI concerns

#### **watchdog.rs** - Safety Timer
Simple but critical safety module:
- `WatchdogTimer` - Detects hung automation
- `reset()` - Reset the timer
- `cancel()` - Stop watching
- Timeout callback mechanism

**Purpose:** Prevents indefinite hangs during automation

#### **telemetry.rs** - Event Logging
Handles telemetry and event tracking:
- `Telemetry` - Event logging system
- `log()` - Log events to file
- Timestamp formatting
- Directory creation

**Purpose:** Separate telemetry from application logging

#### **screenshot.rs** - Screenshot Capture
Optional screenshot functionality:
- `ScreenshotManager` - Screenshot handler
- `capture()` - Take screenshots (when feature enabled)
- `is_enabled()` - Check if screenshots active
- Conditional compilation with `#[cfg(feature = "screenshots")]`

**Purpose:** Keep screenshot code isolated and optional

#### **vidrec.rs** - Video Recording
Video recording from multiple camera sources:
- `VideoRecorder` - Main recording interface
- `CameraSource` - Webcam, RTSP, HTTP support
- `VideoConfig` - Recording configuration
- `MultiCameraRecorder` - Multi-camera management
- OpenCV integration for video capture

**Purpose:** Professional video recording with multi-camera support

#### **youtube.rs** - YouTube Upload
YouTube upload with OAuth 2.0:
- `YouTubeUploader` - Upload interface
- `YouTubeCredentials` - OAuth management
- `VideoMetadata` - Video information
- `VideoValidator` - OpenCV-based validation
- `BatchUploader` - Multi-video uploads

**Purpose:** Automated YouTube uploads with authentication

### Application Module

#### **main.rs** - GUI Application
The GUI application that:
- Uses the library modules
- Implements the egui interface
- Manages application state
- Handles user interactions
- Coordinates between modules

**Purpose:** Pure GUI layer with no business logic

## Module Dependencies

```
main.rs
  ├─> lib.rs (public API)
       ├─> config.rs
       ├─> automation.rs
       │    ├─> config.rs (ClickPoint)
       │    └─> watchdog.rs
       ├─> watchdog.rs
       ├─> telemetry.rs
       ├─> screenshot.rs
       ├─> vidrec.rs
       │    └─> opencv (video capture)
       └─> youtube.rs
            ├─> vidrec.rs (VideoValidator)
            ├─> opencv (validation)
            └─> reqwest (HTTP/API)
```
       ├─> telemetry.rs
       └─> screenshot.rs
```

## Benefits of This Architecture

### 1. **Separation of Concerns**
Each module has one job:
- `config.rs` → Configuration
- `automation.rs` → Automation logic
- `watchdog.rs` → Safety monitoring
- `telemetry.rs` → Event logging
- `screenshot.rs` → Screenshot capture
- `main.rs` → GUI only

### 2. **Maintainability**
- Easy to find specific functionality
- Changes are localized to relevant modules
- Clear boundaries between components

### 3. **Testability**
- Each module can be tested independently
- Mock dependencies easily
- Unit tests per module

### 4. **Extensibility**
Ready for new features:
- Add `vidrec.rs` for video recording
- Add `upload.rs` for YouTube uploads
- Add `camera.rs` for camera management
- Each new feature gets its own module

### 5. **Reusability**
Modules can be used independently:
- `watchdog.rs` could be used in other projects
- `config.rs` provides reusable configuration patterns
- `telemetry.rs` is a standalone logging solution

### 6. **Code Organization**
- Files are focused and manageable (~200-400 lines each)
- Easy navigation for new contributors
- Clear module hierarchy

## How to Add New Modules

### Example: Video Recording (Already Implemented ✅)

The `vidrec.rs` module demonstrates the pattern:

1. **Created `vidrec.rs`:**
```rust
// src/vidrec.rs
pub struct VideoRecorder {
    camera_info: CameraInfo,
    config: VideoConfig,
    // ... fields
}

impl VideoRecorder {
    pub fn new(camera_info: CameraInfo, config: VideoConfig) -> Self { ... }
    pub fn start_recording(&mut self) -> Result<(), String> { ... }
    pub fn stop_recording(&mut self) -> Result<(), String> { ... }
}
```

2. **Declared in `lib.rs`:**
```rust
pub mod vidrec;
pub use vidrec::{VideoRecorder, VideoConfig, CameraInfo, VideoMessage};
```

3. **Used in `main.rs`:**
```rust
use seccamcloud::{VideoRecorder, CameraInfo, VideoConfig};

// In AppState:
video_recorder: Option<VideoRecorder>,
```

### Following This Pattern for New Modules

Use the same pattern for future modules like camera management or streaming.

## File Sizes (Approximate)

- `lib.rs`: ~200 lines (public API)
- `config.rs`: ~120 lines (configuration)
- `automation.rs`: ~280 lines (automation logic)
- `watchdog.rs`: ~60 lines (safety timer)
- `telemetry.rs`: ~50 lines (logging)
- `screenshot.rs`: ~80 lines (screenshots)
- `vidrec.rs`: ~700 lines (video recording)
- `youtube.rs`: ~850 lines (YouTube uploads)
- `main.rs`: ~580 lines (GUI application)

**Total:** ~2,920 lines (well-organized across 9 modules)

## Migration Notes

### From Old Structure
**Before:**
- `lib.rs` - 674 lines (everything mixed together)
- `main.rs` - 717 lines (GUI + some logic)

**After:**
- Split into 9 focused modules
- Each module has clear responsibility
- No duplicate code
- Better organization
- Added video recording and YouTube upload capabilities

### No Breaking Changes
The public API remains the same:
```rust
use seccamcloud::{
    setup_logging,
    load_points,
    save_points,
    ClickPoint,
    AutomationThread,
    AutomationMessage,
    // ... all exports work the same
};
```

## Current Modules

With this modular structure, features are easily added:

1. ✅ **Video Recording** → `vidrec.rs` (IMPLEMENTED)
2. ✅ **YouTube Upload** → `youtube.rs` (IMPLEMENTED)
3. **Camera Management** → `camera.rs` (Future)
4. **Network Streaming** → `streaming.rs` (Future)
5. **Database Storage** → `database.rs` (Future)
6. **API Server** → `server.rs` (Future)

Each feature gets its own module, keeping the codebase clean and organized.

---

**Architecture:** Modular, Single Responsibility
**Maintainability:** High
**Extensibility:** Excellent
**Code Quality:** Professional
