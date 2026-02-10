use {
    crate::{Available, CraftingInProgress, Locked, RecipeNode},
    bevy::prelude::*,
    crafting_events::StartCraftingRequest,
    crafting_resources::RecipeMap,
    recipes_assets::{CONSTRUCTION_TOPIC_PREFIX, RecipeDefinition},
    unlocks_events::StatusCompleted,
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
        category: def.category,
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

    const RECIPE_REWARD_PREFIX: &str = "recipe:";

    let Some(entity) = event
        .reward_id
        .strip_prefix(RECIPE_REWARD_PREFIX)
        .and_then(|recipe_id| recipe_map.entities.get(recipe_id))
    else {
        trace!(%event.reward_id, "unlock achieved not related to recipe");
        return;
    };

    // Only transition if currently Locked
    if locked_query.get(*entity).is_ok() {
        commands
            .entity(*entity)
            .remove::<Locked>()
            .insert(Available);
        info!("Recipe '{}' is now available", event.reward_id);
    }
}

/// System that ticks crafting timers and processes completions.
pub fn update_crafting_progress(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CraftingInProgress)>,
) {
    for (entity, mut crafting) in query.iter_mut() {
        crafting.timer.tick(time.delta());

        if crafting.timer.is_finished() {
            info!(%crafting.recipe_id, ?crafting.category, "Crafting complete" );

            // Despawn the crafting entity
            commands.entity(entity).despawn();
            commands.trigger(StatusCompleted {
                topic: format!(
                    "{}{}",
                    crafting.category.as_topic_prefix(),
                    crafting.recipe_id
                ),
            });
        }
    }
}

/// Observer for StatusCompleted events.
/// Handles construction completion (marking building as constructed, removing recipe from UI).
pub fn on_construction_completed(
    trigger: On<StatusCompleted>,
    mut commands: Commands,
    recipe_map: Res<RecipeMap>,
    recipe_nodes: Query<&RecipeNode>,
    recipe_assets: Res<Assets<RecipeDefinition>>,
    mut constructed_buildings: ResMut<crafting_resources::ConstructedBuildings>,
) {
    let event = trigger.event();

    let Some(recipe_id) = event.topic.strip_prefix(CONSTRUCTION_TOPIC_PREFIX) else {
        return;
    };

    let Some(recipes_assets::RecipeCategory::Construction) = recipe_map
        .entities
        .get(recipe_id)
        .and_then(|&e| recipe_nodes.get(e).ok())
        .and_then(|node| recipe_assets.get(&node.handle))
        .map(|d| d.category)
    else {
        return;
    };

    info!(%recipe_id, "Construction complete");
    constructed_buildings.ids.insert(recipe_id.to_string());

    // Despawn the recipe entity to hide it from UI
    if let Some(&recipe_entity) = recipe_map.entities.get(recipe_id) {
        commands.entity(recipe_entity).despawn();
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
