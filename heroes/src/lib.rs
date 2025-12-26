use {
    bevy::{ecs::relationship::Relationship, prelude::*},
    enemy_components::{Enemy, Health},
    hero_components::{
        AttackRange, AttackSpeed, Damage, Hero, Projectile, ProjectileDamage, ProjectileSpeed,
        ProjectileTarget,
    },
    messages::{AttackIntent, ProjectileHit},
    system_schedule::GameSchedule,
};

pub struct HeroesPlugin;

impl Plugin for HeroesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hero>()
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
                    hero_projectile_spawn_system,
                    projectile_movement_system,
                    projectile_collision_system,
                )
                    .in_set(GameSchedule::PerformAction)
                    .chain(),
                apply_damage_system.in_set(GameSchedule::Effect),
            ),
        );
    }
}

fn hero_attack_intent_system(
    time: Res<Time>,
    mut attack_intent_writer: MessageWriter<AttackIntent>,
    mut heroes: Query<(Entity, &ChildOf, &AttackRange, &mut AttackSpeed), With<Hero>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    transforms: Query<&Transform>,
) {
    for (hero_entity, parent, range, mut attack_speed) in heroes.iter_mut() {
        if let Ok(hero_transform) = transforms.get(parent.get()) {
            if attack_speed.timer.tick(time.delta()).just_finished() {
                let mut closest_enemy: Option<(Entity, f32)> = None;

                for (enemy_entity, enemy_transform) in enemies.iter() {
                    let distance = hero_transform
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
                    attack_intent_writer.write(AttackIntent {
                        attacker: hero_entity,
                        target: enemy_entity,
                    });
                }
            }
        }
    }
}

fn hero_projectile_spawn_system(
    mut commands: Commands,
    mut attack_intent_reader: MessageReader<AttackIntent>,
    heroes: Query<(&ChildOf, &Damage), With<Hero>>,
    transforms: Query<&Transform>,
) {
    for intent in attack_intent_reader.read() {
        if let Ok((parent, damage)) = heroes.get(intent.attacker) {
            if let Ok(hero_transform) = transforms.get(parent.get()) {
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 1.0, 0.0),
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    Transform::from_translation(hero_transform.translation),
                    Projectile,
                    ProjectileTarget(intent.target),
                    ProjectileSpeed(400.0),
                    ProjectileDamage(damage.0),
                ));
            }
        }
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
    mut projectile_hit_writer: MessageWriter<ProjectileHit>,
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
                projectile_hit_writer.write(ProjectileHit {
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
    mut projectile_hit_reader: MessageReader<ProjectileHit>,
    mut enemies: Query<&mut Health, With<Enemy>>,
) {
    for hit in projectile_hit_reader.read() {
        if let Ok(mut health) = enemies.get_mut(hit.target) {
            health.current -= hit.damage;
            info!(
                "Projectile hit enemy {:?} for {} damage. Health: {}/{}",
                hit.target, hit.damage, health.current, health.max
            );
        }
    }
}
