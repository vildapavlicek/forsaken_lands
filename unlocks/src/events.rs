use bevy::prelude::*;

/// Signal propagated up the logic tree to a specific gate entity.
#[derive(EntityEvent)]
pub struct LogicSignalEvent {
    /// The target gate entity to receive this signal.
    pub entity: Entity,
    /// True if the signal is high (condition met), false if low.
    pub is_high: bool,
}

/// Fired globally when an unlock's conditions are fully met.
#[derive(Event)]
pub struct UnlockEvent {
    pub unlock_id: String,
    pub reward_id: String,
}

/// Triggered on a Topic Entity when a stat changes.
#[derive(EntityEvent)]
pub struct StatChangedEvent {
    /// The topic entity this event targets.
    pub entity: Entity,
    pub stat_id: String,
    pub new_value: f32,
}

/// Triggered on a Topic Entity when a resource changes.
#[derive(EntityEvent)]
pub struct ResourceChangedEvent {
    /// The topic entity this event targets.
    pub entity: Entity,
    pub resource_id: String,
    pub new_amount: u32,
}

/// Triggered on a Topic Entity when an unlock is completed.
#[derive(EntityEvent)]
pub struct UnlockCompletedEvent {
    /// The topic entity this event targets.
    pub entity: Entity,
    pub unlock_id: String,
}
