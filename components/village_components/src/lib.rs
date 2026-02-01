use {
    bevy::{picking::prelude::Pickable, prelude::*},
    shared_components::IncludeInSave,
    std::collections::HashMap,
};

#[derive(Reflect, Default, Debug, Clone)]
pub struct EncyclopediaEntry {
    pub display_name: String,
    pub kill_count: u64,
    pub escape_count: u64,
    /// The order in which this enemy was encountered.
    pub encounter_order: usize,
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

/// The singleton marker component for the central hub entity.
///
/// This component anchors the persistent state of the player's progression, specifically
/// regarding knowledge (Encyclopedia) and armory (Inventory). It exists as a single
/// entity in the world that persists across save/load cycles.
///
/// # Usage
/// - **Persistence**: Tagged with `IncludeInSave` to ensure all attached progression data
///   (Encyclopedia, Inventory) is serialized to disk.
/// - **Progression Anchor**: Systems query for `Village` to access the `EnemyEncyclopedia`
///   (for unlocks/stats) and `WeaponInventory` (for equipment management).
/// - **Interaction**: Tagged with `Pickable` to allow UI or gameplay interaction (e.g., opening menus).
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[require(EnemyEncyclopedia, WeaponInventory, Pickable, IncludeInSave)]
pub struct Village;

impl EnemyEncyclopedia {
    pub fn increment_kill_count(&mut self, enemy_id: &str, display_name: &str) {
        if let Some(entry) = self.inner.get_mut(enemy_id) {
            entry.kill_count += 1;
        } else {
            let order = self.inner.len();
            self.inner.insert(
                enemy_id.to_string(),
                EncyclopediaEntry {
                    display_name: display_name.to_string(),
                    kill_count: 1,
                    escape_count: 0,
                    encounter_order: order,
                },
            );
        }
    }

    pub fn increment_escape_count(&mut self, enemy_id: &str, display_name: &str) {
        if let Some(entry) = self.inner.get_mut(enemy_id) {
            entry.escape_count += 1;
        } else {
            let order = self.inner.len();
            self.inner.insert(
                enemy_id.to_string(),
                EncyclopediaEntry {
                    display_name: display_name.to_string(),
                    kill_count: 0,
                    escape_count: 1,
                    encounter_order: order,
                },
            );
        }
    }
}
