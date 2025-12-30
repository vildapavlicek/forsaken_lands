use {
    bevy::prelude::*,
    enemy_components::{
        Enemy, Health, Lifetime, MovementSpeed, NeedsHydration, ResourceRewards, Reward,
        RewardCoefficient,
    },
    game_assets::GameAssets,
    portal_components::{Portal, SpawnTimer},
    system_schedule::GameSchedule,
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
        app.register_type::<Lifetime>();
        app.register_type::<Health>();
        app.register_type::<ResourceRewards>();
        app.register_type::<Reward>();

        app.add_systems(Update, enemy_spawn_system);
        app.add_systems(Update, move_enemy.in_set(GameSchedule::PerformAction));
        app.add_systems(
            Update,
            (despawn_expired_enemies, despawn_dead_enemies).in_set(GameSchedule::FrameEnd),
        );
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

fn despawn_expired_enemies(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime), With<Enemy>>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        if lifetime.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_dead_enemies(mut commands: Commands, query: Query<(Entity, &Health), With<Enemy>>) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).remove::<(Sprite, Transform)>();
        }
    }
}

fn move_enemy(time: Res<Time>, mut query: Query<(&mut Transform, &MovementSpeed), With<Enemy>>) {
    for (mut transform, speed) in query.iter_mut() {
        if transform.translation.y > -250.0 {
            transform.translation.y -= speed.0 * time.delta_secs();
            if transform.translation.y < -250.0 {
                transform.translation.y = -250.0;
            }
        }
    }
}
