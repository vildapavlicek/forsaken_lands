use {bevy::prelude::*, game_assets::GameAssets, portal_components::Portal, states::GameState};

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>().add_systems(
            OnEnter(GameState::Running),
            spawn_portals.run_if(not(any_with_component::<Portal>)),
        );
    }
}

fn spawn_portals(mut scene_spawner: ResMut<SceneSpawner>, game_assets: Res<GameAssets>) {
    info!("spawning portal");
    scene_spawner.spawn_dynamic(game_assets.portal_scene.clone());
}
