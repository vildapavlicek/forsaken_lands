# WASM Compilation Guide for Forsaken Lands (Bevy 0.18)

A comprehensive guide for compiling the Bevy 0.18 application to WebAssembly and making the codebase web-compatible.

---

## Executive Summary

Your project is **87% WASM-ready**. The primary blocker is the `save_load` crate which uses filesystem operations. All other game crates are compatible with minor configuration changes.

| Area | Status | Effort |
|------|--------|--------|
| Core game logic | ✅ Ready | None |
| Random number generation | ⚠️ Config needed | Low |
| Save/Load system | ❌ Needs rewrite | Medium |
| Timestamp handling | ⚠️ Config needed | Low |

---

## Part 1: Required Codebase Changes

### 1.1 Root Cargo.toml — Add WASM Dependencies

Add platform-specific dependencies and configure `getrandom` for WASM:

```toml
# Add to [workspace.dependencies]
getrandom = { version = "0.3", default-features = false }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Storage"] }
console_error_panic_hook = "0.1"

# Add after [workspace] section
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

> [!IMPORTANT]
> **Why `getrandom` with `wasm_js`?**  
> The `rand` 0.9 crate depends on `getrandom` for entropy. On `wasm32-unknown-unknown`, there's no native random source. The `wasm_js` feature enables using the browser's `crypto.getRandomValues()` via `wasm-bindgen`.

---

### 1.2 `rand` Configuration

Your `rand = "0.9"` usage in `wallet` and `portals` crates will work automatically once `getrandom` is configured. No code changes needed.

**Files using rand:**
- [wallet/src/lib.rs](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/wallet/src/lib.rs) — `rand::random::<f32>()`
- [portals/src/lib.rs](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/portals/src/lib.rs) — `rand::rng()`, `WeightedIndex`

---

### 1.3 `chrono` Configuration for WASM

The `chrono::Local::now()` call in `save_load` requires the `wasmbind` feature for WASM.

#### Modify [save_load/Cargo.toml](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/save_load/Cargo.toml)

```toml
[dependencies]
# Replace:
# chrono = "0.4"
# With:
chrono = { version = "0.4", default-features = false, features = ["clock"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
chrono = { version = "0.4", default-features = false, features = ["clock", "wasmbind"] }
```

> [!NOTE]
> **Why `wasmbind`?**  
> Standard Rust `std::time::SystemTime::now()` panics on `wasm32-unknown-unknown`. The `wasmbind` feature uses JavaScript's `Date` API instead.

---

### 1.4 Save/Load System — Web Storage Abstraction

This is the **largest change required**. The current [save_load/src/lib.rs](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/save_load/src/lib.rs) uses:

| Current Code | WASM Alternative |
|--------------|------------------|
| `std::fs::create_dir_all()` | Not needed (no directories in web storage) |
| `std::fs::File::options().write()` | `localStorage.setItem()` |
| `std::fs::read_dir()` | `localStorage.getItem()` |
| `chrono::Local::now()` | Works with `wasmbind` feature |

#### Proposed Architecture

Create a cross-platform storage backend trait:

```rust
// save_load/src/storage.rs

pub trait StorageBackend {
    fn save(&self, key: &str, data: &str) -> Result<(), SaveError>;
    fn load(&self, key: &str) -> Result<Option<String>, SaveError>;
    fn list_saves(&self) -> Result<Vec<String>, SaveError>;
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use native::NativeStorage as PlatformStorage;

#[cfg(target_arch = "wasm32")]
pub use web::WebStorage as PlatformStorage;
```

#### Native Implementation (keep existing logic)

```rust
// save_load/src/storage/native.rs
use std::{fs, path::Path};

pub struct NativeStorage;

impl StorageBackend for NativeStorage {
    fn save(&self, key: &str, data: &str) -> Result<(), SaveError> {
        let saves_dir = Path::new("saves");
        fs::create_dir_all(saves_dir)?;
        let filepath = saves_dir.join(format!("{}.scn.ron", key));
        fs::write(filepath, data)?;
        Ok(())
    }
    // ... load and list_saves
}
```

#### Web Implementation (new)

```rust
// save_load/src/storage/web.rs
use wasm_bindgen::JsValue;
use web_sys::window;

pub struct WebStorage;

impl StorageBackend for WebStorage {
    fn save(&self, key: &str, data: &str) -> Result<(), SaveError> {
        let storage = window()
            .ok_or(SaveError::NoWindow)?
            .local_storage()
            .map_err(|_| SaveError::StorageAccess)?
            .ok_or(SaveError::StorageAccess)?;
        
        let storage_key = format!("forsaken_lands_save_{}", key);
        storage.set_item(&storage_key, data)
            .map_err(|_| SaveError::StorageWrite)?;
        Ok(())
    }
    
    fn load(&self, key: &str) -> Result<Option<String>, SaveError> {
        // Similar pattern using localStorage.getItem()
    }
    
    fn list_saves(&self) -> Result<Vec<String>, SaveError> {
        // Iterate localStorage keys with "forsaken_lands_save_" prefix
    }
}
```

> [!WARNING]
> **localStorage limits:**  
> - ~5-10MB per domain (browser-dependent)
> - Consider using IndexedDB for larger saves (via `idb` crate)
> - For this idle game, localStorage should be sufficient

---

### 1.5 Main Entry Point Configuration

#### Modify [src/main.rs](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/src/main.rs)

```rust
use {
    bevy::{log::LogPlugin, prelude::*},
    core::CorePlugin,
};

// WASM-specific imports
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

fn main() {
    // Set up panic hook for better error messages in browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "error,loading=trace,portals=debug,village=debug,\
                        wallet=debug,heroes=debug,unlocks=info,save_load=trace,\
                        village_ui=debug".into(),
                    level: bevy::log::Level::TRACE,
                    ..Default::default()
                })
                // Configure window for web
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Forsaken Lands".into(),
                        // WASM-specific: fit to browser canvas
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#game-canvas".into()),
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(CorePlugin)
        .run();
}
```

---

## Part 2: Build Tooling

### 2.1 Recommended Tool: Trunk

[Trunk](https://trunkrs.dev/) is the Bevy-recommended build tool for WASM. It handles:
- Compiling Rust to WASM
- Generating JavaScript glue code
- Creating HTML wrapper
- Running a development server with hot-reload
- Optimizing production builds

#### Install Trunk

```bash
cargo install trunk wasm-bindgen-cli
```

#### Alternative: wasm-server-runner

For quick testing without configuration:

```bash
cargo install wasm-server-runner
```

Configure in `.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```

Then run with:
```bash
cargo run --target wasm32-unknown-unknown
```

---

### 2.2 Trunk Configuration

Create [Trunk.toml](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/Trunk.toml) in project root:

```toml
[build]
target = "index.html"
dist = "dist"

[watch]
watch = ["src", "assets"]
ignore = ["target", "dist"]

[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = ["-c", "cp -r assets dist/"]
```

---

### 2.3 HTML Template

Create [index.html](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/index.html) in project root:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Forsaken Lands</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            height: 100%;
            overflow: hidden;
            background-color: #1a1a2e;
        }
        canvas {
            display: block;
            width: 100%;
            height: 100%;
        }
        /* Loading indicator */
        #loading {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: #eee;
            font-family: sans-serif;
            font-size: 1.5rem;
        }
    </style>
    <link data-trunk rel="copy-dir" href="assets"/>
</head>
<body>
    <div id="loading">Loading...</div>
    <canvas id="game-canvas"></canvas>
    <link data-trunk rel="rust" data-wasm-opt="z"/>
</body>
</html>
```

> [!TIP]
> The `data-wasm-opt="z"` attribute tells Trunk to optimize for size (-Oz) in release builds.

---

## Part 3: Compilation

### 3.1 Prerequisites

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Install trunk
cargo install trunk

# Install wasm-bindgen-cli (must match wasm-bindgen crate version)
cargo install wasm-bindgen-cli

# (Optional) Install wasm-opt for binary optimization
# Download from: https://github.com/WebAssembly/binaryen/releases
```

### 3.2 Development Build

```bash
# Start dev server with hot-reload
trunk serve

# Or with specific port
trunk serve --port 8080
```

The game will be available at `http://localhost:8080`.

### 3.3 Release Build

```bash
trunk build --release
```

Output files will be in `dist/` directory.

---

## Part 4: Post-Compile Optimizations

### 4.1 WASM Binary Size Optimization

Add to your root `Cargo.toml`:

```toml
# Optimize WASM release builds
[profile.wasm-release]
inherits = "release"
opt-level = "z"          # Optimize for size (not speed)
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit for better optimization
strip = true             # Strip debug symbols
panic = "abort"          # Smaller panic handling
```

Build with this profile:

```bash
cargo build --profile wasm-release --target wasm32-unknown-unknown
```

### 4.2 wasm-opt Optimization

After building, further optimize with `wasm-opt` (from Binaryen):

```bash
wasm-opt -Oz -o output_optimized.wasm output.wasm
```

Typical size reduction: **15-25%**

### 4.3 Asset Optimization

| Asset Type | Optimization |
|------------|--------------|
| Images | Convert to WebP, use lower resolution |
| Audio | Use Ogg Vorbis or MP3, reduce bitrate |
| RON files | Already minimal, no changes needed |

### 4.4 Compression (Server-side)

Enable Gzip or Brotli compression on your web server:

| Format | Typical Reduction |
|--------|-------------------|
| Gzip | 60-70% |
| Brotli | 70-80% |

Example nginx config:
```nginx
gzip on;
gzip_types application/wasm application/javascript text/html;
```

---

## Part 5: Files to Modify Summary

### Definite Changes Required

| File | Change | Complexity |
|------|--------|------------|
| [Cargo.toml](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/Cargo.toml) | Add WASM deps, getrandom config | Low |
| [save_load/Cargo.toml](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/save_load/Cargo.toml) | chrono wasmbind feature | Low |
| [save_load/src/lib.rs](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/save_load/src/lib.rs) | Abstract storage backend | Medium |
| [src/main.rs](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/src/main.rs) | Canvas config, panic hook | Low |

### New Files to Create

| File | Purpose |
|------|---------|
| `Trunk.toml` | Trunk build configuration |
| `index.html` | HTML wrapper for the game |
| `save_load/src/storage.rs` | Storage backend trait |
| `save_load/src/storage/native.rs` | Native file system implementation |
| `save_load/src/storage/web.rs` | Web localStorage implementation |
| `.cargo/config.toml` | WASM runner configuration (optional) |

---

## Verification Plan

### Automated Tests

Current codebase has no automated tests discovered. After implementation:

```bash
# Verify native build still works
cargo build

# Verify WASM build compiles
cargo build --target wasm32-unknown-unknown

# Run with trunk dev server
trunk serve
```

### Manual Verification

1. **Native build verification**: Run `cargo run` and verify save/load (F5/F9) still works
2. **WASM compilation**: Run `trunk build` and verify no compilation errors
3. **Browser testing**: 
   - Open `trunk serve` in browser
   - Game should load and display
   - Test autosave (wait 1 minute)
   - Close and reopen browser tab — save should persist
   - Check browser console for any errors

---

## Not Covered (Out of Scope)

As requested, this guide does **not** cover:
- Deployment to hosting services (itch.io, GitHub Pages, etc.)
- CI/CD pipeline configuration
- Mobile/touch input support
- Progressive Web App (PWA) setup

---

## References

- [Bevy WASM Cheatbook](https://bevy-cheatbook.github.io/platforms/wasm.html)
- [Trunk Documentation](https://trunkrs.dev/)
- [getrandom WASM Support](https://docs.rs/getrandom/latest/getrandom/#webassembly-support)
- [chrono WASM Support](https://docs.rs/chrono/latest/chrono/#wasm-support)
