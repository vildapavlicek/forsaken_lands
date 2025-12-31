use bevy::prelude::*;
use village::VillagePlugin;
use portals::PortalsPlugin;
use wallet::WalletPlugin;
use states::GameState;
use game_assets::AssetsPlugin;
use heroes::HeroesPlugin;
use messages::MessagesPlugin;
use resources_ui::ResourcesUiPlugin;
use research::ResearchPlugin;
use system_schedule::GameSchedule::*;

mod systems;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .configure_sets(
                Update,
                (
                    FrameStart,
                    ResolveIntent,
                    PerformAction,
                    Effect,
                    FrameEnd,
                )
                    .chain(),
            )
            .add_plugins((
                VillagePlugin,
                PortalsPlugin,
                WalletPlugin,
                AssetsPlugin,
                HeroesPlugin,
                MessagesPlugin,
                ResourcesUiPlugin,
                ResearchPlugin,
            ))
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Running), systems::spawn_starting_scene);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
