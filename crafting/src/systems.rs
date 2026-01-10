use {
    crate::CraftingInProgress, bevy::prelude::*, crafting_events::StartCraftingRequest,
    crafting_resources::RecipesLibrary, divinity_events::IncreaseDivinity,
    village_components::Village,
};

/// Observer that handles StartCraftingRequest events.
/// Spawns a CraftingInProgress entity with a timer.
/// Note: Validation and resource deduction already handled by UI.
pub fn start_crafting(
    trigger: On<StartCraftingRequest>,
    mut commands: Commands,
    library: Res<RecipesLibrary>,
) {
    let recipe_id = &trigger.event().recipe_id;

    let Some(recipe) = library.recipes.get(recipe_id) else {
        warn!("Recipe ID {} not found.", recipe_id);
        return;
    };

    // Spawn crafting entity with timer
    commands.spawn(CraftingInProgress {
        recipe_id: recipe_id.clone(),
        outcomes: recipe.outcomes.clone(),
        timer: Timer::from_seconds(recipe.craft_time, TimerMode::Once),
    });

    info!("Crafting started for: {}", recipe.display_name);
}

/// System that ticks crafting timers and processes completions.
pub fn update_crafting_progress(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut CraftingInProgress)>,
    village_query: Query<Entity, With<Village>>,
) {
    for (entity, mut crafting) in query.iter_mut() {
        crafting.timer.tick(time.delta());

        if crafting.timer.is_finished() {
            info!("Crafting complete for: {}", crafting.recipe_id);

            // Process all outcomes
            for outcome in &crafting.outcomes {
                match outcome {
                    crafting_resources::CraftingOutcome::SpawnPrefab(path) => {
                        let scene_handle: Handle<DynamicScene> = asset_server.load(path);
                        commands.spawn(DynamicSceneRoot(scene_handle));
                        info!("Spawned prefab: {}", path);
                    }
                    crafting_resources::CraftingOutcome::AddResource { id, amount } => {
                        // TODO: Add to wallet
                        info!("Would add {} x {} to wallet", amount, id);
                    }
                    crafting_resources::CraftingOutcome::UnlockFeature(feature) => {
                        info!("Would unlock feature: {}", feature);
                    }
                    crafting_resources::CraftingOutcome::GrantXp(xp) => {
                        info!("Would grant {} XP", xp);
                    }
                    crafting_resources::CraftingOutcome::IncreaseDivinity(amount) => {
                        if let Ok(village_entity) = village_query.single() {
                            commands.trigger(IncreaseDivinity {
                                entity: village_entity,
                                xp_amount: *amount as f32,
                            });
                            info!("Triggered IncreaseDivinity with {} XP for Village", amount);
                        } else {
                            warn!("Could not find Village entity to increase divinity");
                        }
                    }
                }
            }

            // Despawn the crafting entity
            commands.entity(entity).despawn();
        }
    }
}
