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
pub struct TopicSubscribers {
    pub sensors: Vec<Entity>,
}

/// Tracks which unlock definitions have been compiled.
#[derive(Component)]
pub struct CompiledUnlock {
    pub definition_id: String,
}

// ============================================================================
// Generic Sensor Types
// ============================================================================

/// Comparison operators for numeric conditions.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Reflect, serde::Deserialize, serde::Serialize,
)]
pub enum ComparisonOp {
    #[default]
    Ge, // >=
    Le, // <=
    Eq, // ==
    Gt, // >
    Lt, // <
}

/// A sensor that tracks a numeric value against a target.
/// Subscribes to a topic like "kills:goblin", "resource:bones", etc.
#[derive(Component)]
pub struct ValueSensor {
    pub topic: String,
    pub op: ComparisonOp,
    pub target: f32,
}

/// A sensor that waits for a completion status.
/// Subscribes to a topic like "research:bone_sword", "unlock:recipe_x", etc.
#[derive(Component)]
pub struct CompletionSensor {
    pub topic: String,
}
