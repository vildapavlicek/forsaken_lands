use bevy::prelude::*;

/// Signal propagated up the logic tree via ChildOf hierarchy.
/// Uses Bevy 0.17 event bubbling - triggers on sensor/gate entities and propagates to parents.
/// The `entity` field is auto-filled when using `commands.entity(e).trigger()`.
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

/// Triggered on a Topic Entity when a stat changes.
#[derive(EntityEvent)]
pub struct StatChangedEvent {
    /// The topic entity this event targets.
    #[event_target]
    pub entity: Entity,
    pub stat_id: String,
    pub new_value: f32,
}

/// Triggered on a Topic Entity when a resource changes.
#[derive(EntityEvent)]
pub struct ResourceChangedEvent {
    /// The topic entity this event targets.
    #[event_target]
    pub entity: Entity,
    pub resource_id: String,
    pub new_amount: u32,
}

/// Triggered on a Topic Entity when an unlock is completed used to update dependent nodes.
#[derive(Debug, EntityEvent)]
pub struct UnlockTopicUpdated {
    /// The topic entity this event targets.
    #[event_target]
    pub entity: Entity,
    pub unlock_id: String,
}

/// Triggered on a Topic Entity when MaxUnlockedDivinity changes.
#[derive(EntityEvent)]
pub struct MaxUnlockedDivinityChangedEvent {
    /// The topic entity this event targets.
    #[event_target]
    pub entity: Entity,
    pub new_divinity: divinity_components::Divinity,
}
