use {
    bevy::prelude::*,
    enemy_components::{Enemy, Health},
    hero_components::{
        AttackRange, AttackSpeed, Damage, Hero, Projectile, ProjectileDamage, ProjectileSpeed,
        ProjectileTarget, Weapon,
    },
    messages::{AttackIntent, ProjectileHit},
    states::GameState,
    system_schedule::GameSchedule,
    village_components::Village,
};

pub struct HeroesPlugin;

impl Plugin for HeroesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hero>()
            .register_type::<Weapon>()
            .register_type::<Damage>()
            .register_type::<AttackRange>()
            .register_type::<AttackSpeed>()
            .register_type::<Projectile>()
            .register_type::<ProjectileTarget>()
            .register_type::<ProjectileSpeed>()
            .register_type::<ProjectileDamage>();

        app.add_systems(
            Update,
            (
                hero_attack_intent_system.in_set(GameSchedule::ResolveIntent),
                (
                    projectile_movement_system,
                    projectile_collision_system,
                )
                    .in_set(GameSchedule::PerformAction)
                    .chain(),
            )
                .run_if(in_state(GameState::Running)),
        );

        app.add_observer(hero_projectile_spawn_system);
        app.add_observer(apply_damage_system);
    }
}

fn hero_attack_intent_system(
    time: Res<Time>,
    mut commands: Commands,
    mut weapons: Query<(Entity, &AttackRange, &mut AttackSpeed, &ChildOf), With<Weapon>>,
    heroes: Query<&Hero>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Village>)>,
    villages: Query<&Transform, With<Village>>,
) {
    let Ok(village_transform) = villages.single() else {
        error!("village without transform");
        return;
    };

    for (weapon_entity, range, mut attack_speed, child_of) in weapons.iter_mut() {
        // Only attack if current weapon is held by a hero
        if heroes.get(child_of.parent()).is_err() {
            continue;
        }

        if attack_speed.timer.tick(time.delta()).just_finished() {
            let mut closest_enemy: Option<(Entity, f32)> = None;

            for (enemy_entity, enemy_transform) in enemies.iter() {
                let distance = village_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance <= range.0 {
                    if let Some((_, closest_distance)) = closest_enemy {
                        if distance < closest_distance {
                            closest_enemy = Some((enemy_entity, distance));
                        }
                    } else {
                        closest_enemy = Some((enemy_entity, distance));
                    }
                }
            }

            if let Some((enemy_entity, _)) = closest_enemy {
                commands.trigger(AttackIntent {
                    attacker: weapon_entity,
                    target: enemy_entity,
                });
            }
        }
    }
}

fn hero_projectile_spawn_system(
    trigger: On<AttackIntent>,
    mut commands: Commands,
    weapons: Query<&Damage, With<Weapon>>,
    villages: Query<&Transform, With<Village>>,
) {
    let Ok(village_transform) = villages.single() else {
        error!("village without transform");
        return;
    };

    let intent = trigger.event();

    // Double check the attacker is still a valid weapon.
    // The intent system already filters this, but if a weapon was unequipped
    // between intent and action, this would catch it.
    if let Ok(damage) = weapons.get(intent.attacker) {
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_translation(village_transform.translation),
            Projectile,
            ProjectileTarget(intent.target),
            ProjectileSpeed(400.0),
            ProjectileDamage(damage.0),
        ));
    }
}

fn projectile_movement_system(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &ProjectileTarget, &ProjectileSpeed), With<Projectile>>,
    enemies: Query<&Transform, (With<Enemy>, Without<Projectile>)>,
) {
    for (mut transform, target, speed) in projectiles.iter_mut() {
        if let Ok(target_transform) = enemies.get(target.0) {
            let direction = (target_transform.translation - transform.translation).normalize();
            transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}

fn projectile_collision_system(
    mut commands: Commands,
    projectiles: Query<
        (Entity, &Transform, &ProjectileTarget, &ProjectileDamage),
        With<Projectile>,
    >,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for (projectile_entity, projectile_transform, target, damage) in projectiles.iter() {
        if let Ok(target_transform) = enemies.get(target.0) {
            let distance = projectile_transform
                .translation
                .distance(target_transform.translation);

            if distance < 10.0 {
                commands.trigger(ProjectileHit {
                    projectile: projectile_entity,
                    target: target.0,
                    damage: damage.0,
                });
                commands.entity(projectile_entity).despawn();
            }
        } else {
            // Target no longer exists
            commands.entity(projectile_entity).despawn();
        }
    }
}

fn apply_damage_system(
    trigger: On<ProjectileHit>,
    mut enemies: Query<&mut Health, With<Enemy>>,
) {
    let hit = trigger.event();
    if let Ok(mut health) = enemies.get_mut(hit.target) {
        health.current -= hit.damage;
        info!(
            "Projectile hit enemy {:?} for {} damage. Health: {}/{}",
            hit.target, hit.damage, health.current, health.max
        );
    }
}
