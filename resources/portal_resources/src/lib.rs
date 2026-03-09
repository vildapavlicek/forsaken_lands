use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use divinity_components::Divinity;
use serde::Deserialize;

pub struct PortalResourcesPlugin;

impl Plugin for PortalResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnCondition>();
        app.register_type::<SpawnEntry>();

        // Register the asset loader for .spawn_table.ron files
        app.add_plugins(RonAssetPlugin::<SpawnTable>::new(&["spawn_table.ron"]));
    }
}

#[derive(Reflect, Debug, Clone, Deserialize)]
pub enum SpawnCondition {
    /// Spawns only at this exact Divinity.
    Specific(Divinity),
    /// Spawns between min and max (inclusive).
    Range { min: Divinity, max: Divinity },
    /// Spawns at this Divinity or higher.
    Min(Divinity),
}

impl Default for SpawnCondition {
    fn default() -> Self {
        Self::Min(Divinity::default())
    }
}

#[derive(Reflect, Debug, Clone, Default, Deserialize)]
pub struct SpawnEntry {
    pub condition: SpawnCondition,
    pub monster_file: String, // e.g. "goblin"
    pub weight: u32,
}

/// An asset defining the composition of enemy waves for a Portal.
///
/// This struct represents the data loaded from a `.spawn_table.ron` file, detailing
/// which enemies can spawn under specific `Divinity` conditions and their relative weights.
///
/// # Usage
/// - **Spawning**: The `enemy_spawn_system` queries `Assets<SpawnTable>` using the
///   `SpawnTableId` component attached to a `Portal` to determine the next enemy to spawn.
/// - **Filtering**: Entries are filtered against the `CurrentDivinity` resource before
///   a weighted random selection is made.
#[derive(Asset, TypePath, Resource, Default, Debug, Deserialize)]
pub struct SpawnTable {
    pub entries: Vec<SpawnEntry>,
}
