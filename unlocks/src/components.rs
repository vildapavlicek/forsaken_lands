use bevy::prelude::*;

/// Tag for the root entity of an unlock definition.
#[derive(Component)]
pub struct UnlockRoot {
    pub id: String,
    pub display_name: Option<String>,
    pub reward_id: String,
}

/// Represents a boolean logic gate in the ECS world.
/// Parent relationship is tracked via ChildOf component for event bubbling.
#[derive(Component)]
pub struct LogicGate {
    pub operator: LogicOperator,
    /// How many positive signals needed (AND = children count, OR = 1).
    pub required_signals: usize,
    /// Current number of active positive signals from children.
    pub current_signals: usize,
    /// Previous state to detect transitions.
    pub was_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperator {
    And,
    Or,
    Not, // Special: inverts single child
}

/// Represents a Leaf Node (Sensor).
/// Parent relationship is tracked via ChildOf component for event bubbling.
#[derive(Component)]
pub struct ConditionSensor {
    pub is_met: bool,
}

/// Marker for topic entities (event channels).
#[derive(Component)]
pub struct TopicEntity {
    pub key: String,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct TopicSubscribers {
    pub sensors: Vec<Entity>,
}

#[derive(Component)]
pub struct StatSensor(pub crate::assets::StatCheck);

#[derive(Component)]
pub struct ResourceSensor(pub crate::assets::ResourceCheck);

#[derive(Component)]
pub struct UnlockSensor(pub String);

/// Tracks which unlock definitions have been compiled.
#[derive(Component)]
pub struct CompiledUnlock {
    pub definition_id: String,
}
