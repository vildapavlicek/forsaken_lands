use {
    crate::compiler::*,
    bevy::prelude::*,
    divinity_components::*,
    enemy_components::MonsterId,
    hero_events::EnemyKilled,
    unlocks_assets::*,
    unlocks_components::*,
    unlocks_events::*,
    unlocks_resources::*,
    village_components::{EnemyEncyclopedia, Village},
    wallet::Wallet,
};

/// System that compiles newly loaded unlock definitions.
/// Note: This function is now called inline by LoadingManager, but is kept
/// for potential reuse or dynamic recompilation needs.
#[expect(
    unused,
    reason = "this is kept around in case we would need runtime compilation in the future"
)]
pub fn compile_pending_unlocks(
    mut commands: Commands,
    unlock_assets: Res<Assets<UnlockDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    wallet: Res<Wallet>,
    encyclopedia_query: Query<&EnemyEncyclopedia, With<Village>>,
    unlock_state: Res<UnlockState>,
    village_query: Query<&Divinity, With<Village>>,
    compiled: Query<&CompiledUnlock>,
) {
    let encyclopedia = encyclopedia_query.iter().next();
    let max_divinity = village_query.iter().next().copied();

    let ctx = CompilerContext {
        wallet: &wallet,
        encyclopedia,
        unlock_state: &unlock_state,
        max_divinity,
    };

    // Collect already-compiled IDs for filtering
    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    // Filter out already-compiled and already-unlocked definitions
    // Even though this runs once, checking compiled_ids is safe if we re-enter or something weird happens,
    // but mainly we want to skip already unlocked ones.
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
                    display_name: definition.display_name.clone(),
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

/// Observer for logic signal propagation via ChildOf hierarchy.
pub fn propagate_logic_signal(
    mut trigger: On<LogicSignalEvent>,
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
            commands.trigger(UnlockAchieved {
                unlock_id: root.id.clone(),
                display_name: root.display_name.clone(),
                reward_id: root.reward_id.clone(),
            });
        }
        // Stop bubbling at root - there's nothing above it
        trigger.propagate(false);
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
            // Continue bubbling to parent via ChildOf
            trigger.propagate(true);
        } else {
            // State unchanged, stop bubbling
            trigger.propagate(false);
        }
    } else {
        // This is a sensor or intermediate entity - continue bubbling to parent
        trigger.propagate(true);
    }
}

