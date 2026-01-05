use {
    bevy::prelude::*,
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    enemy_components::{
        Dead, Enemy, Health, Lifetime, MonsterId, MovementSpeed, ResourceRewards, Reward,
    },
    game_assets::GameAssets,
    hero_events::EnemyKilled,
    portal_assets::{SpawnCondition, SpawnTable},
    portal_components::{Portal, SpawnTableId, SpawnTimer},
    rand::{distr::weighted::WeightedIndex, prelude::*},
    system_schedule::GameSchedule,
};

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>();
        app.register_type::<SpawnTimer>();
        app.register_type::<SpawnTableId>();

        app.register_type::<Enemy>();
        app.register_type::<MovementSpeed>();
        app.register_type::<Lifetime>();
        app.register_type::<Health>();
        app.register_type::<ResourceRewards>();
        app.register_type::<Reward>();
        app.register_type::<Dead>();
        app.register_type::<MonsterId>();

        app.add_systems(Update, enemy_spawn_system);
        app.add_systems(Update, move_enemy.in_set(GameSchedule::PerformAction));
        app.add_systems(
            Update,
            (despawn_expired_enemies, despawn_dead_enemies).in_set(GameSchedule::FrameEnd),
        );

        app.add_observer(handle_divinity_increase);
    }
}

fn enemy_spawn_system(
    time: Res<Time>,
    mut query: Query<(&mut SpawnTimer, &SpawnTableId, &Divinity), With<Portal>>,
    game_assets: Res<GameAssets>,
    spawn_tables: Res<Assets<SpawnTable>>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    for (mut timer, table_id, divinity) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            let table_handle = if let Some(handle) = game_assets.spawn_tables.get(&table_id.0) {
                handle
            } else {
                error!("Unknown spawn table: {}", table_id.0);
                continue;
            };

            // Get the asset data
            if let Some(table) = spawn_tables.get(table_handle) {
                // Find valid entries based on condition
                let valid_entries: Vec<_> = table
                    .entries
                    .iter()
                    .filter(|e| match &e.condition {
                        SpawnCondition::Min(req) => divinity >= req,
                        SpawnCondition::Specific(req) => divinity == req,
                        SpawnCondition::Range { min, max } => divinity >= min && divinity <= max,
                    })
                    .collect();

                if valid_entries.is_empty() {
                    continue;
                }

                // Collect weights and select randomly
                let weights: Vec<u32> = valid_entries.iter().map(|e| e.weight).collect();
                let Ok(dist) = WeightedIndex::new(&weights) else {
                    error!("Failed to create weighted distribution from spawn table weights");
                    continue;
                };

                let mut rng = rand::rng();
                let selected_entry = valid_entries[dist.sample(&mut rng)];

                info!("Spawning monster: {}", selected_entry.monster_file);

                let Some(prefab_handle) = game_assets
                    .enemies
                    .get(&selected_entry.monster_file)
                    .cloned()
                else {
                    error!(%selected_entry.monster_file, "failed to spawn monster, not found in enemies library");
                    return;
                };

                scene_spawner.spawn_dynamic(prefab_handle);
            }
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

fn despawn_dead_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Health), (With<Enemy>, Without<Dead>)>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.trigger(EnemyKilled { entity });
            commands
                .entity(entity)
                .insert(Dead)
                .remove::<(Sprite, Transform)>();
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

fn handle_divinity_increase(
    trigger: On<IncreaseDivinity>,
    mut query: Query<(&mut Divinity, &mut DivinityStats), With<Portal>>,
) {
    let event = trigger.event();
    if let Ok((mut divinity, mut stats)) = query.get_mut(event.entity) {
        if stats.add_xp(event.xp_amount, &mut divinity) {
            info!(
                tier = divinity.tier,
                level = divinity.level,
                "Portal leveled up"
            );
        }
    }
}
