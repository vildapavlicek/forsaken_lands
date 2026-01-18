use {
    bevy::{picking::prelude::Pickable, prelude::*},
    shared_components::IncludeInSave,
    std::collections::HashMap,
};

#[derive(Reflect, Default, Debug, Clone)]
pub struct EncyclopediaEntry {
    pub display_name: String,
    pub kill_count: u64,
}

/// Stores the history of defeated enemies and their statistics.
///
/// This component acts as the persistent memory for enemy interactions, primarily used for
/// unlocking content (e.g., "Kill 10 Goblins"). It is attached to the `Village` singleton entity.
///
/// # Usage
/// - **Unlocks System**: Queries this component to evaluate `StatCheck::Kills` conditions.
/// - **UI System**: Displays the list of encountered enemies and their kill counts.
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct EnemyEncyclopedia {
    /// Maps enemy IDs (e.g., "goblin_scout") to their historical data.
    pub inner: HashMap<String, EncyclopediaEntry>,
}

/// Tracks all crafted/owned weapons at the village level.
/// Stores weapon IDs that can be used to spawn weapons from prefabs.
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct WeaponInventory {
    pub weapons: Vec<String>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[require(EnemyEncyclopedia, WeaponInventory, Pickable, IncludeInSave)]
pub struct Village;

impl EnemyEncyclopedia {
    pub fn increment_kill_count(&mut self, enemy_id: String, display_name: String) {
        self.inner
            .entry(enemy_id)
            .and_modify(|e| e.kill_count += 1)
            .or_insert(EncyclopediaEntry {
                display_name,
                kill_count: 1,
            });
    }
}
