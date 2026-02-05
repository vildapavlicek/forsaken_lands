//! Recipe asset definitions for the crafting system.
//!
//! Recipes are loaded from `.recipe.ron` files and define craftable items.

use {
    bevy::{platform::collections::HashMap, prelude::*},
    bevy_common_assets::ron::RonAssetPlugin,
    serde::{Deserialize, Serialize},
    unlocks_assets::UnlockDefinition,
};

pub struct RecipesAssetsPlugin;

impl Plugin for RecipesAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<RecipeDefinition>::new(&["recipe.ron"]))
            .register_type::<RecipeCategory>()
            .register_type::<CraftingOutcome>();
    }
}

/// Recipe definition loaded from `.recipe.ron` asset files.
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDefinition {
    /// Unique identifier for the recipe (e.g., "bone_sword")
    pub id: String,
    /// Display name shown in UI
    pub display_name: String,
    /// Category for tab-based organization
    pub category: RecipeCategory,
    /// Time in seconds to craft
    pub craft_time: f32,
    /// Resource costs to craft
    pub cost: HashMap<String, u32>,
    /// Results when crafting completes
    pub outcomes: Vec<CraftingOutcome>,

    /// Optional inline unlock definition for when this recipe becomes available
    #[serde(default)]
    pub unlock: Option<UnlockDefinition>,
}

/// Category for organizing recipes into tabs.
#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeCategory {
    #[default]
    Weapons,
    Idols,
    Construction,
}

/// Actions that occur upon crafting completion.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub enum CraftingOutcome {
    /// Adds a quantity of a resource to the player's wallet
    AddResource { id: String, amount: u32 },
    /// Unlocks a specific tech or feature
    UnlockFeature(String),
}
