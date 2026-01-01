use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, Reflect, Default)]
pub struct RecipesLibrary {
    pub recipes: HashMap<String, CraftingRecipe>,
}

/// Represents the merged crafting recipe definition.
#[derive(Reflect, Default, Debug, Clone)]
pub struct CraftingRecipe {
    /// The unique ID used by Research to unlock this (e.g., "wooden_bow")
    pub id: String,
    /// Display name for UI
    pub display_name: String,
    /// Time in seconds to craft
    pub craft_time: f32,
    /// The inputs required to craft the item.
    pub cost: HashMap<String, u32>,
    /// The outputs produced by the craft.
    pub outcomes: Vec<CraftingOutcome>,
}

/// Represents distinct actions that occur upon crafting completion.
#[derive(Reflect, Clone, Debug)]
pub enum CraftingOutcome {
    /// Spawns a prefab by its asset key/path
    SpawnPrefab(String),
    /// Adds a quantity of a resource to the player's wallet
    AddResource { id: String, amount: u32 },
    /// Unlocks a specific tech or feature
    UnlockFeature(String),
    /// Grants Experience points
    GrantXp(u32),
}

pub struct CraftingResourcesPlugin;

impl Plugin for CraftingResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CraftingRecipe>()
            .register_type::<CraftingOutcome>()
            .register_type::<RecipesLibrary>();

        app.init_resource::<RecipesLibrary>();
    }
}
