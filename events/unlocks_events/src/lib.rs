use bevy::prelude::*;

/// Signal propagated up the logic tree via ChildOf hierarchy.
/// Uses Bevy 0.17 event bubbling - triggers on sensor/gate entities and propagates to parents.
#[derive(EntityEvent)]
#[entity_event(propagate)]
pub struct LogicSignalEvent {
    /// The entity this event targets (auto-filled by entity-targeted trigger).
    #[event_target]
    pub entity: Entity,
    /// True if the signal is high (condition met), false if low.
    pub is_high: bool,
}

/// Fired globally when an unlock's conditions are fully met.
#[derive(Event)]
pub struct UnlockAchieved {
    pub unlock_id: String,
    pub display_name: Option<String>,
    pub reward_id: String,
}

// ============================================================================
// Generic Events - User triggers these to notify the unlock system
// ============================================================================

/// Triggered by game code when any numeric value changes.
/// The library will check if any sensors are subscribed to this topic.
#[derive(Event)]
pub struct ValueChanged {
    /// The topic ID, e.g. "kills:goblin", "resource:bones", "xp:total"
    pub topic: String,
    /// The new value
    pub value: f32,
}

/// Triggered by game code when something is completed (research, quest, etc).
/// The library will check if any sensors are waiting for this topic.
#[derive(Event)]
pub struct StatusCompleted {
    /// The topic ID, e.g. "research:bone_sword", "quest:intro"
    pub topic: String,
}

// ============================================================================
// Constants
// ============================================================================

/// Prefix for crafting completion topics (used in StatusCompleted).
/// Usage: `craft:{recipe_id}`
pub const CRAFTING_TOPIC_PREFIX: &str = "craft:";
