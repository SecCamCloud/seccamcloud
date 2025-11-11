# SecCamCloud - Screenshots Guide

## Overview

The screenshot module provides automated screen capture functionality for SecCamCloud, with support for Windows, Linux (X11 & Wayland), and macOS.

**Version:** 1.0.0  
**Author:** Michael Lauzon  
**License:** GPLv2

---

## Features

✅ **Cross-Platform Support**
- Windows (native)
- Linux X11 (primary method)
- Linux Wayland (primary method)
- macOS (experimental)

✅ **Dual Capture Methods**
- `captrs` - Primary (supports X11 & Wayland)
- `scrap` - Fallback (X11 only)

✅ **Automatic Fallback**
- Tries captrs first
- Falls back to scrap if needed
- Graceful failure handling

✅ **Integration**
- Optional feature flag
- Thread-safe operations
- Timestamped filenames
- Organized output directory

---

## Installation

### Build with Screenshots

```bash
cargo build --release --features screenshots
```

### Dependencies

The screenshots feature includes:
- `scrap` 0.5 - X11 screenshot capture
- `captrs` 0.3 - X11 & Wayland capture
- `image` 0.25 - Image processing and PNG encoding

---

## Usage

### Command Line

```bash
# Enable screenshots
cargo run --release --features screenshots -- --screenshots

# Or with run.bat
run.bat --screenshots
```

### In Code

```rust
use seccamcloud::ScreenshotManager;

// Create screenshot manager (enabled)
let screenshot_mgr = ScreenshotManager::new(true);

// Capture screenshot
if let Some(filename) = screenshot_mgr.capture("step1", "before") {
    println!("Screenshot saved: {}", filename);
}

// Check if enabled
if screenshot_mgr.is_enabled() {
    // Take screenshots
}
```

### Output

Screenshots are saved to `screenshots/` directory with format:
```
screenshots/step1_before_20241108_143022.png
screenshots/step2_after_20241108_143025.png
```

**Filename format:** `{step_name}_{suffix}_{timestamp}.png`

---

## Platform Details

### Windows

**Method:** scrap (native Windows API)
- ✅ Full support
- ✅ Multi-monitor
- ✅ No additional setup

### Linux (X11)

**Methods:** captrs (primary), scrap (fallback)
- ✅ Full support
- ✅ Auto-detection
- ✅ No root required

**Requirements:**
```bash
sudo apt-get install libx11-dev libxrandr-dev
```

### Linux (Wayland)

**Method:** captrs only
- ✅ Full support
- ✅ Auto-detection
- ✅ No permission dialogs

**Note:** scrap doesn't support Wayland, but captrs handles it automatically.

### macOS

**Method:** scrap
- ⚠️ Experimental support
- May require permissions
- Screen Recording permission needed

---

## How It Works

### Capture Process

1. **Check if enabled** - Return None if disabled
2. **Try captrs first** - Works on X11 and Wayland
3. **Fallback to scrap** - If captrs fails (X11 only)
4. **Convert format** - BGRA → RGBA for PNG
5. **Save to disk** - Timestamped filename

### Dual Method Strategy

```rust
pub fn capture(&self, step_name: &str, suffix: &str) -> Option<String> {
    // Try captrs first (X11 & Wayland)
    if let Some(path) = self.capture_with_captrs(&filename) {
        return Some(path);
    }
    
    // Fallback to scrap (X11 only)
    if let Some(path) = self.capture_with_scrap(&filename) {
        return Some(path);
    }
    
    None // Both failed
}
```

### Why Two Methods?

- **captrs** - Modern, supports Wayland, actively maintained
- **scrap** - Mature, stable on X11, good fallback
- **Fallback ensures** - Maximum compatibility

---

## Configuration

### Enable/Disable

```rust
// Enabled
let mgr = ScreenshotManager::new(true);

// Disabled (no overhead)
let mgr = ScreenshotManager::new(false);
```

### Output Directory

Default: `screenshots/`

The directory is created automatically on first capture.

### File Format

- **Format:** PNG (RGBA)
- **Quality:** Lossless
- **Size:** ~1-5MB per screenshot (depends on resolution)

---

## Integration with Automation

### Capture Before/After

```rust
// Before step
screenshot_mgr.capture("step1", "before");

// Perform automation step
execute_click(&point);

// After step
screenshot_mgr.capture("step1", "after");
```

### Error Handling

```rust
match screenshot_mgr.capture("step", "capture") {
    Some(path) => {
        info!("Screenshot saved: {}", path);
    }
    None => {
        warn!("Screenshot failed (disabled or error)");
    }
}
```

---

## Performance Considerations

### Resource Usage

**Per Screenshot:**
- CPU: <1% (brief spike)
- Memory: ~10-20MB temporary
- Disk: 1-5MB per file

**Impact on Automation:**
- Minimal delay (~50-200ms per capture)
- No blocking operations
- Graceful failures don't stop automation

### Recommendations

1. **Use sparingly** - Only capture key steps
2. **Disable in production** - Unless debugging
3. **Monitor disk space** - Screenshots accumulate
4. **Clean old files** - Implement rotation if needed

---

## Troubleshooting

### Screenshots Not Working

**Check if feature enabled:**
```bash
cargo build --features screenshots
```

**Verify at runtime:**
```rust
if !screenshot_mgr.is_enabled() {
    println!("Screenshots disabled");
}
```

### Linux: "Failed to capture"

**X11:**
```bash
# Install dependencies
sudo apt-get install libx11-dev libxrandr-dev

# Check X11 running
echo $XDG_SESSION_TYPE  # Should show "x11"
```

**Wayland:**
```bash
# Check Wayland running
echo $XDG_SESSION_TYPE  # Should show "wayland"

# captrs should work automatically
```

