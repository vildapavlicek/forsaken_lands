mod resources;

use {
    crate::resources::{
        EnemyPrefabsFolderHandle, RecipesFolderHandle, ResearchFolderHandle, UnlocksFolderHandle,
        WeaponsFolderHandle,
    },
    bevy::{asset::LoadedFolder, platform::collections::HashMap, prelude::*},
    crafting_resources::RecipeMap,
    divinity_components::MaxUnlockedDivinity,
    enemy_components::MonsterId,
    portal_assets::SpawnTable,
    recipes_assets::RecipeDefinition,
    research::ResearchMap,
    research_assets::ResearchDefinition,
    states::{GameState, LoadingPhase},
    unlocks::{
        CompiledUnlock, ConditionSensor, LogicSignalEvent, TopicMap, UnlockRoot, UnlockState,
        compiler::{CompilerContext, build_condition_node},
    },
    unlocks_assets::UnlockDefinition,
    village_components::{EnemyEncyclopedia, Village},
    wallet::Wallet,
    weapon_assets::{WeaponDefinition, WeaponMap},
};

pub struct LoadingManagerPlugin;

impl Plugin for LoadingManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingManager>()
            .init_resource::<LoadingStatus>()
            .init_resource::<SceneToLoad>()
            .init_state::<LoadingPhase>()
            // Phase: Assets - load all asset folders
            .add_systems(
                Startup,
                (
                    load_static_assets,
                    load_enemy_prefabs,
                    load_unlocks_assets,
                    load_research_assets,
                    load_recipes_assets,
                    load_weapons_assets,
                ),
            )
            .add_systems(OnEnter(LoadingPhase::Assets), update_scene_handle)
            .add_systems(
                Update,
                check_assets_loaded
                    .run_if(in_state(GameState::Loading).and(in_state(LoadingPhase::Assets))),
            )
            // Phase: SpawnEntities - spawn research and recipe entities
            .add_systems(OnEnter(LoadingPhase::SpawnEntities), spawn_all_entities)
            // Phase: CompileUnlocks - build unlock logic graphs
            .add_systems(OnEnter(LoadingPhase::CompileUnlocks), compile_unlocks)
            // Phase: EvaluateUnlocks - re-fire signals for satisfied conditions
            .add_systems(OnEnter(LoadingPhase::EvaluateUnlocks), evaluate_unlocks)
            // Phase: SpawnScene - spawn scene (startup or save)
            .add_systems(OnEnter(LoadingPhase::SpawnScene), spawn_scene)
            .add_systems(
                Update,
                check_scene_spawned.run_if(in_state(LoadingPhase::SpawnScene)),
            )
            // Phase: Ready - transition to Running
            .add_systems(OnEnter(LoadingPhase::Ready), finish_loading)
            // Loading UI
            .add_systems(
                OnEnter(GameState::Loading),
                (setup_loading_ui, reset_loading_phase),
            )
            .add_systems(
                Update,
                update_loading_ui.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), cleanup_loading_ui);
    }
}

// --- Resources ---

#[derive(Resource)]
pub struct SceneToLoad {
    pub path: String,
    pub is_save: bool,
}

