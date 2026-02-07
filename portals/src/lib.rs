use {
    bevy::prelude::*,
    blessings::{BlessingDefinition, BlessingState, Blessings},
    buildings_components::TheMaw,
    divinity_components::CurrentDivinity,
    enemy_components::{
        Dead, Drop, Drops, Enemy, EnemyRange, Health, Lifetime, MELEE_ENGAGEMENT_RADIUS, MonsterId,
        MovementSpeed, TargetDestination,
    },
    enemy_events::EnemyEscaped,
    hero_events::EnemyKilled,
    loading::GameAssets,
    portal_assets::{SpawnCondition, SpawnTable, SpawnType},
    portal_components::{Portal, SpawnTableId, SpawnTimer},
    rand::{distr::weighted::WeightedIndex, prelude::*},
    system_schedule::GameSchedule,
};

pub mod enemy_details;

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>();
        app.register_type::<SpawnTimer>();
        app.register_type::<SpawnTableId>();
        app.init_resource::<enemy_resources::EnemyDetailsCache>();

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
        app.add_systems(Update, update_floating_text);

        app.add_observer(assign_enemy_destination);
        app.add_observer(apply_blessing_to_lifetime);
        app.add_observer(on_enemy_escaped);
        app.add_observer(enemy_details::cache_details_on_unlock);
        app.add_systems(OnExit(states::GameState::Running), clean_up_portals);
    }
}

fn enemy_spawn_system(
    time: Res<Time>,
    mut query: Query<(&mut SpawnTimer, &SpawnTableId, &CurrentDivinity), With<Portal>>,
    maw_query: Query<&Blessings, With<TheMaw>>,
    game_assets: Res<GameAssets>,
    spawn_tables: Res<Assets<SpawnTable>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    blessing_state: Res<BlessingState>,
    blessing_definitions: Res<Assets<BlessingDefinition>>,
) {
    // Calculate spawn rate modifier
    let mut speed_modifier = 1.0;
    if let Ok(blessings) = maw_query.single() {
        for (id, level) in &blessings.unlocked {
            if let Some(handle) = blessing_state.blessings.get(id)
                && let Some(def) = blessing_definitions.get(handle)
                && def.reward_id == "blessing:spawn_timer:decrease"
            {
                // Example: 10% faster per level (compounding? or linear?)
                // Let's assume linear for now or use the base_stats logic if defined,
                // but for prototype, hardcode effect logic:
                // modifier = 1.0 + (0.1 * level)
                speed_modifier += 0.1 * (*level as f32);
            }
        }
    }

    for (mut timer, table_id, divinity) in query.iter_mut() {
        let divinity = **divinity;
        if timer
            .0
            .tick(time.delta().mul_f32(speed_modifier))
            .just_finished()
        {
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
                        SpawnCondition::Min(req) => divinity >= *req,
                        SpawnCondition::Specific(req) => divinity == *req,
                        SpawnCondition::Range { min, max } => divinity >= *min && divinity <= *max,
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

                // Spawn based on spawn type
                match &selected_entry.spawn_type {
                    SpawnType::Single(monster_id) => {
                        debug!("Spawning single monster: {}", monster_id);
                        if let Some(prefab_handle) = game_assets.enemies.get(monster_id).cloned() {
                            scene_spawner.spawn_dynamic(prefab_handle);
                        } else {
                            error!(%monster_id, "failed to spawn monster, not found in enemies library");
                        }
                    }
                    SpawnType::Group(monster_ids) => {
                        debug!("Spawning monster group: {:?}", monster_ids);
                        for monster_id in monster_ids {
                            if let Some(prefab_handle) =
                                game_assets.enemies.get(monster_id).cloned()
                            {
                                scene_spawner.spawn_dynamic(prefab_handle);
                            } else {
                                error!(%monster_id, "failed to spawn monster in group, not found in enemies library");
                            }
                        }
                    }
                }
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
                commands.trigger(EnemyEscaped { entity });
                should_despawn = true;
            }
        }

        // Handle Death
        if let Some(health) = health_opt
            && dead_opt.is_none()
            && health.current <= 0.0
        {
            commands.trigger(EnemyKilled { entity });
            commands
                .entity(entity)
                .insert(Dead)
                .remove::<(Sprite, Transform)>();
            // Prevent despawn this frame to avoid conflict
            should_despawn = true;
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

#[cfg(test)]
mod tests {
    use {super::*, divinity_components::Divinity};

    #[test]
    fn test_current_divinity_spawn_filter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.register_type::<Portal>();
        app.register_type::<CurrentDivinity>();

        let entity = app
            .world_mut()
            .spawn((Portal, CurrentDivinity(Divinity::new(1, 10))))
            .id();

        let divinity = app.world().get::<CurrentDivinity>(entity).unwrap();
        assert_eq!(divinity.level, 10);
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

/// Applies 'IncreaseMonsterLifetime' blessing effects when Lifetime is added to an enemy.
fn apply_blessing_to_lifetime(
    trigger: On<Add, Lifetime>,
    mut query: Query<&mut Lifetime>,
    maw_query: Query<&Blessings, With<TheMaw>>,
    blessing_state: Res<BlessingState>,
    blessing_definitions: Res<Assets<BlessingDefinition>>,
) {
    if let Ok(mut lifetime) = query.get_mut(trigger.entity)
        && let Ok(blessings) = maw_query.single()
    {
        let mut extra_seconds = 0.0;
        for (id, level) in &blessings.unlocked {
            if let Some(handle) = blessing_state.blessings.get(id)
                && let Some(def) = blessing_definitions.get(handle)
                && def.reward_id == "blessing:monster:lifetime_increase"
            {
                // Example: +1s per level
                extra_seconds += 1.0 * (*level as f32);
            }
        }

        if extra_seconds > 0.0 {
            let current_duration = lifetime.0.duration();
            let new_duration = current_duration + std::time::Duration::from_secs_f32(extra_seconds);
            lifetime.0.set_duration(new_duration);
            debug!(
                "Applied blessing: +{}s to lifetime. New duration: {:?}",
                extra_seconds, new_duration
            );
        }
    }
}

#[derive(Component)]
pub struct FloatingText {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

fn on_enemy_escaped(trigger: On<EnemyEscaped>, mut commands: Commands, query: Query<&Transform>) {
    let event = trigger.event();
    let entity = event.entity;

    // Attempt to get the position of the escaping enemy
    // Note: Since we trigger before despawning, the entity should still exist with its components.
    let position = if let Ok(transform) = query.get(entity) {
        transform.translation
    } else {
        warn!(
            "EnemyEscaped event triggered but Transform not found for entity {:?}",
            entity
        );
        Vec3::ZERO
    };

    commands.spawn((
        Text2d::new("Escaped!"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_translation(position + Vec3::new(0.0, 20.0, 10.0)), // Offset slightly up and ensure z-index
        FloatingText {
            velocity: Vec2::new(0.0, 50.0), // Float up
            lifetime: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

fn update_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut FloatingText, &mut TextColor)>,
) {
    for (entity, mut transform, mut floating, mut color) in query.iter_mut() {
        floating.lifetime.tick(time.delta());

        // Move
        transform.translation.x += floating.velocity.x * time.delta_secs();
        transform.translation.y += floating.velocity.y * time.delta_secs();

        // Fade out
        let alpha = floating.lifetime.fraction_remaining();
        color.0.set_alpha(alpha);

        if floating.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
