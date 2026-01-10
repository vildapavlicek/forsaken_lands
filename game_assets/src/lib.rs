mod resources;

use {
    crate::resources::{EnemyPrefabsFolderHandle, ResearchFolderHandle, UnlocksFolderHandle},
    bevy::{asset::LoadedFolder, platform::collections::HashMap, prelude::*},
    portal_assets::SpawnTable,
    research::{ResearchDefinition, ResearchMap},
    states::{GameState, LoadingPhase},
    std::ffi::OsStr,
    unlocks::{
        compiler::{build_condition_node, CompilerContext},
        CompiledUnlock, TopicMap, UnlockDefinition, UnlockRoot, UnlockState,
    },
    village_components::{EnemyEncyclopedia, Village},
    wallet::Wallet,
};


pub struct LoadingManagerPlugin;

impl Plugin for LoadingManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingManager>()
            .init_resource::<LoadingStatus>()
            .init_state::<LoadingPhase>()
            // Phase: Assets - load all asset folders
            .add_systems(
                Startup,
                (
                    start_loading,
                    load_enemy_prefabs,
                    load_unlocks_assets,
                    load_research_assets,
                ),
            )
            .add_systems(
                Update,
                check_assets_loaded
                    .run_if(in_state(GameState::Loading).and(in_state(LoadingPhase::Assets))),
            )
            // Phase: SpawnEntities - spawn research entities
            .add_systems(OnEnter(LoadingPhase::SpawnEntities), spawn_all_entities)
            // Phase: CompileUnlocks - build unlock logic graphs
            .add_systems(OnEnter(LoadingPhase::CompileUnlocks), compile_unlocks)
            // Phase: SpawnScene - spawn startup scene
            .add_systems(OnEnter(LoadingPhase::SpawnScene), spawn_startup_scene)
            .add_systems(
                Update,
                check_scene_spawned.run_if(in_state(LoadingPhase::SpawnScene)),
            )
            // Phase: Ready - transition to Running
            .add_systems(OnEnter(LoadingPhase::Ready), finish_loading)
            // Loading UI
            .add_systems(OnEnter(GameState::Loading), setup_loading_ui)
            .add_systems(
                Update,
                update_loading_ui.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), cleanup_loading_ui);
    }
}

// --- Resources ---

#[derive(Resource, Default)]
pub struct LoadingManager {
    pub startup_scene: Handle<DynamicScene>,
    pub recipes_library_scene: Handle<DynamicScene>,
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

fn start_loading(mut assets: ResMut<LoadingManager>, asset_server: Res<AssetServer>) {
    info!("started loading assets");
    assets.startup_scene = asset_server.load("startup.scn.ron");
    assets.recipes_library_scene = asset_server.load("recipes/library.scn.ron");
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

fn check_assets_loaded(
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut loading_manager: ResMut<LoadingManager>,
    mut status: ResMut<LoadingStatus>,
    asset_server: Res<AssetServer>,
    enemy_prefabs: Res<EnemyPrefabsFolderHandle>,
    unlocks: Res<UnlocksFolderHandle>,
    research: Res<ResearchFolderHandle>,
    folder: Res<Assets<LoadedFolder>>,
) {
    status.current_phase = "Loading Assets".into();
    status.detail = "Loading files from disk...".into();

    let spawn_tables_loaded = loading_manager
        .spawn_tables
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    if asset_server.is_loaded_with_dependencies(&loading_manager.startup_scene)
        && asset_server.is_loaded_with_dependencies(&loading_manager.recipes_library_scene)
        && spawn_tables_loaded
        && asset_server.is_loaded_with_dependencies(enemy_prefabs.0.id())
        && asset_server.is_loaded_with_dependencies(unlocks.0.id())
        && asset_server.is_loaded_with_dependencies(research.0.id())
    {
        info!("assets loaded");

        let Some(enemy_prefabs_folder) = folder.get(enemy_prefabs.0.id()) else {
            panic!("folder not loaded even tho asset server said it is")
        };

        for untyped_handle in enemy_prefabs_folder.handles.iter().cloned() {
            let Some(asset_path) = asset_server.get_path(untyped_handle.id()) else {
                continue;
            };

            let path = asset_path
                .path()
                .file_name()
                .and_then(OsStr::to_str)
                .expect("expected only files, but got also something else?");

            let key = path
                .split_once('.')
                .map(|(name, _suffix)| name)
                .map(ToString::to_string)
                .expect("invalid file name, missing suffix");

            if let Ok(handle) = untyped_handle.try_typed::<DynamicScene>() {
                debug!(%key, %path, "succesfully typed enemy prefab asset to DynamicScene");
                loading_manager.enemies.insert(key, handle);
            }
        }

        next_phase.set(LoadingPhase::SpawnEntities);
    }
}

// --- Phase: SpawnEntities ---

fn spawn_all_entities(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut assets: ResMut<Assets<ResearchDefinition>>,
    unlock_state: Res<UnlockState>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Spawning Entities".into();
    status.detail = "Creating research nodes...".into();

    debug!("Spawning research entities...");

    // Inline research spawning logic (from research::systems::spawn_research_entities)
    let ids: Vec<_> = assets.ids().collect();

    for id in ids {
        let def_id = {
            let Some(def) = assets.get(id) else {
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

        let handle = assets.get_strong_handle(id).unwrap();

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
                ))
                .id()
        };

        research_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned research entity: {} -> {:?}", def_id, entity);
    }

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
    compiled: Query<&CompiledUnlock>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Compiling Unlocks".into();
    status.detail = "Building logic graphs...".into();

    let encyclopedia = encyclopedia_query.single().ok();

    let ctx = CompilerContext {
        wallet: &wallet,
        encyclopedia,
        unlock_state: &unlock_state,
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

    next_phase.set(LoadingPhase::SpawnScene);
}

// --- Phase: SpawnScene ---

fn spawn_startup_scene(
    mut scene_spawner: ResMut<SceneSpawner>,
    loading_manager: Res<LoadingManager>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Spawning Scene".into();
    status.detail = "Loading world...".into();

    info!("spawning starting scene");
    scene_spawner.spawn_dynamic(loading_manager.startup_scene.clone());
    scene_spawner.spawn_dynamic(loading_manager.recipes_library_scene.clone());
}

fn check_scene_spawned(mut next_phase: ResMut<NextState<LoadingPhase>>, query: Query<(), With<Village>>) {
    if !query.is_empty() {
        info!("scene spawned and validated, entering Ready state");
        next_phase.set(LoadingPhase::Ready);
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
