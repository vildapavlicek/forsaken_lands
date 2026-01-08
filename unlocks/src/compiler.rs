//! Compiles UnlockDefinition assets into runtime ECS logic graphs.

use crate::{assets::*, components::*, events::*, resources::*};
use bevy::prelude::*;

use research::ResearchState;
use village_components::EnemyEncyclopedia;
use wallet::Wallet;

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
    pub research_state: &'a ResearchState,
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
                .spawn(LogicGate {
                    operator: LogicOperator::And,
                    required_signals: children.len(),
                    current_signals: 0,
                    was_active: false,
                    parent,
                })
                .id();

            for child in children {
                build_condition_node(commands, topic_map, child, gate, ctx);
            }
            gate
        }
        ConditionNode::Or(children) => {
            let gate = commands
                .spawn(LogicGate {
                    operator: LogicOperator::Or,
                    required_signals: 1,
                    current_signals: 0,
                    was_active: false,
                    parent,
                })
                .id();

            for child in children {
                build_condition_node(commands, topic_map, child, gate, ctx);
            }
            gate
        }
        ConditionNode::Not(child) => {
            let gate = commands
                .spawn(LogicGate {
                    operator: LogicOperator::Not,
                    required_signals: 1,
                    current_signals: 0,
                    was_active: true, // NOT starts as "true" when child is false
                    parent,
                })
                .id();

            build_condition_node(commands, topic_map, child, gate, ctx);
            gate
        }
        ConditionNode::Stat(check) => {
            let topic_key = format!("stat:{}", check.stat_id);
            let topic_entity = topic_map.get_or_create(commands, &topic_key);

            // Hydrate from encyclopedia if available
            let current_val = ctx
                .encyclopedia
                .and_then(|enc| enc.inner.get(&check.stat_id))
                .map(|e| e.kill_count as f32)
                .unwrap_or(0.0);
            let initially_met = compare_op(current_val, check.value, check.op);

            let sensor = commands
                .spawn((
                    ConditionSensor {
                        parent,
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
                commands.trigger(LogicSignalEvent {
                    entity: parent,
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
                    ConditionSensor {
                        parent,
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
                commands.trigger(LogicSignalEvent {
                    entity: parent,
                    is_high: true,
                });
            }

            sensor
        }
        ConditionNode::Unlock(unlock_id) => {
            let topic_key = format!("unlock:{}", unlock_id);
            let topic_entity = topic_map.get_or_create(commands, &topic_key);

            let initially_met = ctx.research_state.is_researched(unlock_id);

            let sensor = commands
                .spawn((
                    ConditionSensor {
                        parent,
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
                commands.trigger(LogicSignalEvent {
                    entity: parent,
                    is_high: true,
                });
            }

            sensor
        }
    }
}
