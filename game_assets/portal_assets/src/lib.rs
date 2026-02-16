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

#[derive(Reflect, Debug, Clone, Deserialize, Serialize, PartialEq)]
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
#[derive(Reflect, Debug, Clone, Deserialize, Serialize, PartialEq)]
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

#[derive(Reflect, Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct SpawnEntry {
    pub condition: SpawnCondition,
    pub spawn_type: SpawnType,
    pub weight: u32,
}

/// Defines a collection of spawn rules used by Portals to generate enemies.
///
/// This asset acts as a configuration file (typically loaded from `.spawn_table.ron`) that
/// determines *what* can spawn and *when* (based on `Divinity` levels).
///
/// # Usage
/// - **Configuration**: Designers create these assets to define the enemy composition for different
///   areas or portal types (e.g., "Forest Portal" vs "Dungeon Portal").
/// - **Runtime**: The `enemy_spawn_system` retrieves this asset using the `SpawnTableId` component
///   on a `Portal` entity. It then filters `entries` by the portal's `CurrentDivinity` and selects
///   one based on weight.
#[derive(Asset, TypePath, Default, Debug, Deserialize, Serialize, PartialEq)]
pub struct SpawnTable {
    /// The list of potential spawn candidates and their conditions.
    pub entries: Vec<SpawnEntry>,
}
