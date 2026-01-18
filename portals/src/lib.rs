use {
    bevy::prelude::*,
    divinity_components::{Divinity, DivinityStats, MaxUnlockedDivinity},
    divinity_events::IncreaseDivinity,
    enemy_components::{
        Dead, Drop, Drops, Enemy, EnemyRange, Health, Lifetime, MELEE_ENGAGEMENT_RADIUS, MonsterId,
        MovementSpeed, TargetDestination,
    },
    hero_events::EnemyKilled,
    loading::GameAssets,
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
        app.register_type::<Drops>();
        app.register_type::<Drop>();
        app.register_type::<Dead>();
        app.register_type::<MonsterId>();
        app.register_type::<EnemyRange>();
        app.register_type::<TargetDestination>();

        app.add_systems(Update, enemy_spawn_system);
        app.add_systems(Update, move_enemy.in_set(GameSchedule::PerformAction));
        app.add_systems(
            Update,
            manage_enemy_lifecycle.in_set(GameSchedule::FrameEnd),
        );
        app.add_systems(Update, draw_range_gizmos);

        app.add_observer(assign_enemy_destination);
        app.add_observer(assign_enemy_destination);
        app.add_observer(handle_max_divinity_increase);
        app.add_systems(OnExit(states::GameState::Running), clean_up_portals);
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

                debug!("Spawning monster: {}", selected_entry.monster_id);

                let Some(prefab_handle) =
                    game_assets.enemies.get(&selected_entry.monster_id).cloned()
                else {
                    error!(%selected_entry.monster_id, "failed to spawn monster, not found in enemies library");
                    return;
                };

                scene_spawner.spawn_dynamic(prefab_handle);
            }
        }
    }
}

fn manage_enemy_lifecycle(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Option<&mut Lifetime>,
            Option<&Health>,
            Option<&Dead>,
        ),
        With<Enemy>,
    >,
) {
    for (entity, lifetime_opt, health_opt, dead_opt) in query.iter_mut() {
        let mut should_despawn = false;

        // Handle Lifetime
        if let Some(mut lifetime) = lifetime_opt {
            lifetime.0.tick(time.delta());
            if lifetime.0.is_finished() {
                should_despawn = true;
            }
        }

        // Handle Death
        if let Some(health) = health_opt {
            if dead_opt.is_none() && health.current <= 0.0 {
                commands.trigger(EnemyKilled { entity });
                commands
                    .entity(entity)
                    .insert(Dead)
                    .remove::<(Sprite, Transform)>();
                // Prevent despawn this frame to avoid conflict
                should_despawn = false;
            }
        }

        if should_despawn {
            commands.entity(entity).despawn();
        }
    }
}

/// Assigns a random target destination when an Enemy is spawned.
/// Triggered when the Enemy component is inserted.
fn assign_enemy_destination(
    trigger: On<Add, EnemyRange>,
    mut commands: Commands,
    query: Query<&EnemyRange>,
) {
    let entity = trigger.entity;
    let range = match query.get(entity) {
        Ok(range) => range,
        Err(err) => {
            error!(%err, "could not apply TargetDestination");
            return;
        }
    };

    let mut rng = rand::rng();

    match range {
        EnemyRange::CloseRange => {
            // Funnel strategy: Target a point on a circle around the village
            // Village center is assumed at (0, -300)
            let village_center = Vec2::new(0.0, -300.0);

            // Random angle for funneling (restricted to -30 to 210 degrees)
            // This prevents enemies from going "below" the village (South)
            let min_angle = -30.0f32.to_radians();
            let max_angle = 210.0f32.to_radians();
            let theta = rng.random_range(min_angle..max_angle);

            // Random distance buffer to prevent stacking and overlap
            // MELEE_ENGAGEMENT_RADIUS is max, 25.0 is min offset from center
            let min_dist = 25.0;
            let max_dist = MELEE_ENGAGEMENT_RADIUS;
            let r = rng.random_range(min_dist..max_dist);

            let offset = Vec2::new(r * theta.cos(), r * theta.sin());
            let target = village_center + offset;

            commands.entity(entity).insert(TargetDestination(target));
        }
        _ => {
            let (min_y, max_y) = range.y_bounds();
            // Generate random x within game bounds (-200 to 200) and y within range section
            let x = rng.random_range(-200.0..200.0);
            let y = rng.random_range(min_y..max_y);
            commands
                .entity(entity)
                .insert(TargetDestination(Vec2::new(x, y)));
        }
    }
}

