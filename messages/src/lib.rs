use bevy::prelude::*;

pub struct MessagesPlugin;

impl Plugin for MessagesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackIntent>()
            .register_type::<ProjectileHit>()
            .register_type::<EnemyKilled>();
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

#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct AttackIntent {
    pub attacker: Entity,
    pub target: Entity,
}

impl Default for AttackIntent {
    fn default() -> Self {
        Self {
            attacker: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct ProjectileHit {
    pub projectile: Entity,
    pub target: Entity,
    pub damage: f32,
}

impl Default for ProjectileHit {
    fn default() -> Self {
        Self {
            projectile: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
            damage: 0.0,
        }
    }
}
