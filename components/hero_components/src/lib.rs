use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Hero;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Damage(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AttackRange(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AttackSpeed {
    pub timer: Timer,
}
