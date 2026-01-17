use bevy::prelude::*;

/// Event to request starting a crafting operation.
/// Used with observers via commands.trigger().
#[derive(Event)]
pub struct StartCraftingRequest {
    pub recipe_id: String,
}

/// Event triggered when a weapon has been crafted.
/// Observer in village crate listens to this to add to inventory.
#[derive(Event)]
pub struct WeaponCrafted {
    pub weapon_id: String,
}
