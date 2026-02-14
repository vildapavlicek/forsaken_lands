use bevy::prelude::*;

/// The high-level state machine controlling the application's execution flow.
///
/// This state drives the primary loop of the game, gating which systems run
/// during initialization, loading, and active gameplay.
///
/// # Usage
/// - **Bootstrap**: The app starts in `Loading`, where `LoadingPhase` sub-states
///   manage asset loading and entity spawning.
/// - **Gameplay**: Systems tagged with `.run_if(in_state(GameState::Running))`
///   only execute when this state is active.
/// - **Transitions**: The `loading` crate manages the transition from `Loading`
///   to `Running` once all resources are ready.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    /// Bootstrapping phase.
    /// Handles asset loading, world spawning, and system initialization via `LoadingPhase`.
    #[default]
    Loading,
    /// Reserved for post-load initialization logic before the game loop starts.
    /// Currently unused/reserved for future expansion.
    Initializing,
    /// The active gameplay loop.
    /// Physics, input, and game logic systems run in this state.
    Running,
    /// Specialized state for deserializing save data.
    /// Currently unused/reserved (save loading happens within `LoadingPhase`).
    LoadingSave,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum VillageView {
    #[default]
    Closed,
    Menu,
    Crafting,
    Research,
    Encyclopedia,
    Heroes,
    Blessings,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EnemyEncyclopediaState {
    #[default]
    Closed,
    Open,
}

/// Represents the sequential initialization steps required to start the game.
///
/// The loading process acts as a state machine where each phase orchestrates a specific
/// part of the application setup, ensuring dependencies are met before proceeding.
///
/// # Sequence
/// 1. `Assets`: Load raw files (RON, GLB, etc.) from disk.
/// 2. `SpawnScene`: Deserializes the world state (either `startup.scn.ron` or a save file).
/// 3. `SpawnEntities`: Creates dynamic entities defined by assets (e.g., Research, Recipes).
/// 4. `CompileUnlocks`: Constructs the `LogicGate` graph from `UnlockDefinition` assets.
/// 5. `EvaluateUnlocks`: Triggers initial state checks to auto-unlock content satisfied by the loaded state.
/// 6. `PostLoadReconstruction`: Rebuilds complex relationships (e.g., linking EquippedWeapon handles) from save data.
/// 7. `Ready`: Finalizes loading and transitions to `GameState::Running`.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingPhase {
    /// Initial phase. Blocks until all `GameAssets` (spawn tables, prefabs) are fully loaded into memory.
    #[default]
    Assets,
    /// Spawns the static scene hierarchy.
    /// This initializes singleton entities like `Village` and `EnemyEncyclopedia`.
    SpawnScene,
    /// Spawns dynamic entities driven by asset definitions.
    /// Example: Iterates over `ResearchDefinition` assets to spawn `ResearchNode` entities.
    SpawnEntities,
    /// Constructs the reactive logic graph (LogicGates) from `UnlockDefinition` assets.
    /// During this phase, observers are registered but not yet triggered.
    CompileUnlocks,
    /// Triggers initial state checks (e.g., `ValueChanged`) to auto-unlock content
    /// that is already satisfied by the loaded save data or default values.
    EvaluateUnlocks,
    /// Reconstructs complex component relationships from save data.
    /// Example: Linking `EquippedWeaponId` (string) to the actual `Weapon` entity.
    PostLoadReconstruction,
    /// Initialization complete. The system will immediately transition to `GameState::Running`.
    Ready,
}
