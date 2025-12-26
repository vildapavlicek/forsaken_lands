use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RewardCoefficient(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct NeedsHydration;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Lifetime(pub Timer);
