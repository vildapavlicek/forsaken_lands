use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    divinity_components::Divinity,
    serde::{Deserialize, Serialize},
};

pub struct PortalAssetsPlugin;

impl Plugin for PortalAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnCondition>();
        app.register_type::<SpawnEntry>();

        // Register the asset loader for .spawn_table.ron files
        app.add_plugins(RonAssetPlugin::<SpawnTable>::new(&["spawn_table.ron"]));
    }
}

#[derive(Reflect, Debug, Clone, Deserialize, Serialize)]
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

#[derive(Reflect, Debug, Clone, Default, Deserialize, Serialize)]
pub struct SpawnEntry {
    pub condition: SpawnCondition,
    pub monster_id: String, // References MonsterId component value
    pub weight: u32,
}

#[derive(Asset, TypePath, Default, Debug, Deserialize, Serialize)]
pub struct SpawnTable {
    pub entries: Vec<SpawnEntry>,
}
