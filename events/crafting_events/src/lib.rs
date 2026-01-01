use bevy::prelude::*;

/// Event to request starting a crafting operation.
/// Used with observers via commands.trigger().
#[derive(Event)]
pub struct StartCraftingRequest {
    pub recipe_id: String,
}