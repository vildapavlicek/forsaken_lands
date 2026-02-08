use {
    bevy::prelude::*,
    enemy_components::Enemy,
    hero_components::{AttackRange, Hero},
    skill_components::{EquippedSkills, SkillCooldowns},
    skill_events::SkillActivated,
    skills_assets::{SkillDefinition, SkillEffect, SkillMap, SkillType, TargetType},
    village_components::Village,
};

/// System that automatically triggers hero skills of type `AutoActivate`
/// when enemies are within the defensive perimeter.
pub fn hero_auto_activate_skills_system(
    mut commands: Commands,
    heroes: Query<(Entity, &EquippedSkills, &SkillCooldowns, Option<&AttackRange>), With<Hero>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    villages: Query<&Transform, With<Village>>,
    skill_map: Res<SkillMap>,
    skills: Res<Assets<SkillDefinition>>,
) {
    let Ok(village_transform) = villages.single() else {
        return;
    };

    for (hero_entity, equipped, cooldowns, hero_range) in &heroes {
        for skill_id in &equipped.0 {
            // Check if skill is on cooldown
            if let Some(timer) = cooldowns.timers.get(skill_id) {
                if !timer.is_finished() {
                    continue;
                }
            }

            // Get skill definition
            let Some(handle) = skill_map.handles.get(skill_id) else {
                continue;
            };
            let Some(skill_def) = skills.get(handle) else {
                continue;
            };

            // Filter 1: Type must be AutoActivate
            if !matches!(skill_def.skill_type, SkillType::AutoActivate) {
                continue;
            }

            // Filter 2: Only attacking skills (must have Damage or DamagePercent)
            let is_attacking = skill_def.effects.iter().any(|e| {
                matches!(
                    e,
                    SkillEffect::Damage { .. } | SkillEffect::DamagePercent { .. }
                )
            });
            if !is_attacking {
                continue;
            }

            // Target Resolution & Range Check
            match &skill_def.target {
                TargetType::SingleEnemy { range } => {
                    let mut closest: Option<(Entity, f32)> = None;

                    for (enemy_entity, enemy_transform) in &enemies {
                        let dist = village_transform
                            .translation
                            .distance(enemy_transform.translation);

                        if dist <= *range {
                            if let Some((_, closest_dist)) = closest {
                                if dist < closest_dist {
                                    closest = Some((enemy_entity, dist));
                                }
                            } else {
                                closest = Some((enemy_entity, dist));
                            }
                        }
                    }

                    if let Some((enemy_entity, _)) = closest {
                        commands.trigger(SkillActivated {
                            caster: hero_entity,
                            skill_id: skill_id.clone(),
                            target: Some(enemy_entity),
                            target_position: None,
                        });
                    }
                }
                TargetType::AllEnemiesInRange { radius } => {
                    let mut any_in_range = false;
                    for (_, enemy_transform) in &enemies {
                        let dist = village_transform
                            .translation
                            .distance(enemy_transform.translation);

                        if dist <= *radius {
                            any_in_range = true;
                            break;
                        }
                    }

                    if any_in_range {
                        commands.trigger(SkillActivated {
                            caster: hero_entity,
                            skill_id: skill_id.clone(),
                            target: None,
                            target_position: None,
                        });
                    }
                }
                _ => {
                    // Skip other target types (Identity, Point, etc.) for auto-activation attacking skills
                }
            }
        }
    }
}
