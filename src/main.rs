use bevy::prelude::*;
use core::CorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugin)
        .run();
}
