use {bevy::prelude::*, portal_components::{Portal, SpawnTimer}};

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>();
        app.register_type::<SpawnTimer>();

        app.add_systems(Update, enemy_spawn_system);
    }
}

fn enemy_spawn_system(time: Res<Time>, mut query: Query<&mut SpawnTimer, With<Portal>>) {
    for mut timer in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            info!("spawning monster");
        }
    }
}