impl Default for SceneToLoad {
    fn default() -> Self {
        Self {
            path: "startup.scn.ron".to_string(),
            is_save: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct LoadingManager {
    pub startup_scene: Handle<DynamicScene>,
    pub spawn_tables: HashMap<String, Handle<SpawnTable>>,
    pub enemies: HashMap<String, Handle<DynamicScene>>,
}

// Keep GameAssets as alias for backwards compatibility
pub type GameAssets = LoadingManager;

#[derive(Resource, Default)]
pub struct LoadingStatus {
    pub current_phase: String,
    pub detail: String,
}

// --- Phase: Assets ---

fn update_scene_handle(
    mut assets: ResMut<LoadingManager>,
    asset_server: Res<AssetServer>,
    scene_to_load: Res<SceneToLoad>,
) {
    info!(
        "Starting asset load phase. target scene: {}",
        scene_to_load.path
    );
    assets.startup_scene = asset_server.load(&scene_to_load.path);
}

fn load_static_assets(mut assets: ResMut<LoadingManager>, asset_server: Res<AssetServer>) {
    info!("Loading static assets (spawn tables, etc)");
    let default_spawn_table = asset_server.load("default.spawn_table.ron");

    assets
        .spawn_tables
        .insert(String::from("default"), default_spawn_table);
}

fn load_enemy_prefabs(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("prefabs/enemies");
    cmd.insert_resource(EnemyPrefabsFolderHandle(handle));
}

fn load_unlocks_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("unlocks");
    cmd.insert_resource(UnlocksFolderHandle(handle));
}

fn load_research_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("research");
    cmd.insert_resource(ResearchFolderHandle(handle));
}

fn load_recipes_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("recipes");
    cmd.insert_resource(RecipesFolderHandle(handle));
}

fn load_weapons_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("weapons");
    cmd.insert_resource(WeaponsFolderHandle(handle));
}

#[allow(clippy::too_many_arguments)]

fn check_assets_loaded(
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut loading_manager: ResMut<LoadingManager>,
    mut weapon_map: ResMut<WeaponMap>,
    mut status: ResMut<LoadingStatus>,
    asset_server: Res<AssetServer>,
    enemy_prefabs: Res<EnemyPrefabsFolderHandle>,
    unlocks: Res<UnlocksFolderHandle>,
    research: Res<ResearchFolderHandle>,
    recipes: Res<RecipesFolderHandle>,
    weapons: Res<WeaponsFolderHandle>,
    folder: Res<Assets<LoadedFolder>>,
    weapon_assets: Res<Assets<WeaponDefinition>>,
    scenes: Res<Assets<DynamicScene>>,
    type_registry: Res<AppTypeRegistry>,
) {
    status.current_phase = "Loading Assets".into();
    status.detail = "Loading files from disk...".into();

    let spawn_tables_loaded = loading_manager
        .spawn_tables
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    // For the scene file, we only need to check if it's loaded.
    // However, if we are loading a save, we might have already loaded assets in a previous run.
    // But `asset_server.is_loaded_with_dependencies` is generally cheap if already loaded.
    if asset_server.is_loaded_with_dependencies(&loading_manager.startup_scene)
        && spawn_tables_loaded
        && asset_server.is_loaded_with_dependencies(enemy_prefabs.0.id())
        && asset_server.is_loaded_with_dependencies(unlocks.0.id())
        && asset_server.is_loaded_with_dependencies(research.0.id())
        && asset_server.is_loaded_with_dependencies(recipes.0.id())
        && asset_server.is_loaded_with_dependencies(weapons.0.id())
    {
        info!("assets loaded");

        let Some(enemy_prefabs_folder) = folder.get(enemy_prefabs.0.id()) else {
            panic!("folder not loaded even tho asset server said it is")
        };

        for untyped_handle in enemy_prefabs_folder.handles.iter().cloned() {
            let Some(asset_path) = asset_server.get_path(untyped_handle.id()) else {
                continue;
            };

            let path = asset_path.path().display().to_string();

            let Ok(handle) = untyped_handle.try_typed::<DynamicScene>() else {
                continue;
            };

            // Extract MonsterId from the loaded scene
            let key = extract_monster_id(&scenes, &handle, &type_registry).unwrap_or_else(|| {
                panic!("MonsterId component not found in enemy prefab: {}", path)
            });

            debug!(%key, %path, "loaded enemy prefab with MonsterId");
            loading_manager.enemies.insert(key, handle);
        }

        // Populate WeaponMap from loaded weapon assets
        let Some(weapons_folder) = folder.get(weapons.0.id()) else {
            panic!("weapons folder not loaded even though asset server said it is")
        };

        for untyped_handle in weapons_folder.handles.iter().cloned() {
            let Ok(handle) = untyped_handle.try_typed::<WeaponDefinition>() else {
                continue;
            };

            if let Some(def) = weapon_assets.get(&handle) {
                debug!("Loaded weapon definition: {}", def.id);
                weapon_map.handles.insert(def.id.clone(), handle);
            }
        }

        next_phase.set(LoadingPhase::SpawnScene);
    }
}