/// Observer for when an unlock is completed.
pub fn handle_unlock_completion(
    trigger: On<UnlockAchieved>,
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
        commands.trigger(UnlockTopicUpdated {
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
    if let Some(&topic_entity) = topic_map.topics.get(&stat_key) {
        commands.trigger(StatChangedEvent {
            entity: topic_entity,
            stat_id: format!("{}_kills", monster_id.0),
            new_value: kill_count,
        });
    }
}

/// System that checks for Wallet changes and emits resource change signals.
pub fn check_wallet_changes(wallet: Res<Wallet>, topic_map: Res<TopicMap>, mut commands: Commands) {
    // TODO: Replace with `run_if.(resouce_changed_or_removed::<Wallet>)` condition for the system instead of internal check
    if !wallet.is_changed() {
        return;
    }

    // Emit resource updates for all resources
    for (resource_id, &amount) in wallet.resources.iter() {
        let topic_key = format!("resource:{}", resource_id);
        // TODO: refactor to `let else` to flatten structure
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
    // TODO: use let-else pattern to flatten
    if let Some(&topic_entity) = topic_map.topics.get(&topic_key) {
        commands.trigger(UnlockTopicUpdated {
            entity: topic_entity,
            unlock_id: research_id.clone(),
        });
    }
}

/// Observer that updates Stat sensors when a stat changes.
pub fn on_stat_changed(
    trigger: On<StatChangedEvent>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(Entity, &mut ConditionSensor, &StatSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((entity, mut condition, stat_sensor)) = sensors.get_mut(sensor_entity) {
                // Match against enum to get expected stat_id and comparison params
                let (expected_stat_id, op, target_value) = match &stat_sensor.0 {
                    StatCheck::Kills {
                        monster_id,
                        op,
                        value,
                    } => (format!("{}_kills", monster_id), *op, *value),
                    StatCheck::Resource {
                        resource_id,
                        op,
                        value,
                    } => (format!("resource_{}", resource_id), *op, *value),
                };

                if expected_stat_id != event.stat_id {
                    continue;
                }

                let is_met = compare_op(event.new_value, target_value, op);

                debug!(
                    stat_id = %event.stat_id,
                    new_value = %event.new_value,
                    target_value = %target_value,
                    was_met = %condition.is_met,
                    is_met = %is_met,
                    "evaluating stat condition"
                );

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    debug!(sensor = ?entity, is_high = %is_met, "sending logic signal");
                    commands.entity(entity).trigger(|e| LogicSignalEvent {
                        entity: e,
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
    mut sensors: Query<(Entity, &mut ConditionSensor, &ResourceSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((entity, mut condition, resource_sensor)) = sensors.get_mut(sensor_entity) {
                if resource_sensor.0.resource_id != event.resource_id {
                    continue;
                }

                let is_met = event.new_amount >= resource_sensor.0.amount;

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    commands.entity(entity).trigger(|e| LogicSignalEvent {
                        entity: e,
                        is_high: is_met,
                    });
                }
            }
        }
    }
}

/// Observer that updates Unlock sensors when an unlock is completed.
pub fn on_unlock_topic_updated(
    trigger: On<UnlockTopicUpdated>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(Entity, &mut ConditionSensor, &UnlockSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    info!(?event, "received unlock completed event");

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((entity, mut condition, unlock_sensor)) = sensors.get_mut(sensor_entity) {
                if unlock_sensor.0 != event.unlock_id {
                    continue;
                }

                let is_met = true; // UnlockCompleted means it is done.

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    commands.entity(entity).trigger(|e| LogicSignalEvent {
                        entity: e,
                        is_high: is_met,
                    });
                }
            }
        }
    }
}

/// Observer that cleans up the unlock logic graph when an unlock is completed.
pub fn cleanup_finished_unlock(
    trigger: On<UnlockAchieved>,
    mut topic_map: ResMut<TopicMap>,
    roots: Query<(Entity, &UnlockRoot)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let unlock_id = &event.unlock_id;

    // Remove unlock topic
    let topic_key = format!("unlock:{}", unlock_id);
    if let Some(entity) = topic_map.topics.remove(&topic_key) {
        commands.entity(entity).despawn();
    }

    // Find and despawn the root entity
    // This will recursively despawn all children (gates, sensors)
    for (entity, root) in &roots {
        if &root.id == unlock_id {
            debug!(unlock_id = %unlock_id, "cleaned up finished unlock");
            commands.entity(entity).despawn();
            break;
        }
    }
}

/// System that checks for MaxUnlockedDivinity changes and emits signal.
pub fn check_max_divinity_changes(
    query: Query<(Entity, &Divinity), (Changed<Divinity>, With<Village>)>,
    mut topic_map: ResMut<TopicMap>,
    mut commands: Commands,
) {
    for (_entity, max_divinity) in &query {
        let topic_key = "stat:max_unlocked_divinity".to_string();
        let topic_entity = topic_map.get_or_create(&mut commands, &topic_key);

        commands.trigger(MaxUnlockedDivinityChangedEvent {
            entity: topic_entity,
            new_divinity: *max_divinity,
        });
    }
}

/// Observer that updates MaxUnlockedDivinity sensors when the value changes.
pub fn on_max_divinity_changed(
    trigger: On<MaxUnlockedDivinityChangedEvent>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(Entity, &mut ConditionSensor, &MaxUnlockedDivinitySensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let topic_entity = event.entity;

    if let Ok(subs) = subscribers.get(topic_entity) {
        for &sensor_entity in &subs.sensors {
            if let Ok((entity, mut condition, divinity_sensor)) = sensors.get_mut(sensor_entity) {
                let is_met = event.new_divinity >= divinity_sensor.0;

                if condition.is_met != is_met {
                    condition.is_met = is_met;
                    commands.entity(entity).trigger(|e| LogicSignalEvent {
                        entity: e,
                        is_high: is_met,
                    });
                }
            }
        }
    }
}

pub fn clean_up_unlocks(
    mut commands: Commands,
    mut topic_map: ResMut<TopicMap>,
    mut unlock_state: ResMut<UnlockState>,
    unlock_roots: Query<Entity, With<UnlockRoot>>,
    topic_entities: Query<Entity, With<TopicEntity>>,
) {
    debug!("Cleaning up unlocks system state");

    // Despawn all unlock roots (this cleans up the graph)
    for entity in unlock_roots.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn all topic entities
    for entity in topic_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Clear resources
    topic_map.topics.clear();
    unlock_state.completed.clear();
}
