# SecCamCloud

Advanced automation tool with GUI interface, telemetry logging, and optional screenshot capture.

**Version:** 1.0.0  
**Author:** Michael Lauzon  
**Edition:** Rust 2024  
**License:** GPLv2

---

## Features

✅ **Professional GUI**
- Real-time activity log with timestamps
- Timer with animated progress bar
- Configurable automation settings
- Click point editor with save functionality
- Iteration counter and elapsed time tracking

✅ **Advanced Automation**
- 8-step configurable click sequence
- Automatic date field population
- Retry mechanism with watchdog timer
- Dry-run simulation mode
- Emergency stop hotkey (Windows)

✅ **Monitoring & Telemetry**
- Optional telemetry event logging
- Screenshot capture (optional feature)
- Automatic log rotation (5MB files, 3 backups)
- Comprehensive error tracking

✅ **Safety Features**
- Watchdog timer for hang detection
- Emergency stop via DELETE key (cross-platform)
- Clean thread management
- Graceful shutdown handling

---

## Quick Start

### 1. Install Rust

If you don't have Rust installed:
```bash
# Visit https://rustup.rs/ and follow instructions
# Or on Windows, download and run rustup-init.exe
```

### 2. Run the Program

**Easy way (Windows):**
```bash
# Just double-click run.bat
run.bat
```

**Command line:**
```bash
# Standard run
cargo run --release

# With options
cargo run --release -- --telemetry
cargo run --release -- --dry-run
```

The first run will take a few minutes to compile. After that, it starts instantly!

---

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--dry-run` | `-d` | Simulate actions without clicking |
| `--telemetry` | `-t` | Enable event logging to `logs/telemetry.log` |
| `--screenshots` | `-s` | Capture screenshots (requires feature) |

### Examples

```bash
# Run normally
run.bat

# Dry-run mode (safe testing)
run.bat --dry-run

# With telemetry
run.bat --telemetry

# All options
run.bat --telemetry --dry-run
```

---

## GUI Guide

### Main Controls

**Start/Stop Buttons:**
- **▶ Start Automation** - Begin the automation sequence
- **⏹ Stop** - Stop the running automation
- **DELETE key** - Emergency stop (all platforms)

### Settings Panel

**Total Wait Time:**
- Set hours and minutes for the main wait period (Step 6)
- Default: 11 hours 30 minutes

**Step Delay:**
- Delay after each click (in seconds)
- Default: 10 seconds

**Max Retries:**
- Number of retry attempts per click
- Default: 3 retries

**Step 4 Wait:**
- Special wait duration for step 4
- Default: 10 seconds

**Dry Run:**
- Enable to simulate without actual clicking
- Perfect for testing your configuration

### Click Points Editor

1. Check **✏ Edit** to enable editing
2. Modify point names, X coordinates, or Y coordinates
3. Click **💾 Save** to persist changes

**Default Points:**
1. Step 1 (3514, 1640)
2. Step 2 - Date field (1775, 596)
3. Step 3 (1474, 1649)
4. Step 5 (2875, 1640)
5. Step 7 (2674, 1640)
6. Step 8 (2066, 1100)

### Activity Log

- Real-time log display with timestamps
- Auto-scrolls to bottom
- Shows all automation events
- Tracks iterations and errors

---

## Automation Sequence

The automation performs these steps in order:

1. **Step 1** - Initial click
2. **Step 2** - Click date field and enter current date (DD-MM-YYYY format)
3. **Step 3** - Third action
4. **Step 4** - Wait (configurable duration)
5. **Step 5** - Fifth action
6. **Step 6** - Long wait with timer updates (main wait period)
7. **Step 7** - Seventh action
8. **Step 8** - Final action
9. **Loop** - Return to Step 1 and repeat

The sequence continues until you press Stop or the emergency key.

---

## Configuration Files

### Click Points

Stored in two formats:

**Primary:** `clickpoints.json` (JSON format)
```json
[
  {
    "name": "Step 1",
    "x": 3514,
    "y": 1640
  },
  {
    "name": "Step 2 (date field)",
    "x": 1775,
    "y": 596
  }
]
```

**Backup:** Platform-specific confy config (automatic)

### Logs

| File | Description | Max Size |
|------|-------------|----------|
| `automation_log.txt` | Main application log | 5MB (auto-rotates) |
| `logs/telemetry.log` | Telemetry events | Unlimited |

**Log Rotation:**
- `automation_log.txt` → `automation_log.txt.1`
- `automation_log.txt.1` → `automation_log.txt.2`
- `automation_log.txt.2` → `automation_log.txt.3`
- Keeps last 3 backups

---

## Screenshots Feature (Optional)

To enable screenshot capture:

### 1. Build with Screenshots
```bash
cargo build --release --features screenshots
```

### 2. Run with Screenshots
```bash
cargo run --release --features screenshots -- --screenshots
```

### Output
- Screenshots saved to `screenshots/` directory
- Filename format: `stepname_suffix_timestamp.png`
- RGBA PNG format

**⚠️ Warning:** Screenshots slow down automation and consume significant disk space. Use for debugging only.

---

## Platform Support

| Platform | GUI | Hotkeys | Screenshots | Status |
|----------|-----|---------|-------------|--------|
| **Windows 10/11** | ✅ | ✅ | ✅ | Full Support |
| **Linux** | ✅ | ✅ | ✅ (X11 & Wayland) | Full Support |
| **macOS** | ✅ | ✅ | ✅ | Full Support |

**Note:** Emergency stop hotkey (DELETE key) now works on all platforms.

---

## Troubleshooting

### Program won't start
**Solution:** 
- Ensure Rust is installed: `cargo --version`
- Check graphics drivers are up to date
- Try: `cargo clean` then `cargo build --release`

### Clicks miss their targets
**Solution:**
- Verify coordinates in the point editor
- Adjust step delay to allow windows to load
- Run a dry-run test first
- Check if windows are being covered

### Log file errors
**Solution:**
- Check write permissions in program directory
- Ensure no other program has the log file open
- Manually delete old log files if needed

### Emergency stop doesn't work
**Solution:**
- Press DELETE key (works on all platforms)
- Use the GUI Stop button as alternative
- Ensure the program window has focus
- Check logs for hotkey registration errors

### Build fails with "screenshots" feature
**Solution:**
- **Linux:** Install X11 dev libraries
  ```bash
  sudo apt-get install libx11-dev libxrandr-dev
  ```
- **Windows:** Should work out of the box
- **macOS:** Should work out of the box with Xcode Command Line Tools

---

## Performance & Safety

### Performance
- **Memory Usage:** ~50-100MB typical
- **CPU Usage:** <1% idle, ~5-10% during automation
- **Disk Usage:** Logs rotate at 5MB; screenshots can accumulate

### Safety Considerations

⚠️ **Important:**

1. **Test First** - Always run with `--dry-run` before actual automation
2. **Verify Coordinates** - Double-check all click points are correct
3. **Monitor First Run** - Watch the first iteration carefully
4. **Emergency Stop Ready** - Know that DELETE key stops immediately
5. **Close Important Windows** - Automation may interact with any visible window
6. **Screenshots Warning** - Can fill disk space rapidly if enabled

---

## Development

### Project Structure
```
seccamcloud/
├── Cargo.toml              # Project configuration
├── run.bat                 # Windows launcher
├── src/
│   ├── main.rs            # GUI application
│   ├── lib.rs             # Public API and re-exports
│   ├── config.rs          # Configuration management
│   ├── automation.rs      # Automation thread logic
│   ├── watchdog.rs        # Safety watchdog timer
│   ├── telemetry.rs       # Event logging system
│   ├── screenshot.rs      # Screenshot capture
│   ├── vidrec.rs          # Video recording module
│   └── youtube.rs         # YouTube upload module
├── examples/
│   └── test_examples.rs   # Video & YouTube examples
├── clickpoints.json        # Click configuration (auto-generated)
├── youtube_credentials.json # YouTube OAuth (create manually)
├── automation_log.txt      # Main log (auto-generated)
├── logs/                   # Telemetry logs (auto-generated)
├── recordings/             # Video recordings (auto-generated)
└── screenshots/            # Screenshots (if enabled, auto-generated)
```

### Building from Source

```bash
# Debug build (fast compile, slower runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# With screenshots feature
cargo build --release --features screenshots

