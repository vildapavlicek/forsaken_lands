use bevy::prelude::*;
use hero_components::{AttackRange, AttackSpeed, Damage, Hero};
use village_components::Village;

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>()
            .register_type::<Hero>()
            .register_type::<Damage>()
            .register_type::<AttackRange>()
            .register_type::<AttackSpeed>();
    }
}
