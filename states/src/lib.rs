use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Initializing,
    Running,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EnemyEncyclopediaState {
    #[default]
    Closed,
    Open,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingPhase {
    #[default]
    Assets,
    Research,
    Recipes,
    Unlocks,
    Done,
}
