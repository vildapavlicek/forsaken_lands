use {bevy::prelude::*, game_assets::GameAssets, states::GameState, village_components::Village};

pub fn spawn_starting_scene(mut scene_spawner: ResMut<SceneSpawner>, game_assets: Res<GameAssets>) {
    info!("spawning starting scene");
    scene_spawner.spawn_dynamic(game_assets.startup_scene.clone());
    scene_spawner.spawn_dynamic(game_assets.research_library_scene.clone());
    scene_spawner.spawn_dynamic(game_assets.recipes_library_scene.clone());
}

use hero_events::OpenHeroScreen;

pub fn check_scene_spawned(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(), With<Village>>,
) {
    if !query.is_empty() {
        info!("scene spawned and validated, entering Running state");
        next_state.set(GameState::Running);
    }
}

pub fn handle_village_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut open_hero_screen: MessageWriter<OpenHeroScreen>,
    village: Query<(Entity, &GlobalTransform), With<Village>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera.single() {
            if let Ok((_village_entity, village_transform)) = village.single() {
                if let Ok(window) = windows.single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        if let Ok(world_pos) =
                            camera.viewport_to_world_2d(camera_transform, cursor_pos)
                        {
                            let distance =
                                world_pos.distance(village_transform.translation().truncate());
                            if distance < 20.0 {
                                open_hero_screen.write(OpenHeroScreen);
                            }
                        }
                    }
                }
            }
        }
    }
}
