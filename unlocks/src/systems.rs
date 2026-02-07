//! Systems and observers for the unlocks framework.

use {
    crate::compiler::*, bevy::prelude::*, unlocks_assets::UnlockDefinition, unlocks_components::*,
    unlocks_events::*, unlocks_resources::*,
};

// ============================================================================
// Compile System (called by game code)
// ============================================================================

/// System that compiles newly loaded unlock definitions.
/// This is called by game code (e.g., LoadingManager) after assets are loaded.
/// This is called by game code (e.g., LoadingManager) after assets are loaded.
pub fn compile_pending_unlocks(
    mut commands: Commands,
    unlock_assets: Res<Assets<UnlockDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    unlock_state: Res<UnlockState>,
    unlock_progress: Res<UnlockProgress>,
    compiled: Query<&CompiledUnlock>,
) {
    // Collect already-compiled IDs for filtering
    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    // Filter out already-compiled and already-unlocked definitions
    let pending_definitions = unlock_assets
        .iter()
        .map(|(_, def)| def)
        .filter(|def| !compiled_ids.contains(def.id.as_str()));

    for definition in pending_definitions {
        compile_unlock_definition(
            &mut commands,
            &mut topic_map,
            definition,
            &compiled_ids,
            &unlock_state,
            &unlock_progress,
        );
    }
}

// ============================================================================
// Logic Signal Propagation
// ============================================================================

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

// ============================================================================
// Unlock Completion Handler
// ============================================================================

// ============================================================================
// Unlock Completion Handler
// ============================================================================

/// Observer for when an unlock is completed.
pub fn handle_unlock_completion(
    trigger: On<UnlockAchieved>,
    mut unlock_state: ResMut<UnlockState>,
    mut unlock_progress: ResMut<UnlockProgress>,
    topic_map: Res<TopicMap>,
    mut commands: Commands,
) {
    let event = trigger.event();
    info!(unlock_id = %event.unlock_id, reward_id = %event.reward_id, "Processing unlock");

    // Persist progress
    *unlock_progress
        .counts
        .entry(event.unlock_id.clone())
        .or_insert(0) += 1;

    // Mark as completed in session state (improves lookup perf)
    if !unlock_state.completed.contains(&event.unlock_id) {
        unlock_state.completed.push(event.unlock_id.clone());
    }

    // Notify sensors waiting for this unlock as a dependency
    let topic_key = format!("unlock:{}", event.unlock_id);
    if topic_map.topics.contains_key(&topic_key) {
        // Trigger StatusCompleted so dependent sensors update
        commands.trigger(StatusCompleted { topic: topic_key });
    }
}

// ============================================================================
// Generic Event Observers
// ============================================================================

