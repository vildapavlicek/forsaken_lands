use bevy::prelude::*;
use states::GameState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(OnEnter(GameState::Loading), start_loading)
            .add_systems(Update, check_assets.run_if(in_state(GameState::Loading)));
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub portal_scene: Handle<DynamicScene>,
    pub village_scene: Handle<DynamicScene>,
}

fn start_loading(mut assets: ResMut<GameAssets>, asset_server: Res<AssetServer>) {
    assets.portal_scene = asset_server.load("prefabs/portals/portal.scn.ron");
    assets.village_scene = asset_server.load("prefabs/village/village.scn.ron");
}

fn check_assets(
    mut next_state: ResMut<NextState<GameState>>,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    let portal_loaded = asset_server.is_loaded_with_dependencies(&game_assets.portal_scene);
    let village_loaded = asset_server.is_loaded_with_dependencies(&game_assets.village_scene);

    if portal_loaded && village_loaded {
        next_state.set(GameState::Running);
    }
}
