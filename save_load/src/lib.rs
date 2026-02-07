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
    states::{GameState, LoadingPhase},
    std::{fs, io::Write, path::Path},
    wallet::Wallet,
};

/// Event to trigger loading the latest save file.
#[derive(Event)]
pub struct LoadGame {
    is_autosave: bool,
}

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
            // Save systems (only in Running state)
            .add_systems(
                Update,
                trigger_load_on_keypress.run_if(in_state(GameState::Running)),
            )
            .add_systems(
                PostUpdate,
                execute_save.run_if(in_state(GameState::Running)),
            )
            .add_observer(execute_load)
            // Reconstruction phases - Unified Loading
            .add_systems(
                OnEnter(LoadingPhase::PostLoadReconstruction),
                (
                    reconstruction::reconstruct_weapons_from_inventory,
                    reconstruction::relink_in_progress_research,
                    reconstruction::hydrate_research_unlocks,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(LoadingPhase::PostLoadReconstruction),
                reconstruction::finish_reconstruction
                    .after(reconstruction::hydrate_research_unlocks),
            )
            .add_systems(OnExit(GameState::Running), clean_up_save_load);
    }
}

/// Exclusive system that handles manual and automatic saves.
fn execute_save(world: &mut World) {
    let mut is_autosave = false;
    let mut manual_triggered = false;

    // 1. Check Manual Save (F5)
    if let Some(keyboard) = world.get_resource::<ButtonInput<KeyCode>>()
        && keyboard.just_pressed(KeyCode::F5) {
            info!("Manual save triggered (F5)");
            manual_triggered = true;
            is_autosave = false;
        }

    // 2. Check Autosave Timer
    if !manual_triggered {
        let delta = world.get_resource::<Time>().map(|t| t.delta());
        if let Some(delta) = delta {
            if let Some(mut timer) = world.get_resource_mut::<AutosaveTimer>() {
                if timer.0.tick(delta).just_finished() {
                    info!("Autosave triggered");
                    is_autosave = true;
                } else {
                    return; // No save triggered
                }
            } else {
                return;
            }
        } else {
            return;
        }
    } else {
        // Reset autosave timer on manual save to avoid back-to-back saves
        if let Some(mut timer) = world.get_resource_mut::<AutosaveTimer>() {
            timer.0.reset();
        }
    }

    // 3. Process Save
    let filename = if is_autosave {
        "autosave.scn.ron".to_string()
    } else {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        format!("save_{}.scn.ron", timestamp)
    };

    let saves_dir = Path::new("saves");
    let filepath = saves_dir.join(&filename);

    if let Err(e) = fs::create_dir_all(saves_dir) {
        error!("Failed to create saves directory: {}", e);
        return;
    }

    // Collect saveable entities
    let mut query = world.query_filtered::<Entity, (
        With<shared_components::IncludeInSave>,
        Without<hero_components::Weapon>,
    )>();
    let saveable_entities: Vec<Entity> = query.iter(world).collect();

    let scene = build_save_scene(world, saveable_entities);

    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let serialized = match scene.serialize(&type_registry) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize save scene: {}", e);
            return;
        }
    };

    match fs::File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&filepath)
    {
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

/// Triggers a load when F9 is pressed.
fn trigger_load_on_keypress(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.just_pressed(KeyCode::F9) {
        info!("Load triggered (F9)");
        commands.trigger(LoadGame { is_autosave: false });
    }

    if keyboard.just_pressed(KeyCode::F8) {
        info!("Load triggered (F8)");
        commands.trigger(LoadGame { is_autosave: true });
    }
}

/// Observer that handles the LoadGame event.
fn execute_load(
    trigger: On<LoadGame>,
    _commands: Commands,
    mut scene_to_load: ResMut<loading::SceneToLoad>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let LoadGame { is_autosave } = trigger.event();
    let saves_dir = Path::new("saves");

    // Find the latest save file

    let latest_save = if *is_autosave {
        Path::new("saves/autosave.scn.ron").into()
    } else {
        match find_latest_save(saves_dir) {
            Some(path) => path,
            None => {
                warn!("No save files found in saves directory");
                return;
            }
        }
    };

    info!("Loading save file: {}", latest_save.display());

    // Despawn/Cleanup is now handled by OnExit(GameState::Running) systems in each plugin.
    info!("Manual cleanup delegated to OnExit(GameState::Running) systems");

    // Configure loading state
    let relative_path = latest_save.strip_prefix("saves").unwrap_or(&latest_save);
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

/// Builds a DynamicScene containing saveable components and resources.
///
/// Uses IncludeInSave marker to explicitly include only entities we want to save.
/// Components with #[require(IncludeInSave)] automatically get included.
fn build_save_scene(world: &World, saveable_entities: Vec<Entity>) -> DynamicScene {
    DynamicSceneBuilder::from_world(world)
        // === DENY-LIST: Bevy internal components that don't serialize cleanly ===
        .deny_component::<InheritedVisibility>()
        .deny_component::<ViewVisibility>()
        .deny_component::<GlobalTransform>()
        .deny_component::<bevy::camera::visibility::VisibilityClass>()
        .deny_component::<bevy::render::sync_world::RenderEntity>()
        .deny_component::<bevy::render::sync_world::SyncToRenderWorld>()
        .deny_component::<bevy::camera::primitives::Aabb>()
        // === Resources ===
        .allow_resource::<Wallet>()
        .allow_resource::<research::ResearchState>()
        .allow_resource::<village_resources::DivinityUnlockState>()
        .allow_resource::<crafting_resources::ConstructedBuildings>()
        // === Entity extraction ===
        // Only include entities marked with IncludeInSave
        .extract_entities(saveable_entities.into_iter())
        .extract_resources()
        .build()
}

pub fn clean_up_save_load(mut timer: ResMut<AutosaveTimer>) {
    // Reset timer to default (1 minute)
    *timer = AutosaveTimer::default();
}
