use {bevy::prelude::*, shared_components::IncludeInSave};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(IncludeInSave)]
pub struct Enemy;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Dead;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Lifetime(pub Timer);

/// Represents the vitality and durability of an entity (e.g., Enemy, Boss).
///
/// This component is the primary state checked by combat and lifecycle systems to determine
/// if an entity should remain in the world or be destroyed.
///
/// # Usage
/// - **Combat Systems**: `apply_damage_system` and `apply_melee_damage_observer` reduce `current` health.
/// - **Lifecycle System**: `manage_enemy_lifecycle` checks if `current <= 0.0` to trigger death logic.
/// - **UI**: Used to display health bars (ratio of current / max).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health {
    /// The current health points of the entity. Death occurs when this reaches <= 0.0.
    pub current: f32,
    /// The maximum possible health points. Used for clamping healing and calculating health percentages.
    pub max: f32,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Drops(pub Vec<Drop>);

#[derive(Reflect, Default, Debug, Clone)]
pub struct Drop {
    pub id: String,
    pub value: u32,
    /// Drop chance from 0.0 to 1.0 (1.0 = 100% guaranteed drop)
    pub chance: f32,
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component, Default)]
pub struct MonsterId(pub String);

/// Defines which distance range section an enemy belongs to.
/// The game area spans from Portal (y=300) to Village (y=-300).
#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, Default)]
pub enum EnemyRange {
    /// Near the village (y: -250 to -150)
    #[default]
    CloseRange,
    /// Middle ground (y: -150 to 50)
    MediumRange,
    /// Near the portal (y: 50 to 275)
    LongRange,
}

impl EnemyRange {
    /// Returns the (min_y, max_y) bounds for this range section.
    pub fn y_bounds(&self) -> (f32, f32) {
        match self {
            EnemyRange::CloseRange => (-250.0, -150.0),
            EnemyRange::MediumRange => (-150.0, 50.0),
            EnemyRange::LongRange => (50.0, 275.0),
        }
    }
}

pub const MELEE_ENGAGEMENT_RADIUS: f32 = 150.0;

/// The target destination an enemy is moving towards.
/// Generated randomly within the enemy's range section on spawn.
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component, Default)]
pub struct TargetDestination(pub Vec2);
