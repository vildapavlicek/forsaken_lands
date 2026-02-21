use bevy::{platform::collections::HashMap, prelude::*};

/// A snapshot of an enemy's base statistics and attributes.
///
/// This struct holds the static data extracted from an enemy prefab (DynamicScene)
/// upon unlocking. It decouples the UI from the Scene system, preventing the need
/// to instantiate entities just to read their component data.
///
/// # Usage
/// - **Storage**: Stored in `EnemyDetailsCache` mapped by `MonsterId`.
/// - **Display**: Consumed by the `enemy_encyclopedia` UI to render stats cards.
#[derive(Debug, Clone)]
pub struct EnemyStatBlock {
    /// Maximum health points.
    pub health: f32,
    /// Movement speed in logical pixels per second.
    pub speed: f32,
    /// List of Item IDs (e.g., "resource:bones") dropped upon death.
    pub drops: Vec<String>,
    /// Semantic tags (e.g., "undead", "boss") used for categorization and bonus calculations.
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
