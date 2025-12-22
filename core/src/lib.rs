use bevy::prelude::*;
use village::VillagePlugin;
use portals::PortalsPlugin;
use wallet::WalletPlugin;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((VillagePlugin, PortalsPlugin, WalletPlugin));
    }
}
