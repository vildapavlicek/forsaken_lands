use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Dead;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Lifetime(pub Timer);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct ResourceRewards(pub Vec<Reward>);

#[derive(Reflect, Default, Debug, Clone)]
pub struct Reward {
    pub id: String,
    pub value: u32,
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component, Default)]
pub struct MonsterId(pub String);
