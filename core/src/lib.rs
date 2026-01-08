use {
    bevy::prelude::*, crafting::CraftingPlugin, crafting_resources::CraftingResourcesPlugin,
    crafting_ui::CraftingUiPlugin, divinity_components::DivinityComponentsPlugin,
    enemy_encyclopedia::EnemyEncyclopediaUiPlugin,
    game_assets::AssetsPlugin, hero_events::HeroEventsPlugin, heroes::HeroesPlugin,
    portal_ui::PortalUiPlugin, portals::PortalsPlugin, research::ResearchPlugin,
    portal_assets::PortalAssetsPlugin,
    research_ui::ResearchUiPlugin, resources_ui::ResourcesUiPlugin,
    shared_components::SharedComponentsPlugin, states::GameState,
    system_schedule::GameSchedule::*, village::VillagePlugin, village_ui::VillageUiPlugin,
    wallet::WalletPlugin, widgets::WidgetsPlugin, unlocks::UnlocksPlugin,
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
                DivinityComponentsPlugin,
                EnemyEncyclopediaUiPlugin,
                HeroesPlugin,
                HeroEventsPlugin,
                PortalsPlugin,
                PortalAssetsPlugin,
            ))
            .add_plugins((
                PortalUiPlugin,
                ResearchPlugin,
                ResearchUiPlugin,
                ResourcesUiPlugin,
                SharedComponentsPlugin,
                UnlocksPlugin,
                VillagePlugin,
                VillageUiPlugin,
                WalletPlugin,
                WidgetsPlugin,
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
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::AutoMin {
                min_width: 300.0,
                min_height: 700.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
