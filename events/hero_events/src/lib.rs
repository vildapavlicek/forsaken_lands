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

#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct AttackIntent {
    pub attacker: Entity,
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

#[derive(Event, Reflect)]
#[reflect(Default)]
pub struct ProjectileHit {
    pub projectile: Entity,
    pub target: Entity,
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
