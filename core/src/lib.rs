use {
    bevy::prelude::*, game_assets::AssetsPlugin, heroes::HeroesPlugin, messages::MessagesPlugin,
    portals::PortalsPlugin, research::ResearchPlugin, research_ui::ResearchUiPlugin,
    resources_ui::ResourcesUiPlugin, states::GameState, system_schedule::GameSchedule::*,
    village::VillagePlugin, wallet::WalletPlugin,
};

mod systems;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .configure_sets(
                Update,
                (FrameStart, ResolveIntent, PerformAction, Effect, FrameEnd).chain(),
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
                ResearchUiPlugin,
            ))
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Initializing), systems::spawn_starting_scene)
            .add_systems(
                Update,
                systems::check_scene_spawned.run_if(in_state(GameState::Initializing)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
