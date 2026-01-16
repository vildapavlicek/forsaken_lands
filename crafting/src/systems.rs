use {
    crate::{Available, CraftingInProgress, Locked, RecipeNode},
    bevy::prelude::*,
    crafting_events::StartCraftingRequest,
    crafting_resources::RecipeMap,
    divinity_events::IncreaseDivinity,
    recipes_assets::RecipeDefinition,
    village_components::Village,
};

/// Observer that handles StartCraftingRequest events.
/// Spawns a CraftingInProgress entity with a timer.
/// Note: Validation and resource deduction already handled by UI.
pub fn start_crafting(
    trigger: On<StartCraftingRequest>,
    mut commands: Commands,
    recipe_map: Res<RecipeMap>,
    recipe_query: Query<&RecipeNode, With<Available>>,
    assets: Res<Assets<RecipeDefinition>>,
) {
    let recipe_id = &trigger.event().recipe_id;

    // Look up the recipe entity
    let Some(&entity) = recipe_map.entities.get(recipe_id) else {
        warn!("Recipe '{}' not found in RecipeMap", recipe_id);
        return;
    };

    // Verify it's available
    let Ok(node) = recipe_query.get(entity) else {
        warn!("Recipe '{}' not available", recipe_id);
        return;
    };

    // Get the definition
    let Some(def) = assets.get(&node.handle) else {
        warn!("Recipe definition not loaded for '{}'", recipe_id);
        return;
    };

    // Spawn crafting entity with timer
    commands.spawn(CraftingInProgress {
        recipe_id: recipe_id.clone(),
        outcomes: def.outcomes.clone(),
        timer: Timer::from_seconds(def.craft_time, TimerMode::Once),
    });

    info!("Crafting started for: {}", def.display_name);
}

/// Observer for UnlockAchieved events with recipe_ prefix.
/// Transitions recipe entities from Locked → Available.
pub fn on_recipe_unlock_achieved(
    trigger: On<unlocks_events::UnlockAchieved>,
    mut commands: Commands,
    recipe_map: Res<RecipeMap>,
    locked_query: Query<(), With<Locked>>,
) {
    let event = trigger.event();
    const PREFIX: &str = "recipe_";

    if event.reward_id.starts_with(PREFIX) {
        let recipe_id = &event.reward_id[PREFIX.len()..];
        if let Some(&entity) = recipe_map.entities.get(recipe_id) {
            // Only transition if currently Locked
            if locked_query.get(entity).is_ok() {
                commands.entity(entity).remove::<Locked>().insert(Available);
                info!("Recipe '{}' is now available", recipe_id);
            }
        } else {
            debug!(
                "Recipe '{}' not found in RecipeMap (unlock may have fired before asset load)",
                recipe_id
            );
        }
    }
}

/// System that ticks crafting timers and processes completions.
pub fn update_crafting_progress(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
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
                        scene_spawner.spawn_dynamic(scene_handle);
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
