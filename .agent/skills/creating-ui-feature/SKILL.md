---
name: creating-ui-feature
description: Use when continuously adding new UI screens, menus, or HUD elements. Scaffolds a complete UI module with state management.
---

# Creating UI Feature

## Overview
Standardized pattern for creating independent UI modules in the application. Uses discrete states and observers for lifecycle management.

## When to Use
- Adding a new main menu screen
- Adding a new in-game window (e.g., Inventory, Character, Crafting)
- Adding complex HUD elements

## Instructions

### 1. Create UI Crate
Use `creating-bevy-crate` to create a new crate in `ui/`.
*Example: `ui/inventory_ui`*

### 2. Configure Dependencies
Add `widgets` and `states` to your `Cargo.toml`.
```toml
[dependencies]
bevy.workspace = true
widgets = { path = "../widgets" }
states = { path = "../../states" }
```

### 3. Define Plugin Structure
In `src/lib.rs`, implement the standard UI state pattern:

```rust
use {
    bevy::prelude::*,
    states::GameState,
    widgets::{UiTheme, spawn_action_button, spawn_item_card}, // Import common widgets
};

pub struct MyUiPlugin;

impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MyUiState>()
            .add_observer(on_ui_added)
            .add_observer(on_ui_removed)
            .add_systems(
                Update,
                (
                    handle_interactions,
                )
                // Run only when this UI is open AND game is running (if applicable)
                .run_if(in_state(MyUiState::Open).and(in_state(GameState::Running))),
            );
    }
}

// 1. Define State
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MyUiState {
    #[default]
    Closed,
    Open,
}

// 2. Define Root Component
#[derive(Component)]
pub struct MyUiRoot;

// 3. Define Events (Optional)
#[derive(Event)]
pub struct RefreshMyUiEvent;

// 4. Implement Lifecycle Observers
fn on_ui_added(
    _trigger: On<Insert, MyUiRoot>,
    mut next_state: ResMut<NextState<MyUiState>>,
) {
    next_state.set(MyUiState::Open);
}

fn on_ui_removed(
    _trigger: On<Remove, MyUiRoot>,
    mut next_state: ResMut<NextState<MyUiState>>,
) {
    next_state.set(MyUiState::Closed);
}

// 5. Implement Systems
fn handle_interactions(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    // Handle button clicks
}
```

### 4. Spawning behavior
The UI is typically spawned by a parent coordinator (like `village_ui`) spawning a `Node` with the `MyUiRoot` component. Alternatively, you can add a system to spawn it on a keypress or event.

## Common Mistakes
- **Forgetting `widgets` dependency**: creating inconsistent UI styles.
- **Not cleaning up**: The `MyUiRoot` pattern usually relies on the parent despawning the root entity.
- **Leaking systems**: Always use `.run_if(in_state(MyUiState::Open))` for UI interaction systems to prevent them from running when hidden.