# Run tests (if any)
cargo test
```

### Dependencies

**GUI:** eframe 0.33, egui 0.33  
**Automation:** enigo 0.6  
**Logging:** log 0.4, simplelog 0.12  
**Config:** serde 1.0, serde_json 1.0, confy 0.6  
**CLI:** clap 4.5  
**Windows:** windows 0.62 (Windows only)  

---

## Telemetry

When enabled with `--telemetry`, these events are logged:

- Application start/stop
- Automation start/stop
- Configuration saves
- Errors and exceptions
- Session statistics (duration, iterations)

**Example telemetry:**
```
[2024-11-07 14:23:45.123] Application started
[2024-11-07 14:24:10.456] START: 11h30m, retries=3, dry_run=false
[2024-11-07 14:24:15.789] Iteration 1
[2024-11-07 15:54:15.234] COMPLETE: duration=5405.0s, iterations=1
```

**Privacy:** All telemetry is stored locally in `logs/telemetry.log`. No data is sent anywhere.

---

## FAQ

**Q: Can I run multiple instances?**  
A: Not recommended. The program uses fixed click coordinates.

**Q: Does it work with multiple monitors?**  
A: Yes, but coordinates are absolute across all monitors.

**Q: Can I pause and resume?**  
A: No, use Stop and then Start again.

**Q: How do I backup my configuration?**  
A: Copy `clickpoints.json` to a safe location.

**Q: Can I change the hotkeys?**  
A: Currently hardcoded. Requires source modification.

**Q: Why is the first run slow?**  
A: Rust compiles on first run. Subsequent runs are instant.

**Q: Can I run this on a schedule?**  
A: Use Windows Task Scheduler with `run.bat --telemetry`.

---

## License

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 2 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program; if not, write to the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.

---

## Version History

### v1.0.0 (2024-11-07)
- Initial release
- Full GUI with egui/eframe
- 8-step automation sequence
- Telemetry system
- Screenshot capture support
- Log rotation
- Emergency stop hotkey
- Watchdog timer
- Configuration management
- Enigo 0.6 integration
- Rust 2024 edition

---

## Support

For issues, questions, or feature requests:
- Check the troubleshooting section above
- Review log files for error details
- Ensure all dependencies are up to date

---

## Credits

**Author:** Michael Lauzon  
**Framework:** eframe/egui (Rust GUI)  
**Automation:** enigo (Input simulation)  
**Language:** Rust 2024 Edition  
**License:** GPLv2

Built with ❤️ using Rust