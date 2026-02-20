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
#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component)]
pub struct WeaponId(pub String);

/// Collection of tags assigned to a weapon (e.g., "melee", "bone_sword").
/// Used by the `bonus_stats` crate to apply conditional bonuses.
#[derive(Component, Reflect, Default, Clone, Debug, Deref)]
#[reflect(Component)]
pub struct WeaponTags(pub Vec<String>);

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

/// The base damage value of a weapon.
///
/// This component represents the raw power of a weapon before any modifiers (stats, critical hits)
/// are applied. It serves as the starting point for damage calculations in combat.
///
/// # Usage
/// - **Melee Combat**: The `hero_melee_attack_system` queries this component to determine the
///   base damage of a melee strike.
/// - **Ranged Combat**: The `hero_projectile_spawn_system` reads this component to initialize
///   the `ProjectileDamage` component on spawned projectiles.
///
/// # Units
/// - The value is in raw hit points.
#[derive(Component, Default)]
pub struct Damage(pub f32);

/// Defines the maximum effective distance (reach) of a weapon.
///
/// This component serves as the primary engagement gate for the combat AI and the
/// physical limit for melee hit detection.
///
/// # Usage
/// - **Decision Phase**: `hero_attack_intent_system` queries this to determine if any enemy
///   is close enough to initiate an attack (generating an `AttackIntent`).
/// - **Execution Phase (Melee)**: `hero_melee_attack_system` uses this to filter valid targets
///   within the weapon's arc and reach during a swing.
///
/// # Units
/// - Distance in **pixels**.
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

/// Defines the travel velocity of a projectile.
///
/// This component represents the scalar speed magnitude at which a projectile moves through the world.
///
/// # Usage
/// - **Movement**: The `projectile_movement_system` queries this component to calculate the
///   translation update: `velocity = direction * speed * delta_time`.
/// - **Creation**: Initialized on the projectile entity during spawning (e.g., via `ProjectileSpawnRequest`)
///   to set its constant speed.
///
/// # Units
/// - Speed in **logical pixels per second**.
#[derive(Component, Default)]
pub struct ProjectileSpeed(pub f32);

/// Stores the offensive capabilities of a projectile at the moment of its creation.
///
/// This component acts as a "snapshot" of the weapon's damage stats, ensuring that
/// the damage calculation reflects the state of the weapon *when fired*, even if
/// the projectile travels for some time before impact.
///
/// # Usage
/// - **Creation**: `hero_projectile_spawn_system` initializes this component using
///   values from the `Damage` and `WeaponTags` of the source weapon.
/// - **Impact**: `projectile_collision_system` reads this data to populate the
///   `DamageRequest` event when the projectile hits a valid target.
#[derive(Component, Default)]
pub struct ProjectileDamage {
    /// The raw damage value (Hit Points) before bonuses are applied on hit.
    pub base_damage: f32,
    /// Tags describing the damage source (e.g., "arrow", "fire"), used by `bonus_stats`
    /// to apply conditional modifiers.
    pub source_tags: Vec<String>,
}
