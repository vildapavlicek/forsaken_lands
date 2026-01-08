mod resources;

use {
    crate::resources::{EnemyPrefabsFolderHandle, UnlocksFolderHandle},
    bevy::{asset::LoadedFolder, platform::collections::HashMap, prelude::*},
    portal_assets::SpawnTable,
    states::GameState,
    std::ffi::OsStr,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(Startup, (start_loading, load_enemy_prefabs, load_unlocks_assets))
            .add_systems(Update, check_assets.run_if(in_state(GameState::Loading)))
            .add_systems(OnEnter(GameState::Loading), setup_loading_ui)
            .add_systems(OnExit(GameState::Loading), cleanup_loading_ui);
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub startup_scene: Handle<DynamicScene>,
    pub research_library_scene: Handle<DynamicScene>,
    pub recipes_library_scene: Handle<DynamicScene>,
    pub spawn_tables: HashMap<String, Handle<SpawnTable>>,
    pub enemies: HashMap<String, Handle<DynamicScene>>,
}

fn start_loading(mut assets: ResMut<GameAssets>, asset_server: Res<AssetServer>) {
    info!("started loading assets");
    assets.startup_scene = asset_server.load("startup.scn.ron");
    assets.research_library_scene = asset_server.load("research.scn.ron");
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

fn check_assets(
    mut next_state: ResMut<NextState<GameState>>,
    mut game_assets: ResMut<GameAssets>,
    asset_server: Res<AssetServer>,
    enemy_prefabs: Res<EnemyPrefabsFolderHandle>,
    unlocks: Res<UnlocksFolderHandle>,
    folder: Res<Assets<LoadedFolder>>,
) {
    let spawn_tables_loaded = game_assets
        .spawn_tables
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    if asset_server.is_loaded_with_dependencies(&game_assets.startup_scene)
        && asset_server.is_loaded_with_dependencies(&game_assets.research_library_scene)
        && asset_server.is_loaded_with_dependencies(&game_assets.recipes_library_scene)
        && spawn_tables_loaded
        && asset_server.is_loaded_with_dependencies(enemy_prefabs.0.id())
        && asset_server.is_loaded_with_dependencies(unlocks.0.id())
    {
        info!("assets loaded");

        let Some(enemy_prefabs_folder) = folder.get(enemy_prefabs.0.id()) else {
            panic!("folder not loaded even tho asset server said it is")
        };

        for untyped_handle in enemy_prefabs_folder.handles.iter().cloned() {
            // 1) Resolve a stable key (path) for lookup
            let Some(asset_path) = asset_server.get_path(untyped_handle.id()) else {
                continue;
            };

            let path = asset_path
                .path()
                // goblin.scn.ron
                .file_name()
                .and_then(OsStr::to_str)
                .expect("expected only files, but got also something else?");

            let key = path
                // ("goblin", "scn.ron") -> "goblin"
                .split_once('.')
                .map(|(name, _suffix)| name)
                .map(ToString::to_string)
                .expect("invalid file name, missing suffix");

            // 2) Convert untyped -> typed (skip non-matching types safely)
            if let Ok(handle) = untyped_handle.try_typed::<DynamicScene>() {
                debug!(%key, %path, "succesfully typed enemy prefab asset to DynamicScene");
                game_assets.enemies.insert(key, handle);
            }
        }

        next_state.set(GameState::Initializing);
    }
}

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
            ..default()
        },
        LoadingUi,
    ));
}

fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingUi>>) {
    info!("cleaning up loading ui");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
