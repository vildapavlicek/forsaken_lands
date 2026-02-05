use {
    bevy::prelude::*,
    enemy_components::{Enemy, Health, MonsterTags},
    hero_components::{
        AttackRange, AttackSpeed, Damage, Hero, MeleeArc, MeleeWeapon, Projectile,
        ProjectileDamage, ProjectileSpeed, ProjectileTarget, RangedWeapon, Weapon, WeaponTags,
    },
    hero_events::{AttackIntent, DamageRequest, MeleeHit, ProjectileHit},
    shared_components::HitIndicator,
    states::GameState,
    system_schedule::GameSchedule,
    village_components::Village,
};

pub struct HeroesPlugin;

impl Plugin for HeroesPlugin {
    fn build(&self, app: &mut App) {
        // Only register types that derive Reflect (state components)
        app.register_type::<Hero>().register_type::<Weapon>();

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
        app.add_observer(damage_pipeline_observer);
        app.add_observer(apply_hit_indicator_observer);
        app.add_systems(OnExit(GameState::Running), clean_up_heroes);
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
            error!(
                "hero_attack_intent_system: Multiple villages with Transform found! Count: {}",
                villages.iter().count()
            );
        }
        return;
    };

    for (weapon_entity, range, mut attack_speed, child_of) in weapons.iter_mut() {
        attack_speed.timer.tick(time.delta());

        // Only attack if current weapon is held by a hero
        if heroes.get(child_of.parent()).is_err() {
            continue;
        }

        if !attack_speed.timer.is_finished() {
            continue;
        }

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
            // Reset timer only after a successful attack
            attack_speed.timer.reset();
        }
    }
}

fn hero_projectile_spawn_system(
    trigger: On<AttackIntent>,
    mut commands: Commands,
    weapons: Query<(&Damage, &WeaponTags), (With<RangedWeapon>, Without<MeleeWeapon>)>,
    villages: Query<&Transform, With<Village>>,
    enemies: Query<&MonsterTags, With<Enemy>>,
) {
    let Ok(village_transform) = villages.single() else {
        if villages.is_empty() {
            error!("hero_projectile_spawn_system: No village with Transform found.");
        } else {
            error!(
                "hero_projectile_spawn_system: Multiple villages with Transform found! Count: {}",
                villages.iter().count()
            );
        }
        return;
    };

    let intent = trigger.event();

    // Double check the attacker is still a valid weapon.
    if let Ok((damage, tags)) = weapons.get(intent.attacker)
        // Check if target still exists (optional for spawn, but good practice)
        && enemies.get(intent.target).is_ok()
    {
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
            // Store raw damage context, calculate on hit
            ProjectileDamage {
                base_damage: damage.0,
                source_tags: tags.0.clone(),
            },
        ));
    }
}

