//! Compiles UnlockDefinition assets into runtime ECS logic graphs.

use {
    bevy::prelude::*, divinity_components::*, unlocks_assets::*, unlocks_components::*,
    unlocks_events::*, unlocks_resources::*, village_components::EnemyEncyclopedia, wallet::Wallet,
};

/// Compares values using the specified operator.
pub fn compare_op(current: f32, target: f32, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Ge => current >= target,
        ComparisonOp::Le => current <= target,
        ComparisonOp::Eq => (current - target).abs() < f32::EPSILON,
        ComparisonOp::Gt => current > target,
        ComparisonOp::Lt => current < target,
    }
}

/// Context passed during graph compilation for state hydration.
pub struct CompilerContext<'a> {
    pub wallet: &'a Wallet,
    pub encyclopedia: Option<&'a EnemyEncyclopedia>,
    pub unlock_state: &'a UnlockState,
    pub max_divinity: Option<&'a MaxUnlockedDivinity>,
}

pub struct AddTopicSubscriber {
    pub topic: Entity,
    pub sensor: Entity,
}

impl Command for AddTopicSubscriber {
    fn apply(self, world: &mut World) {
        if let Some(mut sub) = world.get_mut::<TopicSubscribers>(self.topic) {
            sub.sensors.push(self.sensor);
        }
    }
}

/// Recursively builds the condition node tree.
pub fn build_condition_node(
    commands: &mut Commands,
    topic_map: &mut TopicMap,
    node: &ConditionNode,
    parent: Entity,
    ctx: &CompilerContext,
) -> Entity {
    match node {
        ConditionNode::And(children) => {
            let gate = commands
                .spawn((
                    ChildOf(parent),
                    LogicGate {
                        operator: LogicOperator::And,
                        required_signals: children.len(),
                        current_signals: 0,
                        was_active: false,
                    },
                ))
                .id();

            for child in children {
                build_condition_node(commands, topic_map, child, gate, ctx);
            }
            gate
        }
        ConditionNode::Or(children) => {
            let gate = commands
                .spawn((
                    ChildOf(parent),
                    LogicGate {
                        operator: LogicOperator::Or,
                        required_signals: 1,
                        current_signals: 0,
                        was_active: false,
                    },
                ))
                .id();

            for child in children {
                build_condition_node(commands, topic_map, child, gate, ctx);
            }
            gate
        }
        ConditionNode::Not(child) => {
            let gate = commands
                .spawn((
                    ChildOf(parent),
                    LogicGate {
                        operator: LogicOperator::Not,
                        required_signals: 1,
                        current_signals: 0,
                        was_active: true, // NOT starts as "true" when child is false
                    },
                ))
                .id();

            build_condition_node(commands, topic_map, child, gate, ctx);
            gate
        }
        ConditionNode::Stat(check) => {
            let topic_key = check.topic_key();
            let topic_entity = topic_map.get_or_create(commands, &topic_key);

            // Hydrate current value based on stat type
            let current_val = match check {
                StatCheck::Kills { monster_id, .. } => ctx
                    .encyclopedia
                    .and_then(|enc| enc.inner.get(monster_id))
                    .map(|e| e.kill_count as f32)
                    .unwrap_or(0.0),
                StatCheck::Resource { .. } => 0.0, // Resources don't hydrate from encyclopedia
            };

            let (op, target_value) = check.comparison();
            let initially_met = compare_op(current_val, target_value, op);

            let sensor = commands
                .spawn((
                    ChildOf(parent),
                    ConditionSensor {
                        is_met: initially_met,
                    },
                    StatSensor(check.clone()),
                ))
                .id();

            commands.queue(AddTopicSubscriber {
                topic: topic_entity,
                sensor,
            });

            if initially_met {
                commands.entity(sensor).trigger(|entity| LogicSignalEvent {
                    entity,
                    is_high: true,
                });
            }

            sensor
        }
        ConditionNode::Resource(check) => {
            let topic_key = format!("resource:{}", check.resource_id);
            let topic_entity = topic_map.get_or_create(commands, &topic_key);

            let current_amount = ctx
                .wallet
                .resources
                .get(&check.resource_id)
                .copied()
                .unwrap_or(0);
            let initially_met = current_amount >= check.amount;

            let sensor = commands
                .spawn((
                    ChildOf(parent),
                    ConditionSensor {
                        is_met: initially_met,
                    },
                    ResourceSensor(check.clone()),
                ))
                .id();

            commands.queue(AddTopicSubscriber {
                topic: topic_entity,
                sensor,
            });

            if initially_met {
                commands.entity(sensor).trigger(|entity| LogicSignalEvent {
                    entity,
                    is_high: true,
                });
            }

            sensor
        }
        ConditionNode::Unlock(unlock_id) => {
            let topic_key = format!("unlock:{}", unlock_id);
            let topic_entity = topic_map.get_or_create(commands, &topic_key);

            let initially_met = ctx.unlock_state.is_unlocked(unlock_id);

            let sensor = commands
                .spawn((
                    ChildOf(parent),
                    ConditionSensor {
                        is_met: initially_met,
                    },
                    UnlockSensor(unlock_id.clone()),
                ))
                .id();

            commands.queue(AddTopicSubscriber {
                topic: topic_entity,
                sensor,
            });

            if initially_met {
                commands.entity(sensor).trigger(|entity| LogicSignalEvent {
                    entity,
                    is_high: true,
                });
            }

            sensor
        }
        ConditionNode::PortalsMaxUnlockedDivinity(required_divinity) => {
            let topic_key = "stat:max_unlocked_divinity".to_string();
            let topic_entity = topic_map.get_or_create(commands, &topic_key);

            let current_divinity = ctx.max_divinity.map(|d| d.0).unwrap_or_default();
            let initially_met = current_divinity >= *required_divinity;

            let sensor = commands
                .spawn((
                    ChildOf(parent),
                    ConditionSensor {
                        is_met: initially_met,
                    },
                    MaxUnlockedDivinitySensor(*required_divinity),
                ))
                .id();

            commands.queue(AddTopicSubscriber {
                topic: topic_entity,
                sensor,
            });

            if initially_met {
                commands.entity(sensor).trigger(|entity| LogicSignalEvent {
                    entity,
                    is_high: true,
                });
            }

            sensor
        }
        ConditionNode::True => {
            // Always met - immediately signal parent to trigger unlock completion
            // No sensor needed since this condition can never change
            let sensor = commands
                .spawn((ChildOf(parent), ConditionSensor { is_met: true }))
                .id();

            // Immediately signal parent - this will propagate up to UnlockRoot
            // and fire UnlockAchieved, adding this unlock to UnlockState.completed
            commands.entity(sensor).trigger(|e| LogicSignalEvent {
                entity: e,
                is_high: true,
            });

            sensor
        }
    }
}
