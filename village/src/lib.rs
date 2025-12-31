use {bevy::prelude::*, village_components::Village};

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>();
    }
}
