use {
    bevy::prelude::*, crafting::CraftingPlugin, crafting_resources::CraftingResourcesPlugin,
    crafting_ui::CraftingUiPlugin, game_assets::AssetsPlugin, hero_events::HeroEventsPlugin,
    heroes::HeroesPlugin, portals::PortalsPlugin, research::ResearchPlugin,
    research_ui::ResearchUiPlugin, resources_ui::ResourcesUiPlugin,
    shared_components::SharedComponentsPlugin, states::GameState, system_schedule::GameSchedule::*,
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
                AssetsPlugin,
                CraftingPlugin,
                CraftingResourcesPlugin,
                CraftingUiPlugin,
                HeroesPlugin,
                HeroEventsPlugin,
                PortalsPlugin,
                ResearchPlugin,
                ResearchUiPlugin,
                ResourcesUiPlugin,
                SharedComponentsPlugin,
                VillagePlugin,
                WalletPlugin,
            ))
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::Initializing),
                systems::spawn_starting_scene,
            )
            .add_systems(
                Update,
                systems::check_scene_spawned.run_if(in_state(GameState::Initializing)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
