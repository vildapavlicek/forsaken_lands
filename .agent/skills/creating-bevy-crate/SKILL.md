---
name: creating-bevy-crate
description: Use when adding a new Rust crate (module) to the Bevy project workspace, such as a new UI feature, game system, or asset library.
---

# Creating Bevy Crate

## Overview
Standardized process for creating new crates in the workspace. Ensures consistent dependency management and plugin structure.

## When to Use
- Adding a new UI feature (e.g., `ui/new_feature`)
- Adding a new game system (e.g., `game_systems/mechanic`)
- Adding a new asset library (e.g., `game_assets/new_assets`)

## Instructions

### 1. Generate Crate
Run the following command from the workspace root:
```powershell
cargo new {path/to/crate_name} --lib
```
*Example: `cargo new ui/inventory_ui --lib`*

### 2. Update Workspace
Add the new crate to `[workspace] members` in the root `Cargo.toml`.
```toml
[workspace]
members = [
  "path/to/crate_name",
  # ... other members
]
```

### 3. Configure Dependencies
Overwrite `{path/to/crate_name}/Cargo.toml` with the standard template:

```toml
[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy.workspace = true
# Add other workspace dependencies as needed:
# shared_components = { path = "../../components/shared_components" }
```

### 4. Create Plugin
Overwrite `{path/to/crate_name}/src/lib.rs` with the standard Plugin template:

```rust
use bevy::prelude::*;

pub struct {CrateName}Plugin;

impl Plugin for {CrateName}Plugin {
    fn build(&self, app: &mut App) {
        // Register systems, resources, and types here
        // app.add_systems(Update, example_system);
    }
}
```

## Common Mistakes
- **Forgetting `bevy.workspace = true`**: Manual versioning breaks the build.
- **Wrong path in root `Cargo.toml`**: Cargo won't pick up the member.
- **Not defining a Plugin**: Bevy relies on Plugins for modularity.
