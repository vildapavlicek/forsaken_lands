use {bevy::prelude::*, game_assets::GameAssets, states::GameState, village_components::Village};

pub fn spawn_starting_scene(mut scene_spawner: ResMut<SceneSpawner>, game_assets: Res<GameAssets>) {
    info!("spawning starting scene");
    scene_spawner.spawn_dynamic(game_assets.startup_scene.clone());
    // scene_spawner.spawn_dynamic(game_assets.research_library_scene.clone());
    scene_spawner.spawn_dynamic(game_assets.recipes_library_scene.clone());
}

pub fn check_scene_spawned(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(), With<Village>>,
) {
    if !query.is_empty() {
        info!("scene spawned and validated, entering Running state");
        next_state.set(GameState::Running);
    }
}
