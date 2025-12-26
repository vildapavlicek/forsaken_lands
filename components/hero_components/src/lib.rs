use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Hero;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Damage(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct AttackRange(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AttackSpeed {
    pub timer: Timer,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Projectile;

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct ProjectileTarget(pub Entity);

impl Default for ProjectileTarget {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ProjectileDamage(pub f32);
