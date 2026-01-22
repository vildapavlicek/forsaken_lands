//! Resources for the crafting system.

use bevy::{platform::collections::HashMap, prelude::*};
// Re-export types from recipes_assets for backwards compatibility
pub use recipes_assets::{CraftingOutcome, RecipeCategory, RecipeDefinition};

// --- Legacy Resource (kept for migration, will be removed) ---

/// DEPRECATED: Use RecipeMap with entity queries instead.
/// This resource remains temporarily for backwards compatibility during migration.
#[derive(Resource, Debug, Reflect, Default)]
#[reflect(Resource)]
pub struct RecipesLibrary {
    pub recipes: HashMap<String, CraftingRecipe>,
}

/// DEPRECATED: Use RecipeDefinition asset instead.
/// This type remains temporarily for backwards compatibility during migration.
#[derive(Reflect, Default, Debug, Clone)]
pub struct CraftingRecipe {
    pub id: String,
    pub display_name: String,
    pub category: RecipeCategory,
    pub craft_time: f32,
    pub required_research: Option<String>,
    pub cost: HashMap<String, u32>,
    pub outcomes: Vec<CraftingOutcome>,
}

// --- New Entity-Based Resources ---

/// O(1) lookup of recipe entities by ID.
/// Similar to ResearchMap for research entities.
#[derive(Resource, Default)]
pub struct RecipeMap {
    pub entities: HashMap<String, Entity>,
}

/// Tracks recipes that have been constructed 1-time.
/// Primarily used for 'Construction' category items (buildings).
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct ConstructedBuildings {
    pub ids: bevy::platform::collections::HashSet<String>,
}

// --- Plugin ---

pub struct CraftingResourcesPlugin;

impl Plugin for CraftingResourcesPlugin {
    fn build(&self, app: &mut App) {
        // Legacy types (kept for migration)
        app.register_type::<CraftingRecipe>()
            .register_type::<RecipesLibrary>()
            .init_resource::<RecipesLibrary>();

        // New entity-based resources
        app.init_resource::<RecipeMap>()
           .register_type::<ConstructedBuildings>()
           .init_resource::<ConstructedBuildings>();
    }
}
