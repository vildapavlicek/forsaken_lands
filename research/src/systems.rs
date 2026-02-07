use {
    crate::{
        Available, Completed, InProgress, Locked, ResearchCompleted, ResearchCompletionCount,
        ResearchDefinition, ResearchMap, ResearchNode, StartResearchRequest,
    },
    bevy::prelude::*,
    unlocks_events,
    unlocks_resources::UnlockState,
    wallet::Wallet,
};

// TODO: Move this to a loading stage once asset loading is consolidated
/// Spawns entities for newly loaded ResearchDefinition assets
pub fn spawn_research_entities(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut assets: ResMut<Assets<ResearchDefinition>>,
    unlock_state: Res<UnlockState>,
) {
    debug!("Spawning research entities...");

    // Collect IDs first to avoid borrowing assets immutably while needing mutable access later
    let ids: Vec<_> = assets.ids().collect();

    for id in ids {
        // Get definition info first and clone what we need
        // We scope this so we drop the immutable borrow before get_strong_handle
        let def_id = {
            let Some(def) = assets.get(id) else {
                continue;
            };

            // Check if already spawned
            if research_map.entities.contains_key(&def.id) {
                continue;
            }

            def.id.clone()
        };

        // Check if the unlock for this research has already been achieved
        // Unlocks use reward_id format: "research_{id}"
        let already_unlocked = unlock_state.completed.iter().any(|unlock_id| {
            unlock_id.ends_with(&format!("{}_unlock", def_id))
                || unlock_id.starts_with(&format!("research_{}", def_id))
        });

        // Now we can mutably borrow assets to get the strong handle
        let handle = assets.get_strong_handle(id).unwrap();

        let entity = if already_unlocked {
            debug!(
                "Research '{}' unlock already achieved, spawning as Available",
                def_id
            );
            commands
                .spawn((
                    ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    Available,
                    ResearchCompletionCount(0),
                ))
                .id()
        } else {
            commands
                .spawn((
                    ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    Locked,
                    ResearchCompletionCount(0),
                ))
                .id()
        };

        research_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned research entity: {} -> {:?}", def_id, entity);
    }
}

/// Listens for UnlockAchieved events with `research:` prefix
pub fn on_unlock_achieved(
    trigger: On<unlocks_events::UnlockAchieved>,
    mut commands: Commands,
    research_map: Res<ResearchMap>,
    locked_query: Query<(), With<Locked>>,
) {
    let event = trigger.event();
    const RESEARCH_REWARD_PREFIX: &str = "research:";

    let Some(entity) = event
        .reward_id
        .strip_prefix(RESEARCH_REWARD_PREFIX)
        .and_then(|research_id| research_map.entities.get(research_id)).copied()
    else {
        return;
    };

    if locked_query.get(entity).is_ok() {
        commands.entity(entity).remove::<Locked>().insert(Available);
        info!("Research '{}' is now available", event.reward_id);
    }
}

/// Ticks timers for in-progress research and handles completion/repeat logic.
pub fn update_research_progress(
    time: Res<Time>,
    assets: Res<Assets<ResearchDefinition>>,
    mut commands: Commands,
    mut research_state: ResMut<crate::ResearchState>,
    mut query: Query<(
        Entity,
        &ResearchNode,
        &mut InProgress,
        &mut ResearchCompletionCount,
    )>,
) {
    for (entity, node, mut progress, mut count) in query.iter_mut() {
        progress.timer.tick(time.delta());
        if progress.timer.just_finished() {
            // Increment completion count (on entity and in persisted state)
            count.0 += 1;
            let current_count = count.0;
            research_state
                .completion_counts
                .insert(node.id.clone(), current_count);

            // Always trigger completion event (for effects/bonuses)
            commands.trigger(ResearchCompleted {
                research_id: node.id.clone(),
            });

            // Notify unlock system about research completion
            commands.trigger(unlocks_events::StatusCompleted {
                topic: format!("research:{}", node.id),
            });

            // Check max_repeats from definition
            let max_repeats = assets
                .get(&node.handle)
                .map(|def| def.max_repeats)
                .unwrap_or(1);

            if current_count >= max_repeats {
                // Fully completed - no more repeats
                commands
                    .entity(entity)
                    .remove::<InProgress>()
                    .insert(Completed);
                info!(
                    "Research fully completed: {} ({}/{})",
                    node.id, current_count, max_repeats
                );
            } else {
                // More repeats available - back to Available
                commands
                    .entity(entity)
                    .remove::<InProgress>()
                    .insert(Available);
                info!(
                    "Research completed iteration: {} ({}/{})",
                    node.id, current_count, max_repeats
                );
            }
        }
    }
}

/// Starts a research (validates Available + cost)
pub fn start_research(
    trigger: On<StartResearchRequest>,
    research_map: Res<ResearchMap>,
    assets: Res<Assets<ResearchDefinition>>,
    query: Query<&ResearchNode, With<Available>>,
    mut wallet: ResMut<Wallet>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let Some(&entity) = research_map.entities.get(&event.0) else {
        warn!("Research '{}' not found", event.0);
        return;
    };

    let Ok(node) = query.get(entity) else {
        warn!("Research '{}' not available", event.0);
        return;
    };

    let Some(def) = assets.get(&node.handle) else {
        warn!("Research definition not loaded for '{}'", event.0);
        return;
    };

    // Deduct cost
    for (res, amt) in &def.cost {
        if let Some(resource_amt) = wallet.resources.get_mut(res) {
            *resource_amt -= amt;
        }
    }

    commands
        .entity(entity)
        .remove::<Available>()
        .insert(InProgress {
            research_id: node.id.clone(),
            timer: Timer::from_seconds(def.time_required, TimerMode::Once),
        });
    info!("Started researching: {}", def.name);
}

pub fn clean_up_research(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    nodes: Query<Entity, With<ResearchNode>>,
) {
    debug!("Cleaning up all research entities");
    // Despawn all research nodes
    for entity in nodes.iter() {
        commands.entity(entity).despawn();
    }
    // Clear the map
    research_map.entities.clear();
}
