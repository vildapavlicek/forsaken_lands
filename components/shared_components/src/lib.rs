use bevy::prelude::*;

pub struct SharedComponentsPlugin;

impl Plugin for SharedComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackBonus>();
        app.register_type::<BonusValue>();
    }
}

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Default)]
pub struct BonusValue {
    pub flat: f32,
    pub percentage: f32, // e.g. 0.1 for 10%
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, Default)]
pub struct AttackBonus {
    pub all: BonusValue,
    pub melee: BonusValue,
    pub ranged: BonusValue,
}
