use {
    bevy::{log::LogPlugin, prelude::*},
    core::CorePlugin,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(LogPlugin {
                filter: "error,loading=trace,\
                    portals=debug,\
                    village=debug,\
                    wallet=debug,\
                    heroes=debug,\
                    unlocks=info,\
                    save_load=trace,\
                    village_ui=debug"
                    .into(),
                level: bevy::log::Level::TRACE,
                ..Default::default()
            }),
        )
        .add_plugins(CorePlugin)
        .run();
}
