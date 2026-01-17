use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(EquippedWeaponId)]
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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Weapon;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RangedWeapon;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MeleeWeapon;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MeleeArc {
    pub width: f32, // In radians
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Damage(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct AttackRange(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AttackSpeed {
    pub timer: Timer,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Projectile;

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct ProjectileTarget(pub Entity);

impl Default for ProjectileTarget {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ProjectileDamage(pub f32);
