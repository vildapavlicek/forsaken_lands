use {bevy::prelude::*, std::collections::HashSet};

pub struct VillageResourcesPlugin;

impl Plugin for VillageResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DivinityUnlockState>()
            .register_type::<DivinityUnlockState>();
    }
}

/// Tracks which divinity unlock IDs have already granted their level-up reward.
///
/// This is persisted in save files to prevent duplicate divinity increases when
/// loading a game. Unlike `UnlockState` which is intentionally not persisted
/// (to allow re-computation of idempotent rewards), divinity level-ups are
/// permanent progression that should only be granted once per unlock.
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct DivinityUnlockState {
    /// Set of unlock IDs that have already granted divinity level-ups.
    pub claimed: HashSet<String>,
}
