use bevy::prelude::*;
use divinity_components::Divinity;

pub struct PortalResourcesPlugin;

impl Plugin for PortalResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnCondition>();
        app.register_type::<SpawnEntry>();
        app.register_type::<SpawnTable>();
    }
}

#[derive(Reflect, Debug, Clone)]
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

#[derive(Reflect, Debug, Clone, Default)]
pub struct SpawnEntry {
    pub condition: SpawnCondition,
    pub monster_file: String, // e.g. "goblin"
    pub weight: u32,
}

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct SpawnTable {
    pub entries: Vec<SpawnEntry>,
}