/// Moves enemies towards their target destination.
fn move_enemy(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementSpeed, Option<&TargetDestination>), With<Enemy>>,
) {
    for (mut transform, speed, target_dest) in query.iter_mut() {
        // Use target destination if available, otherwise fallback to default y=-250
        let target_y = target_dest.map(|t| t.0.y).unwrap_or(-250.0);
        let target_x = target_dest
            .map(|t| t.0.x)
            .unwrap_or(transform.translation.x);

        // Move towards target y
        if transform.translation.y > target_y {
            transform.translation.y -= speed.0 * time.delta_secs();
            if transform.translation.y < target_y {
                transform.translation.y = target_y;
            }
        }

        // Move towards target x
        let x_diff = target_x - transform.translation.x;
        if x_diff.abs() > 1.0 {
            let x_speed = speed.0 * 0.5; // Move slower horizontally
            let x_dir = x_diff.signum();
            transform.translation.x += x_dir * x_speed * time.delta_secs();
            // Clamp to target if we overshoot
            if (target_x - transform.translation.x).signum() != x_dir {
                transform.translation.x = target_x;
            }
        }
    }
}

/// Draws debug gizmo lines at each range section boundary.
fn draw_range_gizmos(mut gizmos: Gizmos) {
    let line_half_width = 250.0;
    let village_center = Vec2::new(0.0, -300.0);

    // LongRange / MediumRange boundary (y = 100) - Yellow
    gizmos.line_2d(
        Vec2::new(-line_half_width, EnemyRange::LongRange.y_bounds().0),
        Vec2::new(line_half_width, EnemyRange::LongRange.y_bounds().0),
        Color::srgb(1.0, 1.0, 0.0),
    );

    // MediumRange / CloseRange boundary (y = -100) - Orange
    gizmos.line_2d(
        Vec2::new(-line_half_width, EnemyRange::MediumRange.y_bounds().0),
        Vec2::new(line_half_width, EnemyRange::MediumRange.y_bounds().0),
        Color::srgb(1.0, 0.5, 0.0),
    );

    // CloseRange: Draw funnel target area circles - Red
    // Max engagement radius
    gizmos.circle_2d(
        village_center,
        MELEE_ENGAGEMENT_RADIUS,
        Color::srgb(1.0, 0.0, 0.0),
    );

    // Min offset radius
    gizmos.circle_2d(village_center, 25.0, Color::srgb(0.5, 0.0, 0.0));
}

fn handle_max_divinity_increase(
    trigger: On<IncreaseDivinity>,
    mut query: Query<(&mut DivinityStats, &mut MaxUnlockedDivinity), With<Portal>>,
) {
    let event = trigger.event();
    if let Ok((mut stats, mut max_divinity)) = query.get_mut(event.entity) {
        if stats.add_xp(event.xp_amount, &mut max_divinity) {
            info!(
                tier = max_divinity.tier,
                level = max_divinity.level,
                "Portal leveled up"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_unlocked_divinity_update() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.register_type::<Divinity>();
        app.register_type::<DivinityStats>();
        app.register_type::<MaxUnlockedDivinity>();
        app.add_observer(handle_max_divinity_increase);

        let entity = app
            .world_mut()
            .spawn((
                Portal,
                Divinity::new(1, 1),
                DivinityStats {
                    current_xp: 0.0,
                    required_xp: 100.0,
                },
                MaxUnlockedDivinity(Divinity::new(1, 1)),
            ))
            .id();

        // Case 1: Increase XP to level up
        app.world_mut().flush();
        app.world_mut().trigger(IncreaseDivinity {
            entity,
            xp_amount: 150.0, // Enough to level up (req 100)
        });
        app.update();

        let max_unlocked = app.world().get::<MaxUnlockedDivinity>(entity).unwrap();

        assert_eq!(max_unlocked.level, 2);

        // Case 2: Verify Tier up
        let entity_tier = app
            .world_mut()
            .spawn((
                Portal,
                Divinity::new(1, 99), // Almost tier up
                DivinityStats {
                    current_xp: 0.0,
                    required_xp: 10000.0, // Arbitrary high
                },
                MaxUnlockedDivinity(Divinity::new(1, 99)),
            ))
            .id();

        // Force level up via event
        app.world_mut().trigger(IncreaseDivinity {
            entity: entity_tier,
            xp_amount: 1000000.0,
        });
        app.update();

        let max_unlocked = app.world().get::<MaxUnlockedDivinity>(entity_tier).unwrap();

        assert_eq!(max_unlocked.tier, 2);
    }
}

pub fn clean_up_portals(
    mut commands: Commands,
    portals: Query<Entity, With<Portal>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    debug!("Cleaning up portals and enemies");
    for entity in portals.iter() {
        commands.entity(entity).despawn();
    }
    for entity in enemies.iter() {
        commands.entity(entity).despawn();
    }
}
