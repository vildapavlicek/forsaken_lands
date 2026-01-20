---
description: Bevy 0.18 API reference and migration notes from 0.17
---

# Bevy 0.18 API Notes

This document captures API changes and migration issues when upgrading from Bevy 0.17 to 0.18.

## Cargo Feature Collections (NEW in 0.18)

Bevy 0.18 introduces high-level feature "profiles" for easier project setup:

| Profile | Description |
|---------|-------------|
| `2d` | Core framework + 2D rendering + UI + scenes + audio + picking |
| `3d` | Core framework + 3D rendering + UI + scenes + audio + picking |
| `ui` | Core framework + UI rendering + scenes + audio + picking |
| `dev` | Development features (hot-reloading, debug tools) |

### Usage for 2D Games

```toml
[dependencies]
bevy = { version = "0.18", default-features = false, features = ["2d", "dev"] }
```

### What `2d` Profile Includes/Excludes

**Includes:**
- `default_app`, `default_platform`, `2d_api`, `2d_bevy_render`
- `ui`, `scene`, `audio`, `picking`
- Sprites, Text, UI, Gizmos, Animation

**Excludes (3D-only):**
- `bevy_pbr` - PBR materials and 3D rendering
- `bevy_gltf` - glTF model loading
- `bevy_light` - 3D light types (point, directional, spot)
- `bevy_mikktspace` - Vertex tangent generation
- `bevy_anti_alias` - TAA, FXAA, SMAA
- `bevy_solari` - Raytraced lighting
- `morph_animation`, `morph` - 3D morph targets
- `ktx2`, `tonemapping_luts`, `smaa_luts` - 3D texture formats

## Breaking Changes from 0.17 → 0.18

### World::iter_entities() Removed

**Old (0.17):**
```rust
#[allow(deprecated)]
fn build_save_scene(world: &World) -> DynamicScene {
    let entities = world
        .iter_entities()
        .filter(|e| e.contains::<IncludeInSave>())
        .map(|e| e.id());
    // ...
}
```

**New (0.18):**
Use a Query system parameter in observers/systems instead:
```rust
fn execute_save(
    _trigger: On<SaveGame>,
    world: &World,
    saveable_query: Query<Entity, With<IncludeInSave>>,
) {
    let saveable_entities: Vec<Entity> = saveable_query.iter().collect();
    let scene = build_save_scene(world, saveable_entities);
    // ...
}

fn build_save_scene(world: &World, entities: Vec<Entity>) -> DynamicScene {
    DynamicSceneBuilder::from_world(world)
        .extract_entities(entities.into_iter())
        // ...
}
```

**Note:** `world.query()` requires `&mut World`, so it cannot be used with `&World` in observers. Use Query system params instead.

### BorderRadius Moved into Node

**Old (0.17):**
```rust
commands.spawn((
    Node {
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    },
    BorderRadius::all(Val::Px(8.0)),  // Separate component
));
```

**New (0.18):**
```rust
commands.spawn((
    Node {
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),  // Field inside Node
        ..default()
    },
));
```

## Ecosystem Version Compatibility

When upgrading Bevy, remember to update ecosystem crates:

| Bevy Version | bevy_common_assets |
|--------------|-------------------|
| 0.17         | 0.14              |
| 0.18         | 0.15              |

## Event/Observer Pattern (unchanged from 0.17)

The Event vs Message bifurcation from 0.17 remains the same in 0.18. See `/bevy_0.17_api` for details.
