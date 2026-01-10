use {
    bevy::{log::LogPlugin, prelude::*},
    core::CorePlugin,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(LogPlugin {
                filter:
                    "error,loading=debug,portals=trace,village=debug,wallet=debug,heroes=debug,unlocks=trace"
                        .into(),
                level: bevy::log::Level::TRACE,
                ..Default::default()
            }),
        )
        .add_plugins(CorePlugin)
        .run();
}
