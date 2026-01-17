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
    SpawnEntities,          // Spawn research & recipe entities
    CompileUnlocks,         // Build unlock logic graphs
    SpawnScene,             // Spawn startup scene
    PostLoadReconstruction, // Save-specific reconstruction
    Ready,                  // All done
}
