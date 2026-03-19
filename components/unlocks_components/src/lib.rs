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

/// Component for unlocks that can be triggered multiple times.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct RepeatableUnlock {
    /// Maximum number of times this unlock can be triggered.
    /// If None, it can be triggered infinitely.
    pub max_triggers: Option<u32>,
    /// Number of times this unlock has been triggered in the current session.
    /// Note: Total progress across sessions is tracked in `UnlockProgress` resource.
    pub trigger_count: u32,
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

/// A registry of sensor entities subscribed to a specific event topic.
///
/// This component is attached to a `TopicEntity` and acts as an optimization for the Event-Driven Architecture.
/// Instead of broadcasting updates to every `ConditionSensor` in the world, systems can query this
/// component to identify exactly which entities need to be notified of a change.
///
/// # Usage
/// - **Compilation**: Populated by `compile_unlock_definition` when building the logic graph.
///   Each leaf node (Sensor) is registered here.
/// - **Event Dispatch**: Read by `on_value_changed` and `on_status_completed` observers
///   to efficiently route updates to the relevant `ConditionSensor` entities.
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

/// Defines how a `ValueSensor` evaluates its tracked numeric value against its target.
///
/// This enum determines the logical condition under which an unlock requirement is considered met.
///
/// # Usage
/// - **Unlocks System**: Queried by the `on_value_changed` observer (in the `unlocks` crate)
///   when processing events (e.g., `IncreaseKills`, `AddResource`). The observer reads this operator
///   from a `ValueSensor` to compare the new global state against the sensor's target value.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Reflect, serde::Deserialize, serde::Serialize,
)]
pub enum ComparisonOp {
    /// The tracked value must be greater than or equal to the target.
    /// Used for accumulation goals (e.g., "Kill 10 or more Goblins").
    #[default]
    Ge,
    /// The tracked value must be less than or equal to the target.
    /// Used for restriction challenges (e.g., "Complete level with 5 or fewer deaths").
    Le,
    /// The tracked value must exactly match the target.
    /// Used for precise sequence or state checks (e.g., "Maintain exactly 3 active wards").
    Eq,
    /// The tracked value must be strictly greater than the target.
    /// Used when a threshold must be exceeded.
    Gt,
    /// The tracked value must be strictly less than the target.
    /// Used for strict restriction challenges.
    Lt,
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
