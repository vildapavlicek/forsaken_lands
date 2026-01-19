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
        app.register_type::<SpawnType>();
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

/// Defines what to spawn: a single monster or a group of monsters together.
#[derive(Reflect, Debug, Clone, Deserialize, Serialize)]
pub enum SpawnType {
    /// Spawn a single monster by ID.
    Single(String),
    /// Spawn a group of monsters together (all at once).
    Group(Vec<String>),
}

impl Default for SpawnType {
    fn default() -> Self {
        Self::Single("goblin_scout".to_string())
    }
}

#[derive(Reflect, Debug, Clone, Default, Deserialize, Serialize)]
pub struct SpawnEntry {
    pub condition: SpawnCondition,
    pub spawn_type: SpawnType,
    pub weight: u32,
}

#[derive(Asset, TypePath, Default, Debug, Deserialize, Serialize)]
pub struct SpawnTable {
    pub entries: Vec<SpawnEntry>,
}
