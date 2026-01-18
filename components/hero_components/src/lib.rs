use bevy::prelude::*;
use shared_components::IncludeInSave;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(EquippedWeaponId, IncludeInSave)]
pub struct Hero;

/// Stable identifier for weapons that persists across save/load.
/// This ID is used to reference weapons by name rather than entity ID.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct WeaponId(pub String);

/// References which weapon a hero has equipped by its WeaponId.
/// Uses Option<String> to represent no weapon equipped (None).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct EquippedWeaponId(pub Option<String>);

/// Marker component for weapon entities.
/// Weapons are filtered out during save and reconstructed from WeaponInventory on load.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Weapon;

// === Definition components (reconstructed from .weapon.ron, no Reflect needed) ===

#[derive(Component, Default)]
pub struct RangedWeapon;

#[derive(Component, Default)]
pub struct MeleeWeapon;

#[derive(Component, Default)]
pub struct MeleeArc {
    pub width: f32, // In radians
}

#[derive(Component, Default)]
pub struct Damage(pub f32);

#[derive(Component, Default)]
pub struct AttackRange(pub f32);

#[derive(Component, Default)]
pub struct AttackSpeed {
    pub timer: Timer,
}

#[derive(Component, Default)]
pub struct Projectile;

#[derive(Component)]
pub struct ProjectileTarget(pub Entity);

impl Default for ProjectileTarget {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Default)]
pub struct ProjectileDamage(pub f32);

