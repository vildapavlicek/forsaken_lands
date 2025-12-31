use {bevy::prelude::*, game_assets::GameAssets};

pub fn spawn_starting_scene(mut scene_spawner: ResMut<SceneSpawner>, game_assets: Res<GameAssets>) {
    info!("spawning starting scene");
    scene_spawner.spawn_dynamic(game_assets.startup_scene.clone());
}
