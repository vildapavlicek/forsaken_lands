use {
    bevy::{platform::collections::HashMap, prelude::*},
    bevy_common_assets::ron::RonAssetPlugin,
    serde::Deserialize,
    system_schedule::GameSchedule,
};

pub mod systems;

// Re-export shared unlock states for backwards compatibility
pub use unlock_states::{Available, Locked};

// --- Asset Definition ---

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct ResearchDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: HashMap<String, u32>,
    pub time_required: f32,
}

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

/// Currently being researched
#[derive(Component)]
pub struct InProgress {
    pub timer: Timer,
}

/// Research completed
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
            .add_systems(
                Update,
                systems::update_research_progress
                    .in_set(GameSchedule::FrameStart)
                    .run_if(in_state(states::GameState::Running)),
            )
            .add_observer(systems::on_unlock_achieved)
            .add_observer(systems::start_research);
    }
}
