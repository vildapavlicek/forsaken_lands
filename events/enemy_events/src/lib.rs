use bevy::prelude::*;

pub struct EnemyEventsPlugin;

impl Plugin for EnemyEventsPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Triggered when an enemy's lifetime expires and they despawn.
#[derive(Event, Debug)]
pub struct EnemyEscaped {
    pub entity: Entity,
}
