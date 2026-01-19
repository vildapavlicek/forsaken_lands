use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    serde::{Deserialize, Serialize},
    unlocks_components::ComparisonOp,
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
///
/// This is a simplified, game-agnostic version that uses string-based topic IDs.
/// The game code is responsible for triggering events with matching topic IDs.
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
    /// Checks if a numeric value meets a threshold.
    /// Topic examples: "kills:goblin", "resource:bones", "xp:total", "divinity:max"
    Value {
        topic: String,
        #[serde(default)]
        op: ComparisonOp,
        target: f32,
    },
    /// Checks if something has been completed.
    /// Topic examples: "research:bone_sword", "quest:intro", "unlock:recipe_x"
    Completed { topic: String },
}
