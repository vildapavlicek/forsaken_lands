---
name: save-system-design
description: Guidelines for the Save/Load architecture, including how to mark entities for persistence and how the game state is reconstructed.
---

# Save/Load System Design

This document describes the architecture of the Save/Load system in the Forsaken Lands project. The system is designed to persist only the dynamic game state while reconstructing the static game world from assets upon loading.

## Overview

The save system uses Bevy's `DynamicScene` to serialize entities and components to a RON file.
-   **Save Format**: RON (Rusty Object Notation)
-   **Location**: `saves/` directory
-   **Auto-save**: Every 1 minute
-   **Manual Save**: F5
-   **Quick Load**: F9 (latest save), F8 (latest autosave)

## Core Principle: Static vs. Dynamic

To keep save files small and robust, we **do not** save everything.
-   **Assets are Static**: Data defined in `assets/` (items, monsters, research trees, recipes) is **never** saved. It is re-loaded from disk every time the game starts.
-   **State is Dynamic**: Only the changes to the world (inventory, unlock status, current health, position) are saved.

## How to Mark Entities for Saving

To include an entity in the save file, you must add the `IncludeInSave` marker component to it.

```rust
use shared_components::IncludeInSave;

#[derive(Component, Default)]
#[require(IncludeInSave)] // Best Practice: Add this to your root components
pub struct MyFeatureRoot;
```

**Rule**: If an entity does not have `IncludeInSave`, it will be **destroyed** on level unload and **not** present in the save file.

### What Components to Save/exclude
The `save_load` crate automatically denies specific Bevy internal components (like `GlobalTransform`, `ViewVisibility`) to avoid bloat and conflicts.
You should generally save:
-   Components that change during gameplay (Health, Inventory, Progress).
-   Marker components that identify the entity (e.g., `Hero`, `Village`).

You should generally **NOT** save:
-   Handles to assets (unless strictly necessary and stable).
-   UI entities (these should be rebuilt by UI systems).
-   VFX/Particles.

## The Loading Process

Loading a save file is not just deserializing the file. It is a multi-stage reconstruction process managed by the `loading` crate phases.

### 1. Asset Phase (`LoadingPhase::Assets`)
-   Loads all static assets (RON files) from `assets/` folders (definitions for items, skills, etc.).
-   This happens *before* any entity spawning.

### 2. Entity Spawning (`LoadingPhase::SpawnEntities`)
-   Systems iterate over the *loaded assets* and spawn the "static" entities of the world.
-   **Example**: Research nodes are spawned based on `ResearchDefinition` assets.
-   **Example**: Recipes are spawned based on `RecipeDefinition` assets.
-   At this stage, these entities are in their "default" state (e.g., all research is Locked).

### 3. Scene Spawning (`LoadingPhase::SpawnScene`)
-   The save file is deserialized and spawned into the world.
-   This overwrites/updates existing entities or spawns new ones (like the Hero or Village).
-   **Note**: Since we use UUID-like identification for some systems (like component-based IDs), the saved data must correctly map to the potential static entities.

### 4. Evaluation & Reconstruction (`LoadingPhase::EvaluateUnlocks` & `PostLoadReconstruction`)
This is the critical "Hydration" step where saved state meets static assets.

#### Unlocks Hydration
The `UnlockSystem` relies on event streams (signals) to update its state graphs. Since events are ephemeral and not saved, we must **re-trigger** them based on saved data.
-   **Research**: The system checks `ResearchState` (saved resource). For every completed research, it fires a `StatusCompleted` event. This updates the unlock graph to unlock dependent items.
-   **Divinity**: Similar re-triggering for claimed divinity nodes.
-   **Stats/Counts**: `Wallet` resources and `Encyclopedia` counts fire `ValueChanged` events to update progress sensors.

#### Entity Reconstruction
Some entities are too complex to save directly or need to be re-assembled.
-   **Weapons**: Weapons are saved as `WeaponInventory` (a list of IDs) on the Village/Hero. We do *not* save every weapon entity.
    -   `reconstruct_weapons_from_inventory` queries the inventory and spawns the actual weapon entities (scenes) again.
-   **In-Progress Research**: Re-links the saved `InProgress` component (which might be orphaned or on a placeholder) to the actual `ResearchNode` entity spawned in step 2.

## Checklist for Adding New Saveable Features

1.  [ ] **Data Component**: Define your component in a shared crate.
2.  [ ] **Marker**: Add `#[require(IncludeInSave)]` if it's a root entity, or ensure the entity has it.
3.  [ ] **Resources**: If you need to save a global Resource, add `.allow_resource::<MyResource>()` in `save_load::lib.rs`.
4.  [ ] **Reconstruction**:
    -   Does your feature rely on assets?
    -   Does it need to "hook up" saved data to static entities?
    -   If yes, add a system to `LoadingPhase::PostLoadReconstruction` in `save_load::reconstruction` or your own plugin.
5.  [ ] **Testing**:
    -   Run game -> Change state -> F5 (Save) -> Stop Game.
    -   Start Game -> F9 (Load).
    -   Verify state is restored AND functional (buttons work, stats update).
