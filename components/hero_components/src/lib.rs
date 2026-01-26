use {bevy::prelude::*, shared_components::IncludeInSave};

/// The primary marker component for the player-controlled character.
///
/// This singleton component acts as the central anchor for the player's existence in the world.
///
/// # Usage
/// - **Input & Movement**: Systems query for this component to apply player input vectors.
/// - **Camera**: The camera system tracks the `Transform` of the entity with this component.
/// - **Save/Load**: The `IncludeInSave` requirement ensures the hero's state (and hierarchy) is persisted.
/// - **Equipment**: It serves as the root for the weapon hierarchy via `EquippedWeaponId`.
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

/// Defines the angular width of the attack area for melee weapons.
///
/// This component limits the hit detection of `MeleeWeapon` attacks to a specific cone
/// centered on the attack direction (Target - Attacker).
///
/// # Usage
/// - **Hit Detection**: The `hero_melee_attack_system` uses this to filter enemies.
///   An enemy is hit if `angle_to_enemy.abs() <= width / 2.0`.
///
/// # Units
/// - `width`: The total arc angle in **radians**.
///   (e.g., `PI` = 180 degrees semicircle, `PI/2` = 90 degrees cone).
#[derive(Component, Default)]
pub struct MeleeArc {
    /// Total angular width in radians.
    pub width: f32,
}

#[derive(Component, Default)]
pub struct Damage(pub f32);

#[derive(Component, Default)]
pub struct AttackRange(pub f32);

/// Governs the frequency of attacks for a weapon.
///
/// This component acts as a cooldown gate for the combat system.
///
/// # Usage
/// - **Combat Loop**: The `hero_attack_intent_system` ticks this timer and only generates
///   an `AttackIntent` event if the timer is finished.
/// - **Reset**: The timer is manually reset by the system *after* a successful intent is generated.
#[derive(Component, Default)]
pub struct AttackSpeed {
    /// The cooldown state. Its `duration` defines the minimum interval (seconds) between attacks.
    pub timer: Timer,
}

#[derive(Component, Default)]
pub struct Projectile;

/// Designates a target entity for a projectile to follow (homing behavior).
///
/// This component is attached to projectile entities to guide their movement.
///
/// # Usage
/// - **Movement**: `projectile_movement_system` queries this to adjust the projectile's
///   velocity towards the target's current position.
/// - **Collision**: `projectile_collision_system` uses this to check distance to the specific target
///   and trigger a hit if close enough.
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
