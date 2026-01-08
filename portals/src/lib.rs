use {
    bevy::prelude::*,
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    enemy_components::{
        Dead, Enemy, EnemyRange, Health, Lifetime, MELEE_ENGAGEMENT_RADIUS, MonsterId,
        MovementSpeed, ResourceRewards, Reward, TargetDestination,
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
        app.register_type::<EnemyRange>();
        app.register_type::<TargetDestination>();

        app.add_systems(Update, enemy_spawn_system);
        app.add_systems(Update, move_enemy.in_set(GameSchedule::PerformAction));
        app.add_systems(
            Update,
            (despawn_expired_enemies, despawn_dead_enemies).in_set(GameSchedule::FrameEnd),
        );
        app.add_systems(Update, draw_range_gizmos);

        app.add_observer(assign_enemy_destination);
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

                debug!("Spawning monster: {}", selected_entry.monster_file);

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
