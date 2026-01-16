# WASM Feasibility Study: Forsaken Lands

**Date:** January 15, 2026  
**Game:** Forsaken Lands (Bevy 0.17 idle/incremental game)

---

## Executive Summary

> [!TIP]
> **Verdict: Highly Feasible** — The game can be compiled to WASM with minimal changes. All core dependencies have mature WASM support.

---

## Dependency Analysis

### Core Dependencies

| Dependency | Version | WASM Status | Notes |
|------------|---------|-------------|-------|
| **bevy** | 0.17 | ✅ Excellent | Official WASM target support, actively maintained |
| **rand** | 0.9 | ✅ Excellent | Uses `wasm-bindgen` for entropy on web |
| **serde** | 1.x | ✅ Excellent | Pure Rust, no issues |
| **bevy_common_assets** | 0.14 | ✅ Excellent | Works with Bevy's asset system on WASM |

### Workspace Structure

- **40+ internal workspace crates** — All use standard Bevy patterns
- **No direct file I/O** in the main game (only in `asset_editor` tool)
- **No threading/networking** — No `std::thread`, `std::net`, or `std::process` usage detected
- **Pure ECS architecture** — Component-based design is fully WASM-compatible

---

## Blockers & Risks

### ✅ No Critical Blockers Found

The codebase is clean of common WASM blockers:

```text
std::fs       → Only in tools/asset_editor (excluded from game build)
std::thread   → Not used
std::net      → Not used  
std::process  → Not used
native FFI    → Not detected
```

### ⚠️ Items Requiring Attention

| Item | Effort | Solution |
|------|--------|----------|
| **Save/Load System** | Medium | Implement using `localStorage` or IndexedDB (currently no save system) |
| **Asset Loading** | Low | Configure for HTTP/fetch-based loading |
| **Window Management** | Low | Adjust for browser canvas |
| **Audio** | Low | Use Bevy's WASM-compatible audio backend |

---

## Implementation Requirements

### 1. Cargo Configuration

Add a web-specific profile in `Cargo.toml`:

```toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
```

### 2. Conditional Compilation

The `main.rs` needs minor adjustments for web:

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Fit canvas to browser window
                fit_canvas_to_parent: true,
                canvas: Some("#game-canvas".into()),
                ..default()
            }),
            ..default()
        }))
        // ... rest of setup
        .run();
}
```

### 3. New Dependencies for WASM

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Storage"] }
console_error_panic_hook = "0.1"
```

### 4. HTML Template

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <title>Forsaken Lands</title>
    <style>
        html, body { margin: 0; padding: 0; height: 100%; overflow: hidden; }
        canvas { width: 100%; height: 100%; }
    </style>
</head>
<body>
    <canvas id="game-canvas"></canvas>
    <script type="module">
        import init from './forsaken_lands.js';
        init();
    </script>
</body>
</html>
```

---

## Build & Deployment

### Build Commands

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Build
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen --out-dir ./web --target web \
    ./target/wasm32-unknown-unknown/release/forsaken_lands.wasm
```

### Recommended Tooling

| Tool | Purpose |
|------|---------|
| [trunk](https://trunkrs.dev/) | Bevy-recommended build tool for WASM |
| wasm-opt | Binary size optimization (Binaryen) |

Using **Trunk** simplifies the process:

```bash
# Install trunk
cargo install trunk

# Development server with hot-reload
trunk serve

# Production build
trunk build --release
```

---

## Save System Considerations

Since there's no save system yet, you have flexibility in implementation:

### Option A: LocalStorage (Recommended for simple data)

```rust
#[cfg(target_arch = "wasm32")]
pub fn save_wallet(wallet: &Wallet) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage()?.unwrap();
    let json = serde_json::to_string(wallet).unwrap();
    storage.set_item("forsaken_lands_save", &json)?;
    Ok(())
}
```

### Option B: IndexedDB (For larger save files)

Use `idb` crate for async IndexedDB access — better for large save data.

### Cross-Platform Strategy

```rust
pub trait SaveBackend {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), SaveError>;
    fn load(&self, key: &str) -> Result<Vec<u8>, SaveError>;
}

#[cfg(target_arch = "wasm32")]
struct WebSaveBackend;

#[cfg(not(target_arch = "wasm32"))]
struct NativeSaveBackend;
```

---

## Performance Expectations

| Metric | Native | WASM (Estimated) |
|--------|--------|------------------|
| Startup | Fast | +1-3s (WASM/asset loading) |
| Runtime FPS | Baseline | 80-95% of native |
| Memory | Baseline | Similar (WASM has 4GB limit) |
| Binary Size | N/A | ~10-30MB (depends on assets) |

### Optimization Tips

1. **Enable LTO** — Already included in profile above
2. **Use `wasm-opt`** — Can reduce size by 10-20%
3. **Asset compression** — Serve gzipped assets from web server
4. **Lazy load assets** — Consider loading screens for large assets

---

## Effort Estimate

| Phase | Tasks | Effort |
|-------|-------|--------|
| **Setup** | Add WASM target, Trunk config, HTML template | 2-4 hours |
| **Core Changes** | Window/canvas config, conditional compilation | 2-4 hours |
| **Save System** | Implement cross-platform save abstraction | 4-8 hours |
| **Testing** | Browser testing, performance validation | 4-8 hours |
| **Polish** | Loading screens, mobile input support | 4-16 hours |

> **Total: 16-40 hours** depending on desired polish level

---

## Conclusion

### ✅ What Makes This Project WASM-Ready

1. **Clean dependency chain** — No native-only crates
2. **Pure Bevy architecture** — ECS works identically on web
3. **No file system dependencies** — Main game has no `std::fs` usage
4. **Modern Bevy version** — 0.17 has mature WASM support

### 📋 Recommended Next Steps

1. Add [Trunk.toml](file:///c:/Users/pavlicek/Documents/Projects/Rust/forsaken_lands/Trunk.toml) configuration
2. Create conditional window setup for web target
3. Test basic build with `trunk serve`
4. Plan save system architecture (LocalStorage vs IndexedDB)
5. Add mobile/touch input support if needed

---

## References

- [Bevy WASM Book](https://bevy-cheatbook.github.io/platforms/wasm.html)
- [Trunk Documentation](https://trunkrs.dev/)
- [Bevy Examples - WASM](https://github.com/bevyengine/bevy/tree/main/examples#wasm)
