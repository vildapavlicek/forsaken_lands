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

#[derive(Asset, TypePath, Resource, Default, Debug, Deserialize)]
pub struct SpawnTable {
    pub entries: Vec<SpawnEntry>,
}
