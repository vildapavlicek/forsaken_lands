use bevy::prelude::*;

pub mod the_maw;

pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(the_maw::TheMawPlugin);
    }
}
