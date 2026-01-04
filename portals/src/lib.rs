use {
    bevy::prelude::*,
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    enemy_components::{
        Dead, Enemy, Health, Lifetime, MonsterId, MovementSpeed, NeedsHydration, ResourceRewards,
        Reward, RewardCoefficient,
    },
    game_assets::GameAssets,
    hero_events::EnemyKilled,
    portal_components::{Portal, SpawnTimer, SpawnTableId},
    portal_assets::{SpawnTable, SpawnCondition},
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
        app.register_type::<RewardCoefficient>();
        app.register_type::<NeedsHydration>();
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
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    for (mut timer, table_id, divinity) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            
            // Resolve ID to Handle (Simple mapping for now)
            let table_handle = if table_id.0 == "default" {
                &game_assets.spawn_table
            } else {
                warn!("Unknown spawn table: {}", table_id.0);
                continue;
            };

            // Get the asset data
            if let Some(table) = spawn_tables.get(table_handle) {
                // Find valid entries based on condition
                let valid_entries: Vec<_> = table.entries.iter().filter(|e| {
                    match &e.condition {
                        SpawnCondition::Min(req) => divinity >= req,
                        SpawnCondition::Specific(req) => divinity == req,
                        SpawnCondition::Range { min, max } => divinity >= min && divinity <= max,
                    }
                }).collect();

                if valid_entries.is_empty() {
                    continue;
                }

                // Simple selection: Pick first or random (Just taking first valid for simplicity here)
                // In a real implementation, you would use weights.
                if let Some(entry) = valid_entries.first() {
                    info!("Spawning monster: {}", entry.monster_file);
                    
                    // Construct path or use known handles. 
                    // Since "goblin" is known, we can match or load dynamically.
                    let prefab_handle = if entry.monster_file == "goblin" {
                        game_assets.goblin_prefab.clone()
                    } else {
                        // Fallback for new monsters defined in table
                        asset_server.load(format!("prefabs/enemies/{}.scn.ron", entry.monster_file))
                    };
                    
                    scene_spawner.spawn_dynamic(prefab_handle);
                }
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

