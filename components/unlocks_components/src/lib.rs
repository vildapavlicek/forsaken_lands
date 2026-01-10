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

#[derive(Component)]
pub struct StatSensor(pub StatCheck);

#[derive(Component)]
pub struct ResourceSensor(pub ResourceCheck);

#[derive(Component)]
pub struct UnlockSensor(pub String);

/// Tracks which unlock definitions have been compiled.
#[derive(Component)]
pub struct CompiledUnlock {
    pub definition_id: String,
}

// --- Shared Types moved from assets.rs to avoid circular deps ---

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

#[derive(Debug, Clone, PartialEq, Reflect, serde::Deserialize, serde::Serialize)]
pub struct StatCheck {
    pub stat_id: String,
    pub value: f32,
    #[serde(default)]
    pub op: ComparisonOp,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect, serde::Deserialize, serde::Serialize)]
pub struct ResourceCheck {
    pub resource_id: String,
    pub amount: u32,
}

// NOTE: These depend on crate::assets types which we agreed to keep in unlocks/assets.rs
// However, circular dependency: unlocks_components would need to depend on unlocks (for assets),
// and unlocks depends on unlocks_components.
// ERROR: We have a circular dependency issue if StatCheck/ResourceCheck are strictly in `unlocks`.
// I MUST fix this. `StatCheck` and `ResourceCheck` define the data.
// Since the user said "Keep assets in unlocks/src/assets.rs for now", this implies a potential issue if I try to use them here.
// But `ConditionSensor` wrapping `crate::assets::StatCheck` was not in the file I read?
// Wait, looking at `components.rs`:
// pub struct StatSensor(pub crate::assets::StatCheck);
// pub struct ResourceSensor(pub crate::assets::ResourceCheck);
// Yes they are there.
// If I move `StatSensor` here, I need `StatCheck`.
// If `StatCheck` stays in `unlocks`, then `unlocks_components` needs `unlocks`.
// But `unlocks` needs `unlocks_components`. Cycle!
//
// Strategy: I will temporarily COPY `StatCheck` and `ResourceCheck` (and related enums) to `unlocks_components` or a `unlocks_shared` module within it?
// Or I should put `StatSensor` and `ResourceSensor` NOT in `unlocks_components` but keep them in `unlocks`?
// But the directive is "Move components to unlocks_components".
//
// Let's look at `assets.rs` really quick to see what `StatCheck` is.
// I'll peek at `unlocks/src/assets.rs` via `view_file` to decide.
// For now I will comment out the modules depending on assets or write them assuming I can move `StatCheck` too (which technically is an asset struct but also a data struct).
// Actually, it's better to move `StatCheck` / `ResourceCheck` to `unlocks_components` or `unlocks_assets` (if I could make one).
// Since the user said "Keep it as is for now" regarding assets... maybe they meant the `UnlockDefinition` asset loading part?
// `StatCheck` and `ResourceCheck` are just structs. They are used in components.
// I will attempt to define them in `unlocks_components` or `unlocks_components::types` if I can't move the file.
// Wait, I can define `StatSensor` in `unlocks` crate if I must, but that defeats the purpose of splitting.
//
// Better plan: Move `StatCheck`, `ResourceCheck`, etc. to `unlocks_components` as pure data structs. They can be re-exported by `unlocks::assets`.
// I'll write the file but comment out the problematic lines for a moment until I see `assets.rs`.
// actually, I'll view `assets.rs` first.
