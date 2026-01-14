use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    serde::{Deserialize, Serialize},
    unlocks_components::{ResourceCheck, StatCheck},
};

pub struct UnlocksAssetsPlugin;

impl Plugin for UnlocksAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<UnlockDefinition>::new(&["unlock.ron"]));
    }
}

/// The top-level asset definition for an unlockable item.
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct UnlockDefinition {
    /// Unique key for this unlock (e.g., "recipe_bone_sword").
    pub id: String,
    /// Optional metadata for UI display.
    pub display_name: Option<String>,
    /// The root node of the logical condition tree.
    pub condition: ConditionNode,
    /// Abstract identifier for the reward (processed by downstream systems).
    pub reward_id: String,
}

/// A node in the logical condition tree.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConditionNode {
    // --- Logic Gates ---
    /// Requires ALL sub-conditions to be true.
    And(Vec<ConditionNode>),
    /// Requires ANY sub-condition to be true.
    Or(Vec<ConditionNode>),
    /// Inverts the result of the sub-condition.
    Not(Box<ConditionNode>),
    /// Always true - for unlocks with no prerequisites.
    True,

    // --- Leaf Sensors ---
    /// Checks if a numeric statistic meets a threshold.
    Stat(StatCheck),
    /// Checks if the player possesses a specific resource.
    Resource(ResourceCheck),
    /// Checks if a specific research/recipe is already unlocked.
    Unlock(String),
    /// Checks if the portal's max unlocked divinity is at least this high.
    PortalsMaxUnlockedDivinity(divinity_components::Divinity),
}
