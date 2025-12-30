use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Default)]
pub struct Wallet {
    pub resources: HashMap<String, u32>,
}

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wallet>().init_resource::<Wallet>();
    }
}
