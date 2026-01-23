use bevy::prelude::*;

pub struct BuildingsComponentsPlugin;

impl Plugin for BuildingsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TheMaw>()
            .register_type::<EntropyGenerator>();
    }
}

/// Tag component for 'The Maw' building entity.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct TheMaw;

/// Component for entities that generate entropy over time.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct EntropyGenerator {
    pub timer: Timer,
}

impl Default for EntropyGenerator {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}
