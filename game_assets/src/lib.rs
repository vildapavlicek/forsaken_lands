use bevy::prelude::*;
use states::GameState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(Startup, start_loading)
            .add_systems(Update, check_assets.run_if(in_state(GameState::Loading)))
            .add_systems(OnEnter(GameState::Loading), setup_loading_ui)
            .add_systems(OnExit(GameState::Loading), cleanup_loading_ui);
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub startup_scene: Handle<DynamicScene>,
    pub goblin_prefab: Handle<DynamicScene>,
}

fn start_loading(mut assets: ResMut<GameAssets>, asset_server: Res<AssetServer>) {
    info!("started loading assets");
    assets.startup_scene = asset_server.load("startup.scn.ron");
    assets.goblin_prefab = asset_server.load("prefabs/enemies/goblin.scn.ron");
}

fn check_assets(
    mut next_state: ResMut<NextState<GameState>>,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    if asset_server.is_loaded_with_dependencies(&game_assets.startup_scene)
        && asset_server.is_loaded_with_dependencies(&game_assets.goblin_prefab)
    {
        info!("assets loaded");
        next_state.set(GameState::Running);
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
