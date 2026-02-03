use {
    bevy::{platform::collections::HashMap, prelude::*},
    bevy_common_assets::ron::RonAssetPlugin,
    serde::Deserialize,
    shared_components::IncludeInSave,
    system_schedule::GameSchedule,
};

pub mod systems;

use research_assets::ResearchDefinition;
// Re-export shared unlock states for backwards compatibility
pub use unlock_states::{Available, Locked};

#[derive(Reflect, Debug, Clone, Deserialize)]
pub enum UnlockEffect {
    Feature(String),
    StatBuff { stat: String, value: f32 },
}

// --- Components ---

/// Associates an entity with a research definition
#[derive(Component)]
pub struct ResearchNode {
    pub id: String,
    pub handle: Handle<ResearchDefinition>,
}

/// Tracks how many times a repeatable research has been completed.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ResearchCompletionCount(pub u32);

/// Currently being researched
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(IncludeInSave)]
pub struct InProgress {
    pub research_id: String,
    pub timer: Timer,
}

/// Research completed (all repeats exhausted)
#[derive(Component)]
pub struct Completed;

// --- Resources ---

/// O(1) lookup of research entities by ID
#[derive(Resource, Default)]
pub struct ResearchMap {
    pub entities: HashMap<String, Entity>,
}

/// Persisted research state - tracks how many times each research was completed.
/// This is saved/loaded and used to reconstruct research entity state on load.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ResearchState {
    pub completion_counts: HashMap<String, u32>,
}

// --- Events ---

/// Represents the successful completion of a research project's timer.
///
/// This **Observer** event signals that the time requirement has been met.
/// It is triggered via `commands.trigger` for immediate handling.
///
/// # Observers
/// - `ui/notification_ui`: Queues a visual notification (toast) for the player.
///
/// # Related Events
/// - `unlocks_events::StatusCompleted`: Triggered simultaneously to update the Unlock System (logic).
///   This event (`ResearchCompleted`) focuses on immediate feedback/effects.
#[derive(Event)]
pub struct ResearchCompleted {
    pub research_id: String,
}

/// Represents a request to begin a research project.
///
/// This **Observer** event (triggered via `commands.trigger`) serves as the bridge between
/// user input (UI) and the research logic system.
///
/// # Observers
/// - `start_research`: Validates that the requested research is `Available`, ensures the
///   player can afford the resource cost (in `Wallet`), deducts the cost, and transitions
///   the entity state to `InProgress`.
#[derive(Event)]
pub struct StartResearchRequest(
    /// The unique identifier of the research definition (matches `ResearchDefinition.id`).
    pub String,
);

// --- Plugin ---

pub struct ResearchPlugin;

impl Plugin for ResearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ResearchDefinition>::new(&["research.ron"]))
            .init_resource::<ResearchMap>()
            .init_resource::<ResearchState>()
            .register_type::<ResearchState>()
            .register_type::<UnlockEffect>()
            .register_type::<ResearchCompletionCount>()
            .register_type::<InProgress>()
            .add_systems(
                Update,
                systems::update_research_progress
                    .in_set(GameSchedule::FrameStart)
                    .run_if(in_state(states::GameState::Running)),
            )
            .add_observer(systems::on_unlock_achieved)
            .add_observer(systems::start_research)
            .add_systems(
                OnExit(states::GameState::Running),
                systems::clean_up_research,
            );
    }
}
