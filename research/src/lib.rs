use {
    bevy::{platform::collections::HashMap, prelude::*},
    system_schedule::GameSchedule,
};

pub mod systems;

// --- Static Data (Loaded from startup.scn.ron) ---

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct ResearchLibrary {
    /// Key: Research ID (e.g., "wooden_bow")
    pub available: HashMap<String, ResearchDefinition>,
}

#[derive(Reflect, Debug, Clone)]
pub struct ResearchDefinition {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub prerequisites: Vec<String>,
    pub cost: HashMap<String, u32>, // Matches Wallet resource keys
    pub time_required: f32,         // In seconds
    pub unlocks: Vec<UnlockEffect>,
}

#[derive(Reflect, Debug, Clone)]
pub enum UnlockEffect {
    Feature(String), // Unlocks a game mechanic (e.g., "auto_mining")
    StatBuff { stat: String, value: f32 },
}

// --- Runtime State ---

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct ResearchState {
    /// ID -> Timer (counts up/down to completion)
    pub in_progress: HashMap<String, Timer>,
    /// List of IDs that are fully researched
    pub completed: Vec<String>,
}

impl ResearchState {
    pub fn is_researched(&self, id: &str) -> bool {
        self.completed.contains(&id.to_string())
    }

    pub fn is_researching(&self, id: &str) -> bool {
        self.in_progress.contains_key(id)
    }
}

// Define an event for UI interactions
#[derive(Message)]
pub struct StartResearchRequest(pub String);

pub struct ResearchPlugin;

impl Plugin for ResearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StartResearchRequest>();

        app.register_type::<ResearchLibrary>()
            .register_type::<ResearchDefinition>()
            .register_type::<UnlockEffect>()
            .register_type::<ResearchState>()
            .init_resource::<ResearchLibrary>()
            .init_resource::<ResearchState>()
            .add_systems(
                Update,
                (
                    systems::start_research.in_set(GameSchedule::Effect),
                    systems::update_research_progress.in_set(GameSchedule::FrameStart),
                )
                    .run_if(in_state(states::GameState::Running)),
            );
    }
}
