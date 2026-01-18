use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Initializing,
    Running,
    LoadingSave,
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
    SpawnScene,             // Spawn startup/save scene first (loads resources like EnemyEncyclopedia)
    SpawnEntities,          // Spawn research & recipe entities
    CompileUnlocks,         // Build unlock logic graphs (observers created)
    EvaluateUnlocks,        // Fire events for satisfied unlocks (all observers now exist)
    PostLoadReconstruction, // Save-specific reconstruction
    Ready,                  // All done
}
