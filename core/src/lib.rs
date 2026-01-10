use {
    bevy::prelude::*, crafting::CraftingPlugin, crafting_resources::CraftingResourcesPlugin,
    crafting_ui::CraftingUiPlugin, divinity_components::DivinityComponentsPlugin,
    enemy_encyclopedia::EnemyEncyclopediaUiPlugin, loading::LoadingManagerPlugin,
    hero_events::HeroEventsPlugin, heroes::HeroesPlugin, portal_assets::PortalAssetsPlugin,
    portal_ui::PortalUiPlugin, portals::PortalsPlugin, research::ResearchPlugin,
    research_ui::ResearchUiPlugin, resources_ui::ResourcesUiPlugin,
    shared_components::SharedComponentsPlugin, states::GameState, system_schedule::GameSchedule::*,
    unlock_notification_ui::UnlockNotificationUiPlugin, unlocks::UnlocksPlugin,
    unlocks_assets::UnlocksAssetsPlugin, village::VillagePlugin, village_ui::VillageUiPlugin,
    wallet::WalletPlugin, widgets::WidgetsPlugin,
};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .configure_sets(
                Update,
                (FrameStart, ResolveIntent, PerformAction, Effect, FrameEnd).chain(),
            )
            .add_plugins((
                LoadingManagerPlugin,
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
                UnlocksAssetsPlugin,
                UnlockNotificationUiPlugin,
                VillagePlugin,
                VillageUiPlugin,
                WalletPlugin,
                WidgetsPlugin,
            ))
            .add_systems(Startup, setup_camera);
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
