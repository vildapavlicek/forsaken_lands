use {
    bevy::prelude::*,
    village_components::{EncyclopediaEntry, EnemyEncyclopedia, Village},
};

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>();
        app.register_type::<EnemyEncyclopedia>();
        app.register_type::<EncyclopediaEntry>();
    }
}
