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
    hero_components::{EquippedWeaponId, Hero, WeaponId},
    portal_components::Portal,
    research::{InProgress, ResearchCompletionCount, ResearchMap},
    states::{GameState, LoadingSavePhase},
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

/// Holds the handle to the save scene being loaded.
#[derive(Resource, Default)]
pub struct LoadingSaveHandle(pub Option<Handle<DynamicScene>>);

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
            .init_resource::<LoadingSaveHandle>()
            .add_sub_state::<LoadingSavePhase>()
            // Save systems (only in Running state)
            .add_systems(
                Update,
                (trigger_save_on_keypress, trigger_load_on_keypress, autosave_tick)
                    .run_if(in_state(GameState::Running)),
            )
            .add_observer(execute_save)
            .add_observer(execute_load)
            // Reconstruction phases
            .add_systems(
                Update,
                reconstruction::wait_for_scene_loaded
                    .run_if(in_state(LoadingSavePhase::WaitingForSceneSpawn)),
            )
            .add_systems(
                OnEnter(LoadingSavePhase::ReconstructingWeapons),
                reconstruction::reconstruct_weapons_from_inventory,
            )
            .add_systems(
                OnEnter(LoadingSavePhase::RebuildingMaps),
                reconstruction::rebuild_research_recipe_maps,
            )
            .add_systems(
                OnEnter(LoadingSavePhase::RelinkingResearch),
                reconstruction::relink_in_progress_research,
            )
            .add_systems(
                OnEnter(LoadingSavePhase::ReconstructingRates),
                reconstruction::reconstruct_resource_rates,
            )
            .add_systems(
                OnEnter(LoadingSavePhase::Complete),
                finish_loading_save,
            );
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

/// Observer that handles the LoadGame event.
fn execute_load(
    _trigger: On<LoadGame>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut save_handle: ResMut<LoadingSaveHandle>,
    mut next_state: ResMut<NextState<GameState>>,
    // Entities to despawn
    villages: Query<Entity, With<Village>>,
    portals: Query<Entity, With<Portal>>,
    heroes: Query<Entity, With<Hero>>,
    research_nodes: Query<Entity, With<research::ResearchNode>>,
    recipe_nodes: Query<Entity, With<crafting::RecipeNode>>,
    unlock_roots: Query<Entity, With<UnlockRoot>>,
    compiled_unlocks: Query<Entity, With<CompiledUnlock>>,
    // Resources to clear
    mut research_map: ResMut<ResearchMap>,
    mut recipe_map: ResMut<RecipeMap>,
) {
    let saves_dir = Path::new("saves");

    // Find the latest save file
    let latest_save = match find_latest_save(saves_dir) {
        Some(path) => path,
        None => {
            warn!("No save files found in saves directory");
            return;
        }
    };

    info!("Loading save file: {}", latest_save.display());

    // Despawn all existing game entities (recursive despawn for hierarchies)
    for entity in villages.iter() {
        commands.entity(entity).despawn();
    }
    for entity in portals.iter() {
        commands.entity(entity).despawn();
    }
    for entity in heroes.iter() {
        // Heroes might already be despawned as children of village
        if commands.get_entity(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }
    for entity in research_nodes.iter() {
        commands.entity(entity).despawn();
    }
    for entity in recipe_nodes.iter() {
        commands.entity(entity).despawn();
    }
    for entity in unlock_roots.iter() {
        commands.entity(entity).despawn();
    }
    for entity in compiled_unlocks.iter() {
        if commands.get_entity(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }

    // Clear the lookup maps
    research_map.entities.clear();
    recipe_map.entities.clear();

    // Load the save scene
    let handle: Handle<DynamicScene> = asset_server.load(latest_save);
    save_handle.0 = Some(handle);

    // Transition to LoadingSave state
    info!("Transitioning to LoadingSave state");
    next_state.set(GameState::LoadingSave);
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

/// Finishes the load process and transitions back to Running state.
fn finish_loading_save(
    mut next_state: ResMut<NextState<GameState>>,
    mut save_handle: ResMut<LoadingSaveHandle>,
) {
    info!("Load reconstruction complete, transitioning to Running");
    save_handle.0 = None;
    next_state.set(GameState::Running);
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

