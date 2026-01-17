//! Save/Load system for persisting game state.
//!
//! This crate provides:
//! - F5 keyboard shortcut for manual saves
//! - Automatic saves every 1 minute
//! - DateTime-based save file naming
//! - Scene-based serialization using Bevy's DynamicSceneBuilder

use {
    bevy::prelude::*,
    chrono::Local,
    crafting::CraftingInProgress,
    divinity_components::{Divinity, DivinityStats, MaxUnlockedDivinity},
    hero_components::{EquippedWeaponId, Hero, WeaponId},
    portal_components::Portal,
    research::{InProgress, ResearchCompletionCount},
    states::GameState,
    std::{fs, io::Write, path::Path},
    unlocks_resources::UnlockState,
    village_components::{EnemyEncyclopedia, Village, WeaponInventory},
    wallet::Wallet,
};

/// Event to trigger a game save (used with observers).
#[derive(Event)]
pub struct SaveGame;

/// Timer resource for automatic saves.
#[derive(Resource)]
pub struct AutosaveTimer(Timer);

impl Default for AutosaveTimer {
    fn default() -> Self {
        // 1 minute autosave interval
        Self(Timer::from_seconds(60.0, TimerMode::Repeating))
    }
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutosaveTimer>()
            .add_systems(
                Update,
                (trigger_save_on_keypress, autosave_tick).run_if(in_state(GameState::Running)),
            )
            .add_observer(execute_save);
    }
}

/// Triggers a save when F5 is pressed.
fn trigger_save_on_keypress(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.just_pressed(KeyCode::F5) {
        info!("Manual save triggered (F5)");
        commands.trigger(SaveGame);
    }
}

/// Ticks the autosave timer and triggers save when elapsed.
fn autosave_tick(time: Res<Time>, mut timer: ResMut<AutosaveTimer>, mut commands: Commands) {
    if timer.0.tick(time.delta()).just_finished() {
        info!("Autosave triggered");
        commands.trigger(SaveGame);
    }
}

/// Observer that handles the SaveGame event and performs the actual save.
fn execute_save(_trigger: On<SaveGame>, world: &World) {
    // Generate filename with timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("save_{}.scn.ron", timestamp);
    let saves_dir = Path::new("saves");
    let filepath = saves_dir.join(&filename);

    // Ensure saves directory exists
    if let Err(e) = fs::create_dir_all(saves_dir) {
        error!("Failed to create saves directory: {}", e);
        return;
    }

    // Build the save scene with filtered components
    let scene = build_save_scene(world);

    // Serialize the scene
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();

    let serialized = match scene.serialize(&type_registry) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize save scene: {}", e);
            return;
        }
    };

    // Write to file
    match fs::File::create(&filepath) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(serialized.as_bytes()) {
                error!("Failed to write save file: {}", e);
                return;
            }
            info!("Game saved to {}", filepath.display());
        }
        Err(e) => {
            error!("Failed to create save file: {}", e);
        }
    }
}

/// Builds a DynamicScene containing only saveable components and resources.
fn build_save_scene(world: &World) -> DynamicScene {
    DynamicSceneBuilder::from_world(world)
        // === Entity Components - ALLOW LIST ===
        // Village components
        .allow_component::<Village>()
        .allow_component::<EnemyEncyclopedia>()
        .allow_component::<WeaponInventory>()
        // Portal components
        .allow_component::<Portal>()
        .allow_component::<Divinity>()
        .allow_component::<DivinityStats>()
        .allow_component::<MaxUnlockedDivinity>()
        // Hero components
        .allow_component::<Hero>()
        .allow_component::<EquippedWeaponId>()
        .allow_component::<WeaponId>()
        // Research/Crafting state
        .allow_component::<InProgress>()
        .allow_component::<ResearchCompletionCount>()
        .allow_component::<CraftingInProgress>()
        // Hierarchy preservation
        .allow_component::<ChildOf>()
        // === Resources ===
        .allow_resource::<Wallet>()
        .allow_resource::<UnlockState>()
        // Extract all entities from the world
        .extract_entities(world.iter_entities().map(|e| e.id()))
        // Extract the allowed resources
        .extract_resources()
        // Build the scene
        .build()
}