fn hero_melee_attack_system(
    trigger: On<AttackIntent>,
    mut commands: Commands,
    weapons: Query<
        (&Damage, &AttackRange, &MeleeArc, &WeaponTags),
        (With<MeleeWeapon>, Without<RangedWeapon>),
    >,
    villages: Query<&Transform, With<Village>>,
    enemies: Query<(Entity, &Transform, &MonsterTags), With<Enemy>>,
    bonus_stats: Res<bonus_stats::BonusStats>,
) {
    let Ok(village_transform) = villages.single() else {
        if villages.is_empty() {
            error!("hero_melee_attack_system: No village with Transform found.");
        } else {
            error!(
                "hero_melee_attack_system: Multiple villages with Transform found! Count: {}",
                villages.iter().count()
            );
        }
        return;
    };

    let intent = trigger.event();

    if let Ok((damage, range, arc, tags)) = weapons.get(intent.attacker) {
        let Ok((_, target_transform, _)) = enemies.get(intent.target) else {
            return;
        };

        // Determine attack direction (Target - Village)
        let attack_direction = (target_transform.translation - village_transform.translation)
            .truncate()
            .normalize();

        let targets = enemies
            .iter()
            .filter_map(|(enemy_entity, enemy_transform, _)| {
                let to_enemy = enemy_transform.translation - village_transform.translation;
                let distance = to_enemy.length();

                if distance > range.0 {
                    return None;
                };

                // Check 2: Angle within MeleeArc
                // angle_to returns value in [-PI, PI], so we check its absolute value
                let angle = attack_direction.angle_to(to_enemy.truncate());
                if angle.abs() > arc.width / 2.0 {
                    return None;
                }

                Some(enemy_entity)
            })
            .collect::<Vec<Entity>>();

        if targets.is_empty() {
            return;
        }

        // Trigger DamageRequest for EACH target individually
        // This allows per-target damage calculation (e.g., individual defenses)
        for &target in &targets {
            commands.trigger(DamageRequest {
                source: intent.attacker,
                target,
                base_damage: damage.0,
                source_tags: tags.0.clone(),
            });
        }

        // Keep MeleeHit for visual indicators (HitIndicator), but damage is handled by DamageRequest.
        commands.trigger(MeleeHit {
            attacker: intent.attacker,
            targets,
            damage: 0.0, // Deprecated, damage applied via DamageRequest
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
    for (projectile_entity, projectile_transform, target, damage_data) in projectiles.iter() {
        if let Ok(target_transform) = enemies.get(target.0) {
            let distance = projectile_transform
                .translation
                .distance(target_transform.translation);

            if distance < 10.0 {
                // Trigger DamageRequest
                commands.trigger(DamageRequest {
                    source: projectile_entity,
                    target: target.0,
                    base_damage: damage_data.base_damage,
                    source_tags: damage_data.source_tags.clone(),
                });

                // Trigger ProjectileHit for other potential observers (visuals, etc)
                commands.trigger(ProjectileHit {
                    projectile: projectile_entity,
                    target: target.0,
                    damage: 0.0, // Deprecated
                });
                commands.entity(projectile_entity).despawn();
            }
        } else {
            // Target no longer exists
            commands.entity(projectile_entity).despawn();
        }
    }
}

fn damage_pipeline_observer(
    trigger: On<DamageRequest>,
    mut enemies: Query<(&mut Health, &MonsterTags), With<Enemy>>,
    bonus_stats: Res<bonus_stats::BonusStats>,
) {
    let req = trigger.event();

    if let Ok((mut health, monster_tags)) = enemies.get_mut(req.target) {
        let ctx =
            bonus_stats::DamageContext::new(req.base_damage, &req.source_tags, &monster_tags.0);

        let final_damage = bonus_stats::pipeline::calculate(&ctx, &bonus_stats);

        health.current -= final_damage;

        trace!(
            "Damage applied to {:?}: base={} -> final={} (health now {}/{})",
            req.target, req.base_damage, final_damage, health.current, health.max
        );
    }
}

fn apply_hit_indicator_observer(
    trigger: On<MeleeHit>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
) {
    let hit = trigger.event();
    for &target in &hit.targets {
        if let Ok(mut sprite) = sprites.get_mut(target) {
            let mut indicator = HitIndicator::new();
            // Swap immediately for instant feedback
            std::mem::swap(&mut sprite.color, &mut indicator.saved_color);
            // Since we already did 1 swap, we need 3 more (total 4) to end back on original color.
            indicator.blink_count = 3;
            commands.entity(target).insert(indicator);
        }
    }
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

pub fn clean_up_heroes(
    mut commands: Commands,
    heroes: Query<Entity, With<Hero>>,
    projectiles: Query<Entity, With<Projectile>>,
    loose_weapons: Query<Entity, (With<Weapon>, Without<ChildOf>)>,
) {
    debug!("Cleaning up heroes, projectiles, and loose weapons");
    for entity in heroes.iter() {
        commands.entity(entity).despawn();
    }
    for entity in projectiles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in loose_weapons.iter() {
        commands.entity(entity).despawn();
    }
}
