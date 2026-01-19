//! Compiles UnlockDefinition assets into runtime ECS logic graphs.

use {
    bevy::prelude::*,
    unlocks_assets::ConditionNode,
    unlocks_components::*,
    unlocks_events::*,
    unlocks_resources::*,
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
/// 
/// This is a simplified version that doesn't require game-specific context.
/// Initial state hydration is handled by the game triggering ValueChanged/StatusCompleted events.
pub fn build_condition_node(
    commands: &mut Commands,
    topic_map: &mut TopicMap,
    node: &ConditionNode,
    parent: Entity,
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
                build_condition_node(commands, topic_map, child, gate);
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
                build_condition_node(commands, topic_map, child, gate);
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

            build_condition_node(commands, topic_map, child, gate);
            gate
        }
        ConditionNode::Value { topic, op, target } => {
            let topic_entity = topic_map.get_or_create(commands, topic);

            // Sensors start as not met - game will trigger ValueChanged to hydrate
            let sensor = commands
                .spawn((
                    ChildOf(parent),
                    ConditionSensor { is_met: false },
                    ValueSensor {
                        topic: topic.clone(),
                        op: *op,
                        target: *target,
                    },
                ))
                .id();

            commands.queue(AddTopicSubscriber {
                topic: topic_entity,
                sensor,
            });

            sensor
        }
        ConditionNode::Completed { topic } => {
            let topic_entity = topic_map.get_or_create(commands, topic);

            // Sensors start as not met - game will trigger StatusCompleted to hydrate
            let sensor = commands
                .spawn((
                    ChildOf(parent),
                    ConditionSensor { is_met: false },
                    CompletionSensor {
                        topic: topic.clone(),
                    },
                ))
                .id();

            commands.queue(AddTopicSubscriber {
                topic: topic_entity,
                sensor,
            });

            sensor
        }
        ConditionNode::True => {
            // Always met - immediately signal parent to trigger unlock completion
            let sensor = commands
                .spawn((ChildOf(parent), ConditionSensor { is_met: true }))
                .id();

            // Immediately signal parent
            commands.entity(sensor).trigger(|e| LogicSignalEvent {
                entity: e,
                is_high: true,
            });

            sensor
        }
    }
}
