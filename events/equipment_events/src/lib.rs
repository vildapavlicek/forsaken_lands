use bevy::prelude::*;

/// Request to equip a weapon to a hero.
/// If the hero already has a weapon, it will be unequipped first.
#[derive(Event)]
pub struct EquipWeaponRequest {
    pub hero: Entity,
    pub weapon: Entity,
}

/// Request to unequip the current weapon from a hero.
#[derive(Event)]
pub struct UnequipWeaponRequest {
    pub hero: Entity,
}
