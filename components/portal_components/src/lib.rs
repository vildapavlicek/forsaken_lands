use {bevy::prelude::*, shared_components::IncludeInSave};

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
#[require(Pickable, IncludeInSave)]
pub struct Portal;

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