/// Extracts the MonsterId component value from a loaded DynamicScene.
fn extract_monster_id(
    scenes: &Assets<DynamicScene>,
    handle: &Handle<DynamicScene>,
    _type_registry: &AppTypeRegistry,
) -> Option<String> {
    let scene = scenes.get(handle)?;

    for entity in &scene.entities {
        for component in &entity.components {
            // Check if this component is a MonsterId
            let type_info = component.try_as_reflect()?.get_represented_type_info()?;
            if type_info.type_id() == std::any::TypeId::of::<MonsterId>() {
                // MonsterId is a tuple struct: MonsterId(String)
                // Access field 0 to get the inner String
                if let bevy::reflect::ReflectRef::TupleStruct(ts) = component.reflect_ref() {
                    if let Some(field) = ts.field(0) {
                        if let Some(s) = field.try_downcast_ref::<String>() {
                            return Some(s.clone());
                        }
                    }
                }
            }
        }
    }

    None
}

// --- Phase: SpawnEntities ---

#[allow(clippy::too_many_arguments)]
fn spawn_all_entities(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut recipe_map: ResMut<RecipeMap>,
    mut research_assets: ResMut<Assets<ResearchDefinition>>,
    mut recipe_assets: ResMut<Assets<RecipeDefinition>>,
    unlock_state: Res<UnlockState>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Spawning Entities".into();
    status.detail = "Creating research and recipe nodes...".into();

    // Spawn research entities
    debug!("Spawning research entities...");
    let research_ids: Vec<_> = research_assets.ids().collect();

    for id in research_ids {
        let def_id = {
            let Some(def) = research_assets.get(id) else {
                continue;
            };

            if research_map.entities.contains_key(&def.id) {
                continue;
            }

            def.id.clone()
        };

        let already_unlocked = unlock_state.completed.iter().any(|unlock_id| {
            unlock_id.ends_with(&format!("{}_unlock", def_id))
                || unlock_id.starts_with(&format!("research_{}", def_id))
        });

        let handle = research_assets.get_strong_handle(id).unwrap();

        let entity = if already_unlocked {
            debug!(
                "Research '{}' unlock already achieved, spawning as Available",
                def_id
            );
            commands
                .spawn((
                    research::ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    research::Available,
                    research::ResearchCompletionCount(0),
                ))
                .id()
        } else {
            commands
                .spawn((
                    research::ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    research::Locked,
                    research::ResearchCompletionCount(0),
                ))
                .id()
        };

        research_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned research entity: {} -> {:?}", def_id, entity);
    }

    // Spawn recipe entities
    debug!("Spawning recipe entities...");
    crafting::spawn_recipe_entities(
        &mut commands,
        &mut recipe_map,
        &mut recipe_assets,
        &unlock_state,
    );

    next_phase.set(LoadingPhase::CompileUnlocks);
}

// --- Phase: CompileUnlocks ---

