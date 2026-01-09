use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
// #[serde(tag = "type", content = "data")]
pub enum ConditionNode {
    // --- Logic Gates ---
    /// Requires ALL sub-conditions to be true.
    And(Vec<ConditionNode>),
    /// Requires ANY sub-condition to be true.
    Or(Vec<ConditionNode>),
    /// Inverts the result of the sub-condition.
    Not(Box<ConditionNode>),

    // --- Leaf Sensors ---
    /// Checks if a numeric statistic meets a threshold.
    Stat(StatCheck),
    /// Checks if the player possesses a specific resource.
    Resource(ResourceCheck),
    /// Checks if a specific research/recipe is already unlocked.
    Unlock(String),
}

pub use unlocks_components::{ComparisonOp, ResourceCheck, StatCheck};
