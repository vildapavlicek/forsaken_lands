use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Debug, Clone)]
pub struct EnemyStatBlock {
    pub health: f32,
    pub speed: f32,
    pub drops: Vec<String>,
    pub tags: Vec<String>,
}

/// A centralized cache for enemy statistics (e.g., health, drops) derived from prefabs.
///
/// This resource serves as the data source for the "Enemy Encyclopedia" UI, allowing the
/// game to display information about unlocked enemies without needing to instantiate
/// their entities or parse scene files every frame.
///
/// # Usage
/// - **Population**: The `cache_details_on_unlock` observer (in `PortalsPlugin`) populates this
///   cache when an `UnlockAchieved` event is triggered for an enemy.
/// - **Query**: The `enemy_encyclopedia` UI system queries this resource to populate
///   informational cards for the player.
#[derive(Resource, Default, Debug)]
pub struct EnemyDetailsCache {
    /// Maps a `MonsterId` (string) to its cached statistical block.
    pub details: HashMap<String, EnemyStatBlock>,
}
