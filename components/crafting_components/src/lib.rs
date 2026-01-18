//! Components for the crafting system.

use {bevy::prelude::*, recipes_assets::RecipeDefinition};

pub struct CraftingComponentsPlugin;

impl Plugin for CraftingComponentsPlugin {
    fn build(&self, _app: &mut App) {
        // RecipeNode not registered - contains Handle which can't be serialized
    }
}

/// Associates an entity with a recipe definition.
/// Similar to ResearchNode for research entities.
/// Does not derive Reflect because Handle<T> contains Arc<StrongHandle> which can't be serialized.
#[derive(Component)]
pub struct RecipeNode {
    /// The unique recipe identifier
    pub id: String,
    /// Handle to the recipe asset definition
    pub handle: Handle<RecipeDefinition>,
}

