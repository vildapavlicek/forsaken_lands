use {
    bevy::{platform::collections::HashMap, prelude::*},
    bevy_common_assets::ron::RonAssetPlugin,
    serde::Deserialize,
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

// --- Events ---

#[derive(Event)]
pub struct ResearchCompleted {
    pub research_id: String,
}

#[derive(Event)]
pub struct StartResearchRequest(pub String);

// --- Plugin ---

pub struct ResearchPlugin;

impl Plugin for ResearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ResearchDefinition>::new(&["research.ron"]))
            .init_resource::<ResearchMap>()
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
            .add_systems(OnExit(states::GameState::Running), systems::clean_up_research);
    }
}
