use bevy::prelude::*;

pub struct MessagesPlugin;

impl Plugin for MessagesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemyKilled>();
    }
}

#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct EnemyKilled {
    pub entity: Entity,
}

impl Default for EnemyKilled {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
