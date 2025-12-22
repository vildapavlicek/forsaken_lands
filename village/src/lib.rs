use bevy::prelude::*;
use game_assets::GameAssets;
use states::GameState;
use village_components::Village;

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>().add_systems(
            OnEnter(GameState::Running),
            spawn_village.run_if(not(any_with_component::<Village>)),
        );
    }
}

fn spawn_village(mut scene_spawner: ResMut<SceneSpawner>, game_assets: Res<GameAssets>) {
    info!("spawning village");
    scene_spawner.spawn_dynamic(game_assets.village_scene.clone());
}
