# SecCamCloud - Build Instructions

## File Organization

Place all source files in the `src/` directory of your SecCamCloud project:

```
seccamcloud/
├── Cargo.toml
├── run.bat
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── automation.rs
│   ├── watchdog.rs
│   ├── telemetry.rs
│   ├── screenshot.rs
│   ├── vidrec.rs
│   └── youtube.rs
└── examples/
    └── test_examples.rs
```

## Building

### Standard Build
```bash
cargo build --release
```

### With Screenshots Feature
```bash
cargo build --release --features screenshots
```

### Running
```bash
# Using run.bat (Windows)
run.bat

# Or directly with cargo
cargo run --release

# With options
cargo run --release -- --telemetry
cargo run --release -- --dry-run
cargo run --release --features screenshots -- --screenshots
```

## Module Compilation Order

Rust automatically compiles modules in the correct order based on dependencies:

1. **Independent modules** (no dependencies):
   - `watchdog.rs`
   - `telemetry.rs`

2. **Low-level modules**:
   - `config.rs` (depends on serde, confy)
   - `screenshot.rs` (depends on image, scrap - optional)

3. **Mid-level modules**:
   - `automation.rs` (depends on config, watchdog)

4. **Library module**:
   - `lib.rs` (declares and re-exports all modules)

5. **Application**:
   - `main.rs` (uses everything via lib.rs)

## Common Build Issues

### Issue: "cannot find module"
**Solution:** Ensure all `.rs` files are in the `src/` directory

### Issue: "unresolved import"
**Solution:** Make sure `lib.rs` declares all modules:
```rust
pub mod config;
pub mod automation;
pub mod watchdog;
pub mod telemetry;
pub mod screenshot;
```

### Issue: "screenshots feature not working"
**Solution:** Build with the feature flag:
```bash
cargo build --release --features screenshots
```

### Issue: Windows-specific errors
**Solution:** The Windows API is only used for hotkey detection. On Linux/Mac, this functionality is disabled automatically.

## Verifying the Build

After building, you should have:
```
target/release/seccamcloud.exe  (Windows)
target/release/seccamcloud      (Linux/Mac)
```

## Clean Build

If you encounter issues, try a clean build:
```bash
cargo clean
cargo build --release
```

## Development Build

For faster compilation during development:
```bash
cargo build  # Debug build (faster compile, slower runtime)
cargo run    # Run debug build
```

## Dependencies

All dependencies are automatically downloaded by Cargo from `Cargo.toml`:

**Core:**
- eframe, egui (GUI)
- enigo (automation)
- chrono (date/time)
- log, simplelog (logging)
- serde, serde_json, confy (config)
- clap (CLI args)
- lazy_static (statics)

**Optional:**
- scrap, image (screenshots - with `screenshots` feature)

**Windows-only:**
- windows crate (hotkey detection)

## File Sizes (Compiled)

**Release build:**
- Windows: ~8-12 MB (without screenshots)
- Windows: ~12-15 MB (with screenshots)
- Linux: ~10-14 MB
- macOS: ~10-14 MB

**Debug build:**
- Much larger (~50-100 MB) but compiles faster

---

**Build Time:**
- First build: 2-5 minutes (downloads dependencies)
- Subsequent builds: 10-30 seconds (incremental)
- Clean release build: 1-3 minutes