### Windows: Black Screenshots

**Solution:**
- Ensure no applications blocking screen capture
- Check antivirus isn't interfering
- Try running as administrator

### macOS: Permission Denied

**Solution:**
1. System Preferences → Security & Privacy
2. Privacy → Screen Recording
3. Add SecCamCloud to allowed apps
4. Restart application

### Empty Screenshots

**Causes:**
- Window minimized during capture
- Display turned off
- Graphics driver issue

**Solutions:**
- Ensure window visible
- Check display active
- Update graphics drivers

---

## Examples

### Example 1: Debug Automation

```rust
use seccamcloud::ScreenshotManager;

fn debug_automation() {
    let screenshots = ScreenshotManager::new(true);
    
    for i in 1..=8 {
        screenshots.capture(&format!("step{}", i), "before");
        
        // Perform automation step
        execute_step(i);
        
        screenshots.capture(&format!("step{}", i), "after");
    }
}
```

### Example 2: Conditional Capture

```rust
fn conditional_capture(debug_mode: bool) {
    let screenshots = ScreenshotManager::new(debug_mode);
    
    if screenshots.is_enabled() {
        screenshots.capture("critical_step", "state");
    }
    
    // Continue normally whether captured or not
}
```

### Example 3: Error Documentation

```rust
fn document_error(error: &str) {
    let screenshots = ScreenshotManager::new(true);
    
    if let Some(path) = screenshots.capture("error", error) {
        log::error!("Error screenshot: {}", path);
    }
}
```

---

## API Reference

### ScreenshotManager

```rust
pub struct ScreenshotManager {
    enabled: bool,
    output_dir: String,
}
```

**Methods:**

#### new()
```rust
pub fn new(enabled: bool) -> Arc<Self>
```
Creates a new screenshot manager.
- `enabled` - Whether screenshots are active
- Returns Arc for thread-safe sharing

#### capture()
```rust
pub fn capture(&self, step_name: &str, suffix: &str) -> Option<String>
```
Captures a screenshot.
- `step_name` - Descriptive name for the step
- `suffix` - Additional identifier (e.g., "before", "after")
- Returns filename if successful, None if disabled or failed

#### is_enabled()
```rust
pub fn is_enabled(&self) -> bool
```
Checks if screenshots are enabled.

---

## Best Practices

### 1. Use Descriptive Names

```rust
// Good
screenshot_mgr.capture("login_form", "before_submit");
screenshot_mgr.capture("dashboard", "after_load");

// Bad
screenshot_mgr.capture("s1", "a");
screenshot_mgr.capture("test", "x");
```

### 2. Capture Key Points Only

```rust
// Good - Only critical steps
screenshot_mgr.capture("error_occurred", "state");
screenshot_mgr.capture("verification", "result");

// Bad - Every minor step
screenshot_mgr.capture("mouse_move_1", "position");
screenshot_mgr.capture("mouse_move_2", "position");
```

### 3. Handle Failures Gracefully

```rust
// Good
if let Some(path) = screenshot_mgr.capture("step", "state") {
    log::debug!("Screenshot: {}", path);
}
// Continue regardless

// Bad
let path = screenshot_mgr.capture("step", "state").unwrap(); // Panic if fails!
```

### 4. Clean Up Old Files

```rust
use std::fs;
use std::time::{SystemTime, Duration};

fn cleanup_old_screenshots(days: u64) {
    let cutoff = SystemTime::now() - Duration::from_secs(days * 86400);
    
    if let Ok(entries) = fs::read_dir("screenshots") {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}
```

---

## Security Considerations

### Privacy

⚠️ **Screenshots may contain sensitive information:**
- Personal data
- Passwords (if visible)
- Confidential information
- Private conversations

**Recommendations:**
1. Disable in production unless necessary
2. Secure screenshot directory permissions
3. Implement automatic cleanup
4. Encrypt storage if needed

### Storage

```bash
# Linux: Restrict access
chmod 700 screenshots/

# Windows: Set folder permissions
# Right-click → Properties → Security
```

---

## Comparison with Other Tools

| Feature | SecCamCloud | Flameshot | Spectacle | Greenshot |
|---------|-------------|-----------|-----------|-----------|
| **Automated** | ✅ | ❌ | ❌ | ⚠️ |
| **Linux (X11 Support)** | ✅ | ✅ | ✅ | N/A |
| **Linux (Wayland Support)** | ✅ | ⚠️ | ✅ | N/A |
| **Windows** | ✅ | ❌ | ❌ | ✅ |
| **API/Code** | ✅ | ❌ | ❌ | ❌ |
| **No GUI Required** | ✅ | ❌ | ❌ | ❌ |

**SecCamCloud Advantages:**
- Fully automated
- Code-integrated
- Cross-platform
- No user interaction needed

---

## Future Enhancements

Possible additions (not yet implemented):

1. **Video Recording** - Capture video sequences (see VIDREC module)
2. **Annotations** - Add text/arrows to screenshots
3. **Comparison** - Diff between before/after
4. **OCR** - Extract text from screenshots
5. **Upload** - Auto-upload to cloud storage
6. **Thumbnails** - Generate preview images

**Note:** Video recording is available via the `vidrec` module.

---

## License

This module is part of SecCamCloud and is licensed under GPLv2.

---

## Support

For screenshot-related issues:

1. Check build includes screenshots feature
2. Verify platform dependencies installed
3. Test with simple capture first
4. Check file permissions on output directory
5. Review logs for error messages

**Platform-specific help:**
- **Linux X11:** Install libx11-dev, libxrandr-dev
- **Linux Wayland:** Should work automatically with captrs
- **Windows:** Ensure no security software blocking
- **macOS:** Grant Screen Recording permission

---

**Ready to capture!** 📸✨
