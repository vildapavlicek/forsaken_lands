use bevy::prelude::*;

/// Request to spawn a weapon. The factory handles spawning + inventory.
#[derive(Event, Clone, Debug)]
pub struct SpawnWeaponRequest {
    /// Weapon definition ID (e.g., "bone_sword")
    pub weapon_id: String,
    /// Optional parent entity (Hero) for equipped weapons
    pub parent: Option<Entity>,
    /// Add to village WeaponInventory (true for crafted, false for reconstruction)
    pub add_to_inventory: bool,
}
