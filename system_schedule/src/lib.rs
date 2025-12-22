use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSchedule {
    FrameStart,
    ResolveIntent,
    PerformAction,
    Effect,
    FrameEnd,
}
