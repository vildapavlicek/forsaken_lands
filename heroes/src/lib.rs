use {
    bevy::prelude::*,
    enemy_components::{Enemy, Health},
    hero_components::{
        AttackRange, AttackSpeed, Damage, Hero, MeleeArc, MeleeWeapon, Projectile,
        ProjectileDamage, ProjectileSpeed, ProjectileTarget, RangedWeapon, Weapon,
    },
    hero_events::{AttackIntent, MeleeHit, ProjectileHit},
    shared_components::HitIndicator,
    states::GameState,
    system_schedule::GameSchedule,
    village_components::Village,
};

pub struct HeroesPlugin;

impl Plugin for HeroesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hero>()
            .register_type::<Weapon>()
            .register_type::<RangedWeapon>()
            .register_type::<MeleeWeapon>()
            .register_type::<MeleeArc>()
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
                (projectile_movement_system, projectile_collision_system)
                    .in_set(GameSchedule::PerformAction)
                    .chain(),
                hit_indicator_system.run_if(in_state(GameState::Running)),
            )
                .run_if(in_state(GameState::Running)),
        );

        app.add_observer(hero_projectile_spawn_system);
        app.add_observer(hero_melee_attack_system);
        app.add_observer(apply_damage_system);
        app.add_observer(apply_melee_damage_observer);
        app.add_observer(apply_hit_indicator_observer);
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
        if villages.is_empty() {
             error!("hero_attack_intent_system: No village with Transform found.");
        } else {
             error!("hero_attack_intent_system: Multiple villages with Transform found! Count: {}", villages.iter().count());
        }
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
    weapons: Query<&Damage, (With<RangedWeapon>, Without<MeleeWeapon>)>,
    villages: Query<&Transform, With<Village>>,
) {
    let Ok(village_transform) = villages.single() else {
        if villages.is_empty() {
             error!("hero_projectile_spawn_system: No village with Transform found.");
        } else {
             error!("hero_projectile_spawn_system: Multiple villages with Transform found! Count: {}", villages.iter().count());
        }
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

fn hero_melee_attack_system(
    trigger: On<AttackIntent>,
    mut commands: Commands,
    weapons: Query<(&Damage, &AttackRange, &MeleeArc), (With<MeleeWeapon>, Without<RangedWeapon>)>,
    villages: Query<&Transform, With<Village>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    let Ok(village_transform) = villages.single() else {
        if villages.is_empty() {
             error!("hero_melee_attack_system: No village with Transform found.");
        } else {
             error!("hero_melee_attack_system: Multiple villages with Transform found! Count: {}", villages.iter().count());
        }
        return;
    };

    let intent = trigger.event();

    if let Ok((damage, range, arc)) = weapons.get(intent.attacker) {
        let Ok((_, target_transform)) = enemies.get(intent.target) else {
            return;
        };

        // Determine attack direction (Target - Village)
        let attack_direction = (target_transform.translation - village_transform.translation)
            .truncate()
            .normalize();

        let targets = enemies
            .iter()
            .filter_map(|(enemy_entity, enemy_transform)| {
                let to_enemy = enemy_transform.translation - village_transform.translation;
                let distance = to_enemy.length();

                if distance > range.0 {
                    return None;
                };

                // Check 2: Angle within MeleeArc
                // angle_between returns value in [0, PI], so we just check if it's <= half the width
                let angle = attack_direction.angle_to(to_enemy.truncate());
                if angle > arc.width / 2.0 {
                    return None;
                }

                Some(enemy_entity)
            })
            .collect::<Vec<Entity>>();

        if targets.is_empty() {
            return;
        }

        commands.trigger(MeleeHit {
            attacker: intent.attacker,
            targets,
            damage: damage.0,
        });
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

fn apply_damage_system(trigger: On<ProjectileHit>, mut enemies: Query<&mut Health, With<Enemy>>) {
    let hit = trigger.event();
    if let Ok(mut health) = enemies.get_mut(hit.target) {
        health.current -= hit.damage;
        trace!(
            "Projectile hit enemy {:?} for {} damage. Health: {}/{}",
            hit.target, hit.damage, health.current, health.max
        );
    }
}

fn apply_melee_damage_observer(
    trigger: On<MeleeHit>,
    mut enemies: Query<&mut Health, With<Enemy>>,
) {
    let hit = trigger.event();
    for &target in &hit.targets {
        if let Ok(mut health) = enemies.get_mut(target) {
            health.current -= hit.damage;
            trace!(
                "Melee hit enemy {:?} for {} damage. Health: {}/{}",
                target, hit.damage, health.current, health.max
            );
        }
    }
}

fn apply_hit_indicator_observer(trigger: On<MeleeHit>, mut commands: Commands) {
    let hit = trigger.event();
    commands.insert_batch(
        hit.targets
            .iter()
            .filter_map(|target| {
                let target = *target;

                Some((target, HitIndicator::new()))
            })
            .collect::<Vec<_>>(),
    );
}

fn hit_indicator_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut HitIndicator, &mut Sprite)>,
) {
    for (entity, mut indicator, mut sprite) in query.iter_mut() {
        if indicator.timer.tick(time.delta()).just_finished() {
            std::mem::swap(&mut sprite.color, &mut indicator.saved_color);
            indicator.blink_count -= 1;

            if indicator.blink_count == 0 {
                commands.entity(entity).remove::<HitIndicator>();
            }
        }
    }
}
