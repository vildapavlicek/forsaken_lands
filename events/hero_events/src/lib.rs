use bevy::prelude::*;

pub struct HeroEventsPlugin;

impl Plugin for HeroEventsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackIntent>()
            .register_type::<ProjectileHit>();
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
