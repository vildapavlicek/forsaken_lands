use crate::{assets::*, compiler::*, components::*, events::*, resources::*};
use bevy::prelude::*;
use enemy_components::MonsterId;
use hero_events::EnemyKilled;
use research::ResearchState;
use village_components::{EnemyEncyclopedia, Village};
use wallet::Wallet;

/// System that compiles newly loaded unlock definitions.
pub fn compile_pending_unlocks(
    mut commands: Commands,
    unlock_assets: Res<Assets<UnlockDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    wallet: Res<Wallet>,
    encyclopedia_query: Query<&EnemyEncyclopedia, With<Village>>,
    research_state: Res<ResearchState>,
    unlock_state: Res<UnlockState>,
    compiled: Query<&CompiledUnlock>,
) {
    let encyclopedia = encyclopedia_query.single().ok();

    let ctx = CompilerContext {
        wallet: &wallet,
        encyclopedia,
        research_state: &research_state,
    };

    // Collect already-compiled IDs for filtering
    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    // Filter out already-compiled and already-unlocked definitions
    let pending_definitions = unlock_assets
        .iter()
        .map(|(_, def)| def)
        .filter(|def| !compiled_ids.contains(def.id.as_str()))
        .filter(|def| !unlock_state.is_unlocked(&def.id));

    for definition in pending_definitions {
        debug!(unlock_id = %definition.id, "Compiling unlock definition");

        // Spawn root entity
        let root = commands
            .spawn((
                UnlockRoot {
                    id: definition.id.clone(),
                    reward_id: definition.reward_id.clone(),
                },
                CompiledUnlock {
                    definition_id: definition.id.clone(),
                },
            ))
            .id();

        // Build the condition tree
        build_condition_node(
            &mut commands,
            &mut topic_map,
            &definition.condition,
            root,
            &ctx,
        );
    }
}

/// Observer for logic signal propagation.
pub fn propagate_logic_signal(
    trigger: On<LogicSignalEvent>,
    mut gates: Query<(Entity, &mut LogicGate)>,
    roots: Query<&UnlockRoot>,
    mut commands: Commands,
) {
    let signal = trigger.event();
    let gate_entity = signal.entity;

    trace!(target = ?gate_entity, is_high = %signal.is_high, "reacting to logic signal event");

    // Check if this is a root first
    if let Ok(root) = roots.get(gate_entity) {
        if signal.is_high {
            info!(unlock_id = %root.id, "Unlock achieved!");
            commands.trigger(UnlockEvent {
                unlock_id: root.id.clone(),
                reward_id: root.reward_id.clone(),
            });
        }
        return;
    }

    // Handle logic gate
    if let Ok((_, mut gate)) = gates.get_mut(gate_entity) {
        // Update counter
        if signal.is_high {
            gate.current_signals += 1;
        } else {
            gate.current_signals = gate.current_signals.saturating_sub(1);
        }

        // Determine new state
        let is_active = match gate.operator {
            LogicOperator::And => gate.current_signals >= gate.required_signals,
            LogicOperator::Or => gate.current_signals > 0,
            LogicOperator::Not => gate.current_signals == 0, // Inverts child
        };

        // Only propagate if state changed
        if is_active != gate.was_active {
            gate.was_active = is_active;
            commands.trigger(LogicSignalEvent {
                entity: gate.parent,
                is_high: is_active,
            });
        }
    }
}

/// Observer for when an unlock is completed.
pub fn handle_unlock_completion(
    trigger: On<UnlockEvent>,
    mut unlock_state: ResMut<UnlockState>,
    topic_map: Res<TopicMap>,
    mut commands: Commands,
) {
    let event = trigger.event();
    info!(unlock_id = %event.unlock_id, reward_id = %event.reward_id, "Processing unlock");

    // Mark as completed
    if !unlock_state.completed.contains(&event.unlock_id) {
        unlock_state.completed.push(event.unlock_id.clone());
    }

    // Notify topic for unlock dependencies
    let topic_key = format!("unlock:{}", event.unlock_id);
    if let Some(&topic_entity) = topic_map.topics.get(&topic_key) {
        commands.trigger(UnlockCompletedEvent {
            entity: topic_entity,
            unlock_id: event.unlock_id.clone(),
        });
    }
}

