use {
    bevy::prelude::*,
    enemy_components::{Enemy, MovementSpeed, NeedsHydration, RewardCoefficient},
    game_assets::GameAssets,
    portal_components::{Portal, SpawnTimer},
};

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>();
        app.register_type::<SpawnTimer>();

        app.register_type::<Enemy>();
        app.register_type::<MovementSpeed>();
        app.register_type::<RewardCoefficient>();
        app.register_type::<NeedsHydration>();

        app.add_systems(Update, enemy_spawn_system);
    }
}

fn enemy_spawn_system(
    time: Res<Time>,
    mut query: Query<&mut SpawnTimer, With<Portal>>,
    game_assets: Res<GameAssets>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    for mut timer in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            info!("spawning monster");
            scene_spawner.spawn_dynamic(game_assets.goblin_prefab.clone());
        }
    }
}
