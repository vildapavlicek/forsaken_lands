use {
    crate::{Available, CraftingInProgress, Locked, RecipeNode},
    bevy::prelude::*,
    crafting_events::StartCraftingRequest,
    crafting_resources::RecipeMap,
    recipes_assets::RecipeDefinition,
    unlocks_events::StatusCompleted,
    village_components::Village,
    weapon_factory_events,
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
    mut query: Query<(Entity, &mut CraftingInProgress)>,
    recipe_map: Res<RecipeMap>,
    recipe_nodes: Query<&RecipeNode>,
    recipe_assets: Res<Assets<RecipeDefinition>>,
) {
    for (entity, mut crafting) in query.iter_mut() {
        crafting.timer.tick(time.delta());

        if crafting.timer.is_finished() {
            info!("Crafting complete for: {}", crafting.recipe_id);

            // Check if this is a weapon and spawn it via factory
            if recipe_map
                .entities
                .get(&crafting.recipe_id)
                .and_then(|&e| recipe_nodes.get(e).ok())
                .and_then(|node| recipe_assets.get(&node.handle))
                .is_some_and(|def| matches!(def.category, recipes_assets::RecipeCategory::Weapons))
            {
                commands.trigger(weapon_factory_events::SpawnWeaponRequest {
                    weapon_id: crafting.recipe_id.clone(),
                    parent: None,
                    add_to_inventory: true,
                });
                info!("Triggered SpawnWeaponRequest for '{}'", crafting.recipe_id);
            }

            // Process all outcomes
            for outcome in &crafting.outcomes {
                match outcome {
                    crafting_resources::CraftingOutcome::AddResource { id, amount } => {
                        // TODO: Add to wallet
                        info!("Would add {} x {} to wallet", amount, id);
                    }
                    crafting_resources::CraftingOutcome::UnlockFeature(feature) => {
                        info!("Would unlock feature: {}", feature);
                    }
                }
            }

            // Despawn the crafting entity
            commands.entity(entity).despawn();
            commands.trigger(StatusCompleted {
                topic: format!("craft:{}", crafting.recipe_id),
            });
        }
    }
}

/// Observer for StatusCompleted events.
/// Handles construction completion (marking building as constructed, removing recipe from UI).
pub fn on_crafting_completed(
    trigger: On<StatusCompleted>,
    mut commands: Commands,
    recipe_map: Res<RecipeMap>,
    recipe_nodes: Query<&RecipeNode>,
    recipe_assets: Res<Assets<RecipeDefinition>>,
    mut constructed_buildings: ResMut<crafting_resources::ConstructedBuildings>,
) {
    let event = trigger.event();
    const PREFIX: &str = "craft:";

    if event.topic.starts_with(PREFIX) {
        let recipe_id = &event.topic[PREFIX.len()..];

        // Check if this is a construction recipe
        if recipe_map
            .entities
            .get(recipe_id)
            .and_then(|&e| recipe_nodes.get(e).ok())
            .and_then(|node| recipe_assets.get(&node.handle))
            .is_some_and(|def| matches!(def.category, recipes_assets::RecipeCategory::Construction))
        {
            info!("Construction complete: {}", recipe_id);
            constructed_buildings.ids.insert(recipe_id.to_string());

            // Despawn the recipe entity to hide it from UI
            if let Some(&recipe_entity) = recipe_map.entities.get(recipe_id) {
                commands.entity(recipe_entity).despawn();
                // We can't remove from recipe_map here as we don't have mutable access
                // But since we check recipe_nodes.get(e) in UI and systems,
                // despawning the entity effectively removes it.
            }
        }
    }
}

pub fn clean_up_crafting(
    mut commands: Commands,
    mut recipe_map: ResMut<RecipeMap>,
    recipes: Query<Entity, With<RecipeNode>>,
    in_progress: Query<Entity, With<CraftingInProgress>>,
) {
    debug!("Cleaning up crafting system");
    // Despawn recipe entities
    for entity in recipes.iter() {
        // Use despawn_recursive to be safe, though they might not have children
        commands.entity(entity).despawn();
    }
    // Despawn in-progress crafting
    for entity in in_progress.iter() {
        commands.entity(entity).despawn();
    }

    recipe_map.entities.clear();
}