/// Observer that intercepts EnemyKilled events and emits stat change signals.
pub fn on_enemy_killed_stat_update(
    trigger: On<EnemyKilled>,
    topic_map: Res<TopicMap>,
    monster_query: Query<&MonsterId>,
    encyclopedia_query: Query<&EnemyEncyclopedia, With<Village>>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Get the monster ID
    let Ok(monster_id) = monster_query.get(event.entity) else {
        return;
    };

    // Get the current kill count from encyclopedia
    let encyclopedia = encyclopedia_query.single().ok();
    let kill_count = encyclopedia
        .and_then(|enc| enc.inner.get(&monster_id.0))
        .map(|e| e.kill_count as f32)
        .unwrap_or(1.0); // At least 1 since we just killed one

    // Emit stat change to topic entity for this monster's kills
    let stat_key = format!("stat:{}_kills", monster_id.0);
    debug!(%stat_key, ?monster_id, %kill_count, "looking for a topic for enemy kill");
    if let Some(&topic_entity) = topic_map.topics.get(&stat_key) {
        debug!(%stat_key, "found topic");
        commands.trigger(StatChangedEvent {
            entity: topic_entity,
            stat_id: format!("{}_kills", monster_id.0),
            new_value: kill_count,
        });
    }
}

/// System that checks for Wallet changes and emits resource change signals.
pub fn check_wallet_changes(wallet: Res<Wallet>, topic_map: Res<TopicMap>, mut commands: Commands) {
    // Only run if wallet changed
    if !wallet.is_changed() {
        return;
    }

    // Emit resource updates for all resources
    for (resource_id, &amount) in wallet.resources.iter() {
        let topic_key = format!("resource:{}", resource_id);
        if let Some(&topic_entity) = topic_map.topics.get(&topic_key) {
            commands.trigger(ResourceChangedEvent {
                entity: topic_entity,
                resource_id: resource_id.clone(),
                new_amount: amount,
            });
        }
    }
}

/// Observer that handles research completion and emits unlock signals.
pub fn on_research_completed(
    trigger: On<research::ResearchCompleted>,
    topic_map: Res<TopicMap>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let research_id = &event.research_id;

    // Emit unlock events for completed research
    let topic_key = format!("unlock:{}", research_id);
    if let Some(&topic_entity) = topic_map.topics.get(&topic_key) {
        commands.trigger(UnlockCompletedEvent {
            entity: topic_entity,
            unlock_id: research_id.clone(),
        });
    }
}

/// Observer that updates Stat sensors when a stat changes.
pub fn on_stat_changed(
    trigger: On<StatChangedEvent>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(&mut ConditionSensor, &StatSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((mut condition, stat_sensor)) = sensors.get_mut(sensor_entity) {
                // Double check stat ID matches (should be guaranteed by topic wiring but good for safety)
                if stat_sensor.0.stat_id != event.stat_id {
                    continue;
                }

                let is_met = compare_op(event.new_value, stat_sensor.0.value, stat_sensor.0.op);

                debug!(
                    stat_id = %event.stat_id,
                    new_value = %event.new_value,
                    target_value = %stat_sensor.0.value,
                    was_met = %condition.is_met,
                    is_met = %is_met,
                    "evaluating stat condition"
                );

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    debug!(parent = ?condition.parent, is_high = %is_met, "sending logic signal");
                    commands.trigger(LogicSignalEvent {
                        entity: condition.parent,
                        is_high: is_met,
                    });
                }
            }
        }
    }
}

/// Observer that updates Resource sensors when a resource changes.
pub fn on_resource_changed(
    trigger: On<ResourceChangedEvent>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(&mut ConditionSensor, &ResourceSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((mut condition, resource_sensor)) = sensors.get_mut(sensor_entity) {
                if resource_sensor.0.resource_id != event.resource_id {
                    continue;
                }

                let is_met = event.new_amount >= resource_sensor.0.amount;

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    commands.trigger(LogicSignalEvent {
                        entity: condition.parent,
                        is_high: is_met,
                    });
                }
            }
        }
    }
}

/// Observer that updates Unlock sensors when an unlock is completed.
pub fn on_unlock_completed_notification(
    trigger: On<UnlockCompletedEvent>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(&mut ConditionSensor, &UnlockSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    info!(?event, "received unlock completed event");

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((mut condition, unlock_sensor)) = sensors.get_mut(sensor_entity) {
                if unlock_sensor.0 != event.unlock_id {
                    continue;
                }

                let is_met = true; // UnlockCompleted means it is done.

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    commands.trigger(LogicSignalEvent {
                        entity: condition.parent,
                        is_high: is_met,
                    });
                }
            }
        }
    }
}
