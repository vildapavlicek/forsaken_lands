//! Shared unlock state components for entities that can be locked/unlocked.
//!
//! These components are used by both research and recipe systems to track
//! whether an item is available to the player.

use bevy::prelude::*;

pub struct UnlockStatesPlugin;

impl Plugin for UnlockStatesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Locked>()
            .register_type::<Available>();
    }
}

/// Default state - not visible in UI, waiting for unlock condition to be met.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Locked;

/// Visible in UI, available for use by the player.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Available;