/// Observer that updates ValueSensor components when a value changes.
pub fn on_value_changed(
    trigger: On<ValueChanged>,
    topic_map: Res<TopicMap>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(Entity, &mut ConditionSensor, &ValueSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Find the topic entity for this topic key
    let Some(&topic_entity) = topic_map.topics.get(&event.topic) else {
        return;
    };

    let Ok(subs) = subscribers.get(topic_entity) else {
        return;
    };

    for &sensor_entity in &subs.sensors {
        if let Ok((entity, mut condition, value_sensor)) = sensors.get_mut(sensor_entity) {
            // Verify this sensor is subscribed to the same topic
            if value_sensor.topic != event.topic {
                continue;
            }

            let is_met = compare_op(event.value, value_sensor.target, value_sensor.op);

            debug!(
                topic = %event.topic,
                value = %event.value,
                target = %value_sensor.target,
                was_met = %condition.is_met,
                is_met = %is_met,
                "evaluating value condition"
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

/// Observer that updates CompletionSensor components when a status is completed.
pub fn on_status_completed(
    trigger: On<StatusCompleted>,
    topic_map: Res<TopicMap>,
    subscribers: Query<&TopicSubscribers>,
    mut sensors: Query<(Entity, &mut ConditionSensor, &CompletionSensor)>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Find the topic entity for this topic key
    let Some(&topic_entity) = topic_map.topics.get(&event.topic) else {
        return;
    };

    let Ok(subs) = subscribers.get(topic_entity) else {
        return;
    };

    for &sensor_entity in &subs.sensors {
        if let Ok((entity, mut condition, completion_sensor)) = sensors.get_mut(sensor_entity) {
            // Verify this sensor is subscribed to the same topic
            if completion_sensor.topic != event.topic {
                continue;
            }

            // Completion is always a one-way transition (false -> true)
            if !condition.is_met {
                condition.is_met = true;
                debug!(sensor = ?entity, topic = %event.topic, "marking completion sensor as met");
                commands.entity(entity).trigger(|e| LogicSignalEvent {
                    entity: e,
                    is_high: true,
                });
            }
        }
    }
}

// ============================================================================
// Cleanup / Lifecycle
// ============================================================================

/// Observer that manages the graph lifecycle when an unlock is completed.
/// Either despawns it (if done) or resets it (if repeatable).
pub fn handle_unlock_lifecycle(
    trigger: On<UnlockAchieved>,
    mut topic_map: ResMut<TopicMap>,
    mut roots: Query<(Entity, &UnlockRoot, Option<&mut RepeatableUnlock>)>,
    children: Query<&Children>,
    mut condition_sensors: Query<&mut ConditionSensor>,
    mut logic_gates: Query<&mut LogicGate>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let unlock_id = &event.unlock_id;

    // Find the root entity for this unlock
    let Some((root_entity, _root, list_repeatable)) =
        roots.iter_mut().find(|(_, r, _)| r.id == *unlock_id)
    else {
        return;
    };

    let mut should_despawn = true;

    // Check if repeatable
    if let Some(mut repeatable) = list_repeatable {
        repeatable.trigger_count += 1;

        let is_exhausted = repeatable
            .max_triggers
            .map(|max| repeatable.trigger_count >= max)
            .unwrap_or(false);

        if !is_exhausted {
            should_despawn = false;
            debug!(unlock_id = %unlock_id, count = repeatable.trigger_count, "Resetting repeatable unlock");

            // Reset the graph sensors and gates
            reset_condition_tree(
                root_entity,
                &children,
                &mut condition_sensors,
                &mut logic_gates,
            );
        }
    }

    if should_despawn {
        debug!(unlock_id = %unlock_id, "Cleaning up finished unlock");

        // Remove unlock topic
        let topic_key = format!("unlock:{}", unlock_id);
        if let Some(entity) = topic_map.topics.remove(&topic_key) {
            commands.entity(entity).despawn();
        }

        commands.entity(root_entity).despawn();
    }
}

/// Recursively resets all sensors and gates in the tree to their initial state.
fn reset_condition_tree(
    entity: Entity,
    children_query: &Query<&Children>,
    sensors: &mut Query<&mut ConditionSensor>,
    gates: &mut Query<&mut LogicGate>,
) {
    // Reset sensor
    if let Ok(mut sensor) = sensors.get_mut(entity) {
        sensor.is_met = false;
    }

    // Reset gate
    if let Ok(mut gate) = gates.get_mut(entity) {
        gate.current_signals = 0;
        gate.was_active = match gate.operator {
            LogicOperator::Not => true, // NOT starts true when 0 signals
            _ => false,
        };
    }

    // Recurse
    if let Ok(children) = children_query.get(entity) {
        for &child in children {
            reset_condition_tree(child, children_query, sensors, gates);
        }
    }
}

pub fn clean_up_unlocks(
    mut commands: Commands,
    mut topic_map: ResMut<TopicMap>,
    mut unlock_state: ResMut<UnlockState>,
    mut unlock_progress: ResMut<UnlockProgress>,
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
    unlock_progress.counts.clear();
}
