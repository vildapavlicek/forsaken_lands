---
name: creating-game-asset
description: Use when adding a new game asset type (RON-based) to the project, such as items, monster stats, or configurations.
---

# Creating Game Asset (RON)

## Overview
Standardized pattern for creating new data-driven assets using RON files. leveraging `bevy_common_assets`.

## When to Use
- Adding new configuration data (e.g., spawn tables, loot tables)
- Definining new entity types (e.g., monster definitions, item definitions)
- Any time you need hot-reloadable data files.

## Instructions

### 1. Create Asset Crate
Use the `creating-bevy-crate` skill to create a new crate (e.g., `game_assets/my_new_assets`).

### 2. Configure Dependencies
Ensure `Cargo.toml` has the necessary dependencies:

```toml
[dependencies]
bevy.workspace = true
bevy_common_assets = { version = "0.15", features = ["ron"] }
serde = { version = "1.0", features = ["derive"] }
```

### 3. Define Asset Structure
In `src/lib.rs`, define your asset struct. It MUST derive `Asset`, `TypePath`, `Debug`, `Deserialize`, and `Serialize`.

```rust
use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    serde::{Deserialize, Serialize},
};

pub struct MyNewAssetsPlugin;

impl Plugin for MyNewAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Register the asset loader for .mydata.ron files
        // The file extension MUST match what you use in assets/
        app.add_plugins(RonAssetPlugin::<MyDataAsset>::new(&["mydata.ron"]));
    }
}

#[derive(Asset, TypePath, Debug, Deserialize, Serialize)]
pub struct MyDataAsset {
    pub name: String,
    pub value: u32,
    // Add other fields here
}
```

### 4. Create Asset File
Create the actual data file in `assets/` (e.g., `assets/examples.mydata.ron`).

```ron
(
    name: "Example Data",
    value: 42,
)
```

## Common Mistakes
- **Forgetting `TypePath`**: Bevy assets require `TypePath` for reflection and identification.
- **Wrong feature in `bevy_common_assets`**: Ensure `features = ["ron"]` is enabled.
- **Mismatched file extension**: The string passed to `RonAssetPlugin::new` acts as a filter for file extensions.
