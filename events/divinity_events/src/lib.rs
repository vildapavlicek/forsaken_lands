use bevy::prelude::*;

/// Event to increase divinity XP for a specific entity.
/// Target entity should have Divinity and DivinityStats components.
#[derive(EntityEvent)]
pub struct IncreaseDivinity {
    /// The entity receiving the XP
    #[event_target]
    pub entity: Entity,
    /// Amount of XP to add
    pub xp_amount: f32,
}
