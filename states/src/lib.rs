use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Initializing,
    Running,
    LoadingSave,
}

/// Substates for LoadingSave - each phase handles a reconstruction step
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::LoadingSave)]
pub enum LoadingSavePhase {
    #[default]
    WaitingForSceneSpawn,
    ReconstructingWeapons,
    RebuildingMaps,
    RelinkingResearch,
    ReconstructingRates,
    Complete,
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
    SpawnEntities,  // Spawn research & recipe entities
    CompileUnlocks, // Build unlock logic graphs
    SpawnScene,     // Spawn startup scene
    Ready,          // All done
}
