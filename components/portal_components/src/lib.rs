use {bevy::prelude::*, shared_components::IncludeInSave};

/// The primary marker component for a portal entity that spawns enemies.
///
/// This component anchors the enemy spawning logic, serving as the root for
/// wave management and difficulty scaling.
///
/// # Usage
/// - **Spawning**: The `enemy_spawn_system` queries entities with this component to
///   tick their `SpawnTimer` and generate enemies from the assigned `SpawnTableId`.
/// - **Progression**: The attached `CurrentDivinity` component (level/tier) is used to
///   filter spawn table entries, scaling difficulty as the portal levels up.
/// - **Interaction**: The `Pickable` requirement enables player interaction (e.g., selecting
///   the portal to view stats or upgrade).
/// - **Persistence**: The `IncludeInSave` requirement ensures the portal's state
///   is preserved across sessions.
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
#[require(Pickable, IncludeInSave)]
pub struct Portal;

/// Controls the frequency of enemy generation from a Portal.
///
/// This component acts as the "metronome" for the spawning loop, governing how often
/// the system attempts to spawn enemies based on the associated `SpawnTableId`.
///
/// # Usage
/// - **Game Loop**: The `enemy_spawn_system` (in `PortalsPlugin`) ticks this timer every frame.
///   When finished, it triggers a spawn attempt and resets.
/// - **Modifiers**: The effective speed of this timer can be altered by gameplay effects,
///   such as `BlessingEffect::DecreaseSpawnTimer`, which are applied dynamically in the system.
/// - **Dependencies**: It functions in tandem with `CurrentDivinity` (to filter spawn tables)
///   and `SpawnTableId` (to define *what* spawns).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SpawnTimer(pub Timer);

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(10.0, TimerMode::Repeating))
    }
}

/// Links a Portal to a specific SpawnTable asset (e.g., "default")
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct SpawnTableId(pub String);
