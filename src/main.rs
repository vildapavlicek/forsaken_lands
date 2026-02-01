use {
    bevy::{log::LogPlugin, prelude::*},
    bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin},
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
                    bonus_stats=trace,\
                    village_ui=debug"
                    .into(),
                level: bevy::log::Level::TRACE,
                ..Default::default()
            }),
        )
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
        .add_plugins(CorePlugin)
        .run();
}
