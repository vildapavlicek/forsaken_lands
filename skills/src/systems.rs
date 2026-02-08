use {
    crate::*,
    bevy::prelude::*,
    bonus_stats::{BonusStats, StatBonus},
    enemy_components::Enemy,
    hero_events::DamageRequest,
    std::time::Duration,
};

/// Ticks all skill cooldown timer and initializes missing ones
pub fn tick_cooldowns(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, Option<&mut SkillCooldowns>, Option<&EquippedSkills>)>,
    skill_map: Res<SkillMap>,
    skills: Res<Assets<SkillDefinition>>,
) {
    for (entity, mut cooldowns, equipped) in &mut query {
        if let Some(ref mut cooldowns) = cooldowns {
            for timer in cooldowns.timers.values_mut() {
                timer.tick(time.delta());
            }
        }

        // Initialize timers for skills that are equipped but not yet tracked
        if let Some(equipped) = equipped {
            for skill_id in &equipped.0 {
                let already_tracked = cooldowns
                    .as_ref()
                    .map_or(false, |c| c.timers.contains_key(skill_id));

                if !already_tracked {
                    if let Some(skill_def) =
                        skill_map.handles.get(skill_id).and_then(|h| skills.get(h))
                    {
                        let duration = Duration::from_millis(skill_def.cooldown_ms as u64);
                        let timer = Timer::new(duration, TimerMode::Once);

                        if let Some(ref mut c) = cooldowns {
                            c.timers.insert(skill_id.clone(), timer);
                        } else {
                            let mut timers = std::collections::HashMap::new();
                            timers.insert(skill_id.clone(), timer);
                            commands.entity(entity).insert(SkillCooldowns { timers });
                        }
                    }
                }
            }
        }
    }
}

/// Ticks active skill buffs and removes them when expired
pub fn tick_buffs(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SkillBuff)>,
    mut bonus_stats: ResMut<BonusStats>,
) {
    for (entity, mut buff) in &mut query {
        buff.timer.tick(time.delta());
        if buff.timer.is_finished() {
            bonus_stats.remove(
                &buff.stat_key,
                StatBonus {
                    value: buff.value,
                    mode: match buff.mode {
                        1 => bonus_stats::StatMode::Percent,
                        2 => bonus_stats::StatMode::Multiplicative,
                        _ => bonus_stats::StatMode::Additive,
                    },
                },
            );

            commands.entity(entity).despawn();
        }
    }
}

/// Processes SkillActivated events, applies effects
pub fn process_skill_activation(
    trigger: On<SkillActivated>,
    mut commands: Commands,
    skills: Res<Assets<SkillDefinition>>,
    skill_map: Res<SkillMap>,
    mut cooldowns_query: Query<&mut SkillCooldowns>,
    mut bonus_stats: ResMut<BonusStats>,
    transforms: Query<&Transform>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    let event = trigger.event();

    let Some(handle) = skill_map.handles.get(&event.skill_id) else {
        return;
    };
    let Some(skill_def) = skills.get(handle) else {
        return;
    };

    // Check cooldown
    if let Some(timer) = cooldowns_query
        .get_mut(event.caster)
        .as_mut()
        .ok()
        .and_then(|cooldowns| cooldowns.get_timer_mut(&event.skill_id))
        && !timer.is_finished()
    {
        // Still on cooldown
        return;
    }

    let mut targets = Vec::new();
    info!(?skill_def.display_name, "triggering skill");

    // Target Resolution
    match skill_def.target {
        TargetType::Identity => {
            targets.push(event.caster);
        }
        TargetType::SingleEnemy => {
            if let Some(target) = event.target {
                targets.push(target);
            }
        }
        TargetType::Point { radius } => {
            // Find enemies in radius of target_position
            if let Some(pos) = event.target_position {
                for (entity, transform) in &enemies {
                    if transform.translation.truncate().distance(pos) <= radius {
                        targets.push(entity);
                    }
                }
            }
        }
        TargetType::AllEnemiesInRange { radius } => {
            if let Ok(caster_transform) = transforms.get(event.caster) {
                for (entity, transform) in &enemies {
                    if transform
                        .translation
                        .truncate()
                        .distance(caster_transform.translation.truncate())
                        <= radius
                    {
                        targets.push(entity);
                    }
                }
            }
        }
    }

    // Process effects
    for (index, effect) in skill_def.effects.iter().enumerate() {
        let mut affected_entities = Vec::new();

        match effect {
            SkillEffect::Damage { amount } => {
                for target in &targets {
                    commands.trigger(DamageRequest {
                        source: event.caster,
                        target: *target,
                        base_damage: *amount,
                        source_tags: skill_def.tags.clone(),
                    });
                    affected_entities.push(*target);
                }
            }
            SkillEffect::DamagePercent { percent: _ } => {
                // Implementation pending health query
            }
            SkillEffect::StatModifier {
                stat_key,
                value,
                mode,
                duration_ms,
            } => {
                let mode_enum = match mode {
                    skills_assets::StatModifierMode::Percent => bonus_stats::StatMode::Percent,
                    skills_assets::StatModifierMode::Multiplicative => {
                        bonus_stats::StatMode::Multiplicative
                    }
                    _ => bonus_stats::StatMode::Additive,
                };

                // TODO! we need to resolve if it is global buff (affects everyone)
                // or only personal (affects caster)
                // If only caster, we need to construct unique key, probably based on
                // `topic:caster:stat_key`
                bonus_stats.add(
                    stat_key,
                    StatBonus {
                        value: *value,
                        mode: mode_enum,
                    },
                );

                if *duration_ms > 0 {
                    // Spawn a tracking entity to remove it later
                    commands.spawn(SkillBuff {
                        source_skill_id: event.skill_id.clone(),
                        stat_key: stat_key.clone(),
                        value: *value,
                        mode: *mode as u8,
                        timer: Timer::from_seconds(*duration_ms as f32 / 1000.0, TimerMode::Once),
                    });
                }
            }
            SkillEffect::ApplyStatus {
                status_id,
                duration_ms,
            } => {
                for target in &targets {
                    commands.entity(*target).insert(StatusEffect {
                        status_id: status_id.clone(),
                        timer: Timer::from_seconds(*duration_ms as f32 / 1000.0, TimerMode::Once),
                    });
                    affected_entities.push(*target);
                }
            }
            _ => todo!("triggered skill effect that has not been implemented yet! {effect:?}"),
        }

        if !affected_entities.is_empty() {
            commands.trigger(SkillEffectApplied {
                caster: event.caster,
                skill_id: event.skill_id.clone(),
                // todo! change this into effect, not just index
                effect_index: index,
                targets: affected_entities,
            });
        }
    }

    // Apply Cooldown
    if skill_def.cooldown_ms > 0 {
        if let Ok(mut cooldowns) = cooldowns_query.get_mut(event.caster) {
            cooldowns.timers.insert(
                event.skill_id.clone(),
                Timer::from_seconds(skill_def.cooldown_ms as f32 / 1000.0, TimerMode::Once),
            );
        } else {
            // Component might not exist, insert it
            let mut timers = std::collections::HashMap::new();
            timers.insert(
                event.skill_id.clone(),
                Timer::from_seconds(skill_def.cooldown_ms as f32 / 1000.0, TimerMode::Once),
            );
            commands
                .entity(event.caster)
                .insert(SkillCooldowns { timers });
        }
    }
}
