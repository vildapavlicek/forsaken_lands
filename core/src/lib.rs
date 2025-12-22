use bevy::prelude::*;
use village::VillagePlugin;
use portals::PortalsPlugin;
use wallet::WalletPlugin;
use states::GameState;
use game_assets::AssetsPlugin;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((VillagePlugin, PortalsPlugin, WalletPlugin, AssetsPlugin))
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
