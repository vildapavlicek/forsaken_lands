use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(SystemSet, ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSchedule {
    FrameStart,
    ResolveIntent,
    PerformAction,
    Effect,
    FrameEnd,
}
