use bevy::prelude::*;

pub struct HeroEventsPlugin;

impl Plugin for HeroEventsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackIntent>()
            .register_type::<ProjectileHit>()
            .register_type::<MeleeHit>()
            .register_type::<EnemyKilled>();
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

#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct EnemyKilled {
    pub entity: Entity,
}

impl Default for EnemyKilled {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}

/// Represents a successful melee attack connection that hits one or more targets.
///
/// This is an **Observer** event (triggered via `commands.trigger`) that decouples the
/// attack logic (hit detection, arc calculation) from its effects (damage, visuals).
///
/// # Observers
/// - `apply_melee_damage_observer`: Reduces health of all entities in `targets`.
/// - `apply_hit_indicator_observer`: Spawns visual feedback on hit entities.
#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct MeleeHit {
    /// The weapon entity performing the attack
    pub attacker: Entity,
    /// List of enemies hit by the attack arc
    pub targets: Vec<Entity>,
    /// Raw damage amount to apply to each target
    pub damage: f32,
}

impl Default for MeleeHit {
    fn default() -> Self {
        Self {
            attacker: Entity::PLACEHOLDER,
            targets: Vec::new(),
            damage: 0.0,
        }
    }
}
