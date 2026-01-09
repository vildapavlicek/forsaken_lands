use {
    crate::{
        Available, Completed, InProgress, Locked, ResearchCompleted, ResearchDefinition,
        ResearchMap, ResearchNode, StartResearchRequest,
    },
    bevy::prelude::*,
    wallet::Wallet,
};

// TODO: Move this to a loading stage once asset loading is consolidated
/// Spawns entities for newly loaded ResearchDefinition assets
pub fn spawn_research_entities(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut assets: ResMut<Assets<ResearchDefinition>>,
    mut events: MessageReader<AssetEvent<ResearchDefinition>>,
) {
    // Collect added asset IDs first to avoid borrow conflicts
    let added_ids: Vec<_> = events
        .read()
        .filter_map(|event| {
            if let AssetEvent::Added { id } = event {
                Some(*id)
            } else {
                None
            }
        })
        .collect();

    for id in added_ids {
        // Get definition info first and clone what we need
        let Some(def) = assets.get(id) else {
            continue;
        };

        // Check if already spawned
        if research_map.entities.contains_key(&def.id) {
            continue;
        }

        // Clone the definition ID before getting handle
        let def_id = def.id.clone();

        // Now get the strong handle (needs mutable borrow, but def is no longer borrowed)
        let Some(handle) = assets.get_strong_handle(id) else {
            continue;
        };

        let entity = commands
            .spawn((
                ResearchNode {
                    id: def_id.clone(),
                    handle,
                },
                Locked,
            ))
            .id();
        research_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned research entity: {} -> {:?}", def_id, entity);
    }
}

/// Listens for UnlockAchieved events with research_ prefix
pub fn on_unlock_achieved(
    trigger: On<unlocks_events::UnlockAchieved>,
    mut commands: Commands,
    research_map: Res<ResearchMap>,
    locked_query: Query<(), With<Locked>>,
) {
    let event = trigger.event();
    const PREFIX: &str = "research_";

    if event.reward_id.starts_with(PREFIX) {
        let research_id = &event.reward_id[PREFIX.len()..];
        if let Some(&entity) = research_map.entities.get(research_id) {
            // Only transition if currently Locked
            if locked_query.get(entity).is_ok() {
                commands.entity(entity).remove::<Locked>().insert(Available);
                info!("Research '{}' is now available", research_id);
            }
        } else {
            debug!(
                "Research '{}' not found in ResearchMap (unlock may have fired before asset load)",
                research_id
            );
        }
    }
}

/// Ticks timers for in-progress research
pub fn update_research_progress(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &ResearchNode, &mut InProgress)>,
) {
    for (entity, node, mut progress) in query.iter_mut() {
        progress.timer.tick(time.delta());
        if progress.timer.just_finished() {
            commands
                .entity(entity)
                .remove::<InProgress>()
                .insert(Completed);
            commands.trigger(ResearchCompleted {
                research_id: node.id.clone(),
            });
            info!("Research completed: {}", node.id);
        }
    }
}

/// Starts a research (validates Available + cost)
pub fn start_research(
    mut events: MessageReader<StartResearchRequest>,
    research_map: Res<ResearchMap>,
    assets: Res<Assets<ResearchDefinition>>,
    query: Query<&ResearchNode, With<Available>>,
    mut wallet: ResMut<Wallet>,
    mut commands: Commands,
) {
    for event in events.read() {
        let Some(&entity) = research_map.entities.get(&event.0) else {
            warn!("Research '{}' not found", event.0);
            continue;
        };

        let Ok(node) = query.get(entity) else {
            warn!("Research '{}' not available", event.0);
            continue;
        };

        let Some(def) = assets.get(&node.handle) else {
            warn!("Research definition not loaded for '{}'", event.0);
            continue;
        };

        // Deduct cost
        for (res, amt) in &def.cost {
            if let Some(resource_amt) = wallet.resources.get_mut(res) {
                *resource_amt -= amt;
            }
        }

        commands.entity(entity).remove::<Available>().insert(InProgress {
            timer: Timer::from_seconds(def.time_required, TimerMode::Once),
        });
        info!("Started researching: {}", def.name);
    }
}
