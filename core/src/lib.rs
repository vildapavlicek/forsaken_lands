use bevy::prelude::*;
use village::VillagePlugin;
use portals::PortalsPlugin;
use wallet::WalletPlugin;
use states::GameState;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((VillagePlugin, PortalsPlugin, WalletPlugin));
    }
}
