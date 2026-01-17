//! Save/Load system for persisting game state.
//!
//! This crate provides:
//! - F5 keyboard shortcut for manual saves
//! - F9 keyboard shortcut to load the latest save
//! - Automatic saves every 1 minute
//! - DateTime-based save file naming
//! - Scene-based serialization using Bevy's DynamicSceneBuilder

mod reconstruction;

use {
    bevy::prelude::*,
    chrono::Local,
    crafting::CraftingInProgress,
    crafting_resources::RecipeMap,
    divinity_components::{Divinity, DivinityStats, MaxUnlockedDivinity},
    enemy_components::{
        Enemy, EnemyRange, Health, Lifetime, MonsterId, MovementSpeed, ResourceRewards,
        TargetDestination,
    },
    hero_components::{EquippedWeaponId, Hero, WeaponId},
    loading::SceneToLoad,
    portal_components::{Portal, SpawnTableId, SpawnTimer},
    research::{InProgress, ResearchCompletionCount, ResearchMap},
    states::{GameState, LoadingPhase},
    std::{fs, io::Write, path::Path},
    unlocks::{CompiledUnlock, UnlockRoot},
    unlocks_resources::UnlockState,
    village_components::{EnemyEncyclopedia, Village, WeaponInventory},
    wallet::Wallet,
};

/// Event to trigger a game save (used with observers).
#[derive(Event)]
pub struct SaveGame;

/// Event to trigger loading the latest save file.
#[derive(Event)]
pub struct LoadGame;

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
            // Save systems (only in Running state)
            .add_systems(
                Update,
                (
                    trigger_save_on_keypress,
                    trigger_load_on_keypress,
                    //  disable auto save for testing purposes
                    // autosave_tick
                )
                    .run_if(in_state(GameState::Running)),
            )
            .add_observer(execute_save)
            .add_observer(execute_load)
            // Reconstruction phases - Unified Loading
            .add_systems(
                OnEnter(LoadingPhase::PostLoadReconstruction),
                (
                    reconstruction::reconstruct_weapons_from_inventory,
                    reconstruction::relink_in_progress_research,
                    reconstruction::reconstruct_resource_rates,
                ).chain(),
            )
            .add_systems(
                OnEnter(LoadingPhase::PostLoadReconstruction),
                reconstruction::finish_reconstruction.after(reconstruction::reconstruct_resource_rates),
            )
            .add_systems(OnExit(GameState::Running), clean_up_save_load);
    }
}

/// Triggers a save when F5 is pressed.
fn trigger_save_on_keypress(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.just_pressed(KeyCode::F5) {
        info!("Manual save triggered (F5)");
        commands.trigger(SaveGame);
    }
}

/// Triggers a load when F9 is pressed.
fn trigger_load_on_keypress(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.just_pressed(KeyCode::F9) {
        info!("Load triggered (F9)");
        commands.trigger(LoadGame);
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
    let saves_dir = Path::new("assets/saves");
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

/// Observer that handles the LoadGame event.
fn execute_load(
    _trigger: On<LoadGame>,
    _commands: Commands,
    mut scene_to_load: ResMut<loading::SceneToLoad>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let saves_dir = Path::new("assets/saves");

    // Find the latest save file
    let latest_save = match find_latest_save(saves_dir) {
        Some(path) => path,
        None => {
            warn!("No save files found in saves directory");
            return;
        }
    };

    info!("Loading save file: {}", latest_save.display());

    // Despawn/Cleanup is now handled by OnExit(GameState::Running) systems in each plugin.
    info!("Manual cleanup delegated to OnExit(GameState::Running) systems");


    // Configure loading state
    let relative_path = latest_save.strip_prefix("assets").unwrap_or(&latest_save);
    scene_to_load.path = relative_path.to_string_lossy().to_string();
    scene_to_load.is_save = true;

    // Transition to unified Loading state
    info!("Transitioning to unified Loading state");
    next_state.set(GameState::Loading);
}

/// Finds the most recent save file in the saves directory.
fn find_latest_save(saves_dir: &Path) -> Option<std::path::PathBuf> {
    let entries = fs::read_dir(saves_dir).ok()?;

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "ron")
                .unwrap_or(false)
        })
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("save_"))
                .unwrap_or(false)
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

/// Builds a DynamicScene containing only saveable components and resources.
#[allow(deprecated)] // iter_entities - no mutable alternative available here
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
        // Positioning
        .allow_component::<Transform>()
        .allow_component::<GlobalTransform>()
        // Visibility
        .allow_component::<Visibility>()
        .allow_component::<InheritedVisibility>()
        .allow_component::<ViewVisibility>()
        .allow_component::<Sprite>()
        // Portal state & timers
        .allow_component::<SpawnTimer>()
        .allow_component::<SpawnTableId>()
        // Enemy state & timers
        .allow_component::<Enemy>()
        .allow_component::<Lifetime>()
        .allow_component::<Health>()
        .allow_component::<MovementSpeed>()
        .allow_component::<MonsterId>()
        .allow_component::<EnemyRange>()
        .allow_component::<TargetDestination>()
        .allow_component::<ResourceRewards>()
        // Hierarchy preservation
        .allow_component::<ChildOf>()
        // === Resources ===
        .allow_resource::<Wallet>()
        .allow_resource::<UnlockState>()
        // Extract all entities from the world, except those with Weapon component
        .extract_entities(
            world
                .iter_entities()
                .filter(|e| !e.contains::<hero_components::Weapon>())
                .map(|e| e.id()),
        )
        // Extract the allowed resources
        .extract_resources()
        // Build the scene
        .build()
}

pub fn clean_up_save_load(mut timer: ResMut<AutosaveTimer>) {
    // Reset timer to default (1 minute)
    *timer = AutosaveTimer::default();
}

