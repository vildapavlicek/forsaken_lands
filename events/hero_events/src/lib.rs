use bevy::prelude::*;

pub struct HeroEventsPlugin;

impl Plugin for HeroEventsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackIntent>()
            .register_type::<ProjectileHit>()
            .register_type::<EnemyKilled>()
            .register_type::<DamageRequest>();
    }
}

/// Represents an entity's decision and readiness to attack a specific target.
///
/// This **Observer** event acts as the bridge between the "Decision Phase" (AI, Input, Cooldowns)
/// and the "Execution Phase" (Weapon-specific logic). It allows the system to declare *that*
/// an attack should happen, without knowing *how* (Melee vs Ranged) it will be performed.
///
/// # Observers
/// - `hero_projectile_spawn_system`: Spawns a projectile if `attacker` is a `RangedWeapon`.
/// - `hero_melee_attack_system`: Calculates damage arcs if `attacker` is a `MeleeWeapon`.
#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct AttackIntent {
    /// The entity (typically a weapon) attempting to attack.
    pub attacker: Entity,
    /// The intended victim of the attack.
    pub target: Entity,
}

impl Default for AttackIntent {
    fn default() -> Self {
        Self {
            attacker: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
        }
    }
}

/// Represents a successful collision between a projectile and a target.
///
/// This **Observer** event (triggered via `commands.trigger`) decouples the physics/collision
/// detection from the gameplay effects (damage application).
///
/// # Observers
/// - `apply_damage_system`: Reduces health of the `target`.
#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct ProjectileHit {
    /// The projectile entity that made contact.
    /// Note: This entity is typically despawned immediately after this event is triggered.
    pub projectile: Entity,
    /// The entity that was hit.
    pub target: Entity,
    /// The raw damage amount to apply to the target.
    pub damage: f32,
}

impl Default for ProjectileHit {
    fn default() -> Self {
        Self {
            projectile: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
            damage: 0.0,
        }
    }
}

/// Represents the confirmed death of an enemy entity.
///
/// This **Observer** event (triggered via `commands.trigger`) serves as the primary signal
/// for progression systems to process rewards, statistics, and quest updates.
///
/// # Observers
/// - `update_encyclopedia` (Village): Increments kill counts in `EnemyEncyclopedia` and triggers kill-based unlocks.
/// - `process_enemy_killed_rewards` (Wallet): Rolls for loot based on `Drops` and adds resources to `Wallet`.
#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct EnemyKilled {
    /// The enemy entity that was killed.
    /// Note: This entity is guaranteed to be alive with its components (e.g., `Drops`, `MonsterId`) during the event trigger.
    pub entity: Entity,
}

impl Default for EnemyKilled {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}

/// Represents a request to apply damage to a single target.
/// Triggered once per target hit by any damage source.
#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct DamageRequest {
    /// The weapon/source entity performing the attack
    pub source: Entity,
    /// The target entity receiving damage
    pub target: Entity,
    /// Base damage before any modifiers
    pub base_damage: f32,
    /// Tags from the damage source (e.g., ["damage:melee", "damage:bone"])
    pub source_tags: Vec<String>,
}

impl Default for DamageRequest {
    fn default() -> Self {
        Self {
            source: Entity::PLACEHOLDER,
            target: Entity::PLACEHOLDER,
            base_damage: 0.0,
            source_tags: Vec::new(),
        }
    }
}

/// Represents a request to spawn a projectile.
#[derive(Event)]
pub struct ProjectileSpawnRequest {
    /// The entity or position the projectile originates from.
    pub source_position: Vec3,
    /// The target entity the projectile should home in on.
    pub target: Entity,
    /// The speed of the projectile.
    pub speed: f32,
    /// The base damage of the projectile.
    pub base_damage: f32,
    /// Tags describing the damage source.
    pub source_tags: Vec<String>,
}
