use {
    bevy::prelude::*,
    crafting_resources::CraftingOutcome,
    states::GameState,
    system_schedule::GameSchedule,
};

pub mod systems;

/// Component to track an in-progress crafting operation.
/// Derives Reflect for save file inclusion.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CraftingInProgress {
    pub recipe_id: String,
    pub outcomes: Vec<CraftingOutcome>,
    pub timer: Timer,
}

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CraftingInProgress>()
            .add_observer(systems::start_crafting)
            .add_systems(
                Update,
                systems::update_crafting_progress
                    .in_set(GameSchedule::FrameStart)
                    .run_if(in_state(GameState::Running)),
            );
    }
}