fn compile_unlocks(
    mut commands: Commands,
    unlock_assets: Res<Assets<UnlockDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    wallet: Res<Wallet>,
    encyclopedia_query: Query<&EnemyEncyclopedia, With<Village>>,
    unlock_state: Res<UnlockState>,
    max_divinity_query: Query<&MaxUnlockedDivinity>,
    compiled: Query<&CompiledUnlock>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Compiling Unlocks".into();
    status.detail = "Building logic graphs...".into();

    let encyclopedia = encyclopedia_query.single().ok();
    let max_divinity = max_divinity_query.iter().next();

    let ctx = CompilerContext {
        wallet: &wallet,
        encyclopedia,
        unlock_state: &unlock_state,
        max_divinity,
    };

    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    let pending_definitions = unlock_assets
        .iter()
        .map(|(_, def)| def)
        .filter(|def| !compiled_ids.contains(def.id.as_str()))
        .filter(|def| !unlock_state.is_unlocked(&def.id));

    for definition in pending_definitions {
        debug!(unlock_id = %definition.id, "Compiling unlock definition");

        let root = commands
            .spawn((
                UnlockRoot {
                    id: definition.id.clone(),
                    display_name: definition.display_name.clone(),
                    reward_id: definition.reward_id.clone(),
                },
                CompiledUnlock {
                    definition_id: definition.id.clone(),
                },
            ))
            .id();

        build_condition_node(
            &mut commands,
            &mut topic_map,
            &definition.condition,
            root,
            &ctx,
        );
    }

    next_phase.set(LoadingPhase::EvaluateUnlocks);
}

// --- Phase: EvaluateUnlocks ---

/// After all unlock logic graphs are compiled, re-fire signals for sensors that are already satisfied.
/// This ensures that unlock cascades happen correctly, since all observers now exist.
fn evaluate_unlocks(
    mut commands: Commands,
    sensors: Query<(Entity, &ConditionSensor)>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Evaluating Unlocks".into();
    status.detail = "Checking satisfied conditions...".into();

    // Re-fire LogicSignalEvent for all sensors that are already satisfied
    // This triggers the unlock cascade now that all observers exist
    let mut satisfied_count = 0;
    for (entity, sensor) in &sensors {
        if sensor.is_met {
            debug!(sensor = ?entity, "Re-firing signal for satisfied sensor");
            commands.entity(entity).trigger(|e| LogicSignalEvent {
                entity: e,
                is_high: true,
            });
            satisfied_count += 1;
        }
    }
    info!("Evaluated {} sensors, {} already satisfied", sensors.iter().count(), satisfied_count);

    next_phase.set(LoadingPhase::PostLoadReconstruction);
}

// --- Phase: SpawnScene ---

fn spawn_scene(
    mut scene_spawner: ResMut<SceneSpawner>,
    loading_manager: Res<LoadingManager>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Spawning Scene".into();
    status.detail = "Loading world...".into();

    info!("spawning scene");
    scene_spawner.spawn_dynamic(loading_manager.startup_scene.clone());
}

fn check_scene_spawned(
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    query: Query<(), With<Village>>,
    scene_to_load: Res<SceneToLoad>,
) {
    if !query.is_empty() {
        info!("Scene spawned and validated");
        if scene_to_load.is_save {
            info!("Save loaded - now spawning entities with loaded EnemyEncyclopedia");
        } else {
            info!("New game - now spawning entities");
        }
        next_phase.set(LoadingPhase::SpawnEntities);
    }
}

// --- Phase: Ready ---

fn finish_loading(mut next_state: ResMut<NextState<GameState>>) {
    info!("Loading complete, transitioning to Running");
    next_state.set(GameState::Running);
}

// --- Loading UI ---

#[derive(Component)]
struct LoadingUi;

fn setup_loading_ui(mut commands: Commands) {
    info!("spawning loading ui");
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        LoadingUi,
    ));
}

fn update_loading_ui(status: Res<LoadingStatus>, mut query: Query<&mut Text, With<LoadingUi>>) {
    if let Ok(mut text) = query.single_mut() {
        *text = Text::new(format!(
            "Loading...\n{}\n{}",
            status.current_phase, status.detail
        ));
    }
}

fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingUi>>) {
    info!("cleaning up loading ui");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn reset_loading_phase(mut next_phase: ResMut<NextState<LoadingPhase>>) {
    info!("Resetting LoadingPhase to Assets");
    next_phase.set(LoadingPhase::Assets);
}
