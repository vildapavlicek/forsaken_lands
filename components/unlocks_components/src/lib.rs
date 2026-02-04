use bevy::prelude::*;

/// The root anchor of an active unlock logic graph in the ECS.
///
/// This component sits at the top of the logic hierarchy (`ConditionSensor` -> `LogicGate` -> `UnlockRoot`).
/// It represents an in-progress unlock requirement (e.g., "Kill 10 Goblins").
///
/// # Usage
/// - **Signal Termination**: The `propagate_logic_signal` observer bubbles `LogicSignalEvent` up the
///   `ChildOf` hierarchy. When the signal reaches this component and is High, the unlock is achieved.
/// - **Event Trigger**: Upon receiving a positive signal, this component's data is used to fire the
///   `UnlockAchieved` event (Observer).
/// - **Lifecycle**: The `cleanup_finished_unlock` system queries this component to identify and
///   recursively despawn the entire condition tree once the unlock is completed.
/// - **Instantiation**: Spawned by `loading::compile_unlocks` from an `UnlockDefinition` asset.
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

/// Defines the boolean logic behavior for a `LogicGate`.
///
/// This enum determines how signals from child entities (Conditions or other Gates)
/// are aggregated to determine the state of the parent Gate.
///
/// # Usage
/// - **Unlocks System**: Used in `propagate_logic_signal` to evaluate if an unlock condition is met.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperator {
    /// Requires ALL child conditions to be met (Signal count == Child count).
    /// Used for "Complete X AND Y" objectives.
    And,
    /// Requires ANY child condition to be met (Signal count > 0).
    /// Used for "Complete X OR Y" objectives.
    Or,
    /// Inverts the signal of its single child (Signal count == 0 is TRUE).
    /// Used for "Complete X but NOT Y" (rare) or gating logic.
    Not,
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
