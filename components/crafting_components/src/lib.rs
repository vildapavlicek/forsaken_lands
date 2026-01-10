//! Components for the crafting system.

use {bevy::prelude::*, recipes_assets::RecipeDefinition};

pub struct CraftingComponentsPlugin;

impl Plugin for CraftingComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RecipeNode>();
    }
}

/// Associates an entity with a recipe definition.
/// Similar to ResearchNode for research entities.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RecipeNode {
    /// The unique recipe identifier
    pub id: String,
    /// Handle to the recipe asset definition
    pub handle: Handle<RecipeDefinition>,
}
