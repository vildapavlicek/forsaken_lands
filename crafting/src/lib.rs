use {
    bevy::prelude::*,
    crafting_resources::CraftingOutcome,
    recipes_assets::RecipeDefinition,
    shared_components::IncludeInSave,
    states::GameState,
    system_schedule::GameSchedule,
};

pub mod systems;

// Re-export for convenience
pub use {
    crafting_components::RecipeNode,
    unlock_states::{Available, Locked},
};

/// Component to track an in-progress crafting operation.
/// Derives Reflect for save file inclusion.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(IncludeInSave)]
pub struct CraftingInProgress {
    pub recipe_id: String,
    pub outcomes: Vec<CraftingOutcome>,
    pub timer: Timer,
}

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(recipes_assets::RecipesAssetsPlugin)
            .register_type::<CraftingInProgress>()
            .add_observer(systems::start_crafting)
            .add_observer(systems::on_recipe_unlock_achieved)
            .add_systems(
                Update,
                systems::update_crafting_progress
                    .in_set(GameSchedule::FrameStart)
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(
                OnExit(states::GameState::Running),
                systems::clean_up_crafting,
            );
    }
}

/// Spawns recipe entities from loaded RecipeDefinition assets.
/// Called by LoadingManager during asset loading phase.
pub fn spawn_recipe_entities(
    commands: &mut Commands,
    recipe_map: &mut crafting_resources::RecipeMap,
    assets: &mut Assets<RecipeDefinition>,
    unlock_state: &unlocks_resources::UnlockState,
) {
    debug!("Spawning recipe entities...");

    let ids: Vec<_> = assets.ids().collect();

    for id in ids {
        let def_id = {
            let Some(def) = assets.get(id) else {
                continue;
            };

            // Check if already spawned
            if recipe_map.entities.contains_key(&def.id) {
                continue;
            }

            def.id.clone()
        };

        // Check if the unlock for this recipe has already been achieved
        let already_unlocked = unlock_state.completed.iter().any(|unlock_id| {
            unlock_id == &format!("recipe_{}_unlock", def_id)
                || unlock_id.starts_with(&format!("recipe_{}", def_id))
        });

        let handle = assets.get_strong_handle(id).unwrap();

        let entity = if already_unlocked {
            debug!(
                "Recipe '{}' unlock already achieved, spawning as Available",
                def_id
            );
            commands
                .spawn((
                    RecipeNode {
                        id: def_id.clone(),
                        handle,
                    },
                    Available,
                ))
                .id()
        } else {
            commands
                .spawn((
                    RecipeNode {
                        id: def_id.clone(),
                        handle,
                    },
                    Locked,
                ))
                .id()
        };

        recipe_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned recipe entity: {} -> {:?}", def_id, entity);
    }
}
