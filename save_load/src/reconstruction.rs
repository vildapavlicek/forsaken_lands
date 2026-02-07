//! Post-load reconstruction systems for the save/load system.
//!
//! These systems run in sequence after a save file is loaded to:
//! 1. Reconstruct weapon entities from WeaponInventory
//! 2. Relink in-progress research to research entities
//! 3. Reconstruct resource rates from completed research

use {
    bevy::prelude::*,
    blessings::Blessings,
    hero_components::{EquippedWeaponId, Hero},
    research::{ResearchCompletionCount, ResearchMap, ResearchNode},
    states::LoadingPhase,
    village_components::{Village, WeaponInventory},
    weapon_factory_events,
};

/// Reconstructs weapon entities from the WeaponInventory and EquippedWeaponId.
///
/// 1. Spawns equipped weapons directly as children of Heroes.
/// 2. Spawns remaining unequipped weapons from inventory as loose entities.
pub fn reconstruct_weapons_from_inventory(
    mut commands: Commands,
    village_query: Query<&WeaponInventory, With<Village>>,
    // We iterate heroes to find what they should have equipped
    hero_query: Query<(Entity, &EquippedWeaponId), With<Hero>>,
) {
    let Ok(inventory) = village_query.single() else {
        warn!("No village found, skipping weapon reconstruction");
        return;
    };

    // Track how many of each weapon we spawn for heroes
    let mut spawned_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    // 1. Spawn equipped weapons for Heroes
    for (hero_entity, equipped) in hero_query.iter() {
        if let Some(weapon_id) = &equipped.0 {
            commands.trigger(weapon_factory_events::SpawnWeaponRequest {
                weapon_id: weapon_id.clone(),
                parent: Some(hero_entity),
                add_to_inventory: false,
            });
            *spawned_counts.entry(weapon_id.clone()).or_insert(0) += 1;
            info!(
                "Spawning equipped weapon '{}' for hero {:?}",
                weapon_id, hero_entity
            );
        }
    }

    // 2. Spawn remaining unequipped weapons from Inventory
    // TODO: Detailed tracking of equipped vs inventory items is out of scope.
    // Currently equipped items remain in inventory count. Future refactor should
    // remove equipped items from inventory and re-add them when unequipped.

    // Count total inventory
    let inventory_counts: std::collections::HashMap<String, usize> =
        inventory
            .weapons
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, weapon_id| {
                *acc.entry(weapon_id.clone()).or_insert(0) += 1;
                acc
            });

    // Spawn difference
    for (weapon_id, total_count) in inventory_counts {
        let spawned = spawned_counts.get(&weapon_id).copied().unwrap_or(0);
        let remaining = total_count.saturating_sub(spawned);

        if remaining > 0 {
            info!(
                "Spawning {} unequipped copies of '{}'",
                remaining, weapon_id
            );
            for _ in 0..remaining {
                commands.trigger(weapon_factory_events::SpawnWeaponRequest {
                    weapon_id: weapon_id.clone(),
                    parent: None,
                    add_to_inventory: false,
                });
            }
        }
    }

    info!("Weapon reconstruction complete");
}

/// Relinks in-progress research components to the correct research entities.
pub fn relink_in_progress_research(
    mut commands: Commands,
    research_map: Res<ResearchMap>,
    in_progress_query: Query<(Entity, &research::InProgress)>,
    research_nodes: Query<Entity, With<ResearchNode>>,
) {
    info!("Relinking in-progress research to research entities");

    for (entity, in_progress) in in_progress_query.iter() {
        // Check if this InProgress is on a ResearchNode (correct) or orphaned
        if research_nodes.contains(entity) {
            debug!(
                "InProgress for '{}' is already on correct entity",
                in_progress.research_id
            );
            continue;
        }

        // Find the correct research entity
        if let Some(&research_entity) = research_map.entities.get(&in_progress.research_id) {
            // Transfer InProgress to the correct entity
            commands
                .entity(research_entity)
                .insert(research::InProgress {
                    research_id: in_progress.research_id.clone(),
                    timer: in_progress.timer.clone(),
                });

            // Remove from wrong entity
            commands.entity(entity).remove::<research::InProgress>();

            // Also update state: remove Available/Locked, it's now in progress
            commands
                .entity(research_entity)
                .remove::<research::Available>();
            commands
                .entity(research_entity)
                .remove::<research::Locked>();

            info!(
                "Relinked InProgress '{}' to research entity {:?}",
                in_progress.research_id, research_entity
            );
        } else {
            warn!(
                "Could not find research entity for in-progress research '{}'",
                in_progress.research_id
            );
            // Remove orphaned InProgress
            commands.entity(entity).remove::<research::InProgress>();
        }
    }

    info!("Research relinking complete");
}

/// Hydrates the unlock system by replaying research completion events.
///
/// This iterates through all completed research and triggers `StatusCompleted` events
/// for each completion count. This ensures that:
/// 1. Repeatable unlocks (e.g., stacking stats) are re-applied correct number of times.
/// 2. One-time unlocks are marked as completed in the new session.
pub fn hydrate_research_unlocks(
    mut commands: Commands,
    research_query: Query<(&ResearchNode, &ResearchCompletionCount)>,
) {
    info!("Hydrating unlock system from research history...");

    let mut total_triggers = 0;

    for (node, count) in research_query.iter() {
        if count.0 > 0 {
            let topic = format!("research:{}", node.id);
            debug!("Replaying '{}' completion {} times", topic, count.0);

            // Replay the completion event N times
            for _ in 0..count.0 {
                commands.trigger(unlocks_events::StatusCompleted {
                    topic: topic.clone(),
                });
                total_triggers += 1;
            }
        }
    }

    info!(
        "Hydration complete. Replayed {} completion events.",
        total_triggers
    );
}

/// Hydrates the unlock system by replaying blessing purchase events.
///
/// Iterates through all purchased blessings and triggers `ValueChanged` events
/// to restore the topic values (e.g., "blessing:my_blessing_id" = level).
/// This allows dependent unlocks (like stat bonuses) to re-trigger.
pub fn hydrate_blessed_unlocks(mut commands: Commands, blessings_query: Query<&Blessings>) {
    info!("Hydrating unlock system from blessings...");

    for blessings in blessings_query.iter() {
        for (blessing_id, level) in &blessings.unlocked {
            // Restore topic value
            commands.trigger(unlocks_events::ValueChanged {
                topic: format!("blessing:{}", blessing_id),
                value: *level as f32,
            });
            debug!(
                "Restored blessing topic: blessing:{} = {}",
                blessing_id, level
            );
        }
    }
}

/// Finishes the reconstruction phase and transitions to Ready.
pub fn finish_reconstruction(mut next_phase: ResMut<NextState<LoadingPhase>>) {
    info!("Reconstruction complete, transitioning to LoadingPhase::Ready");
    next_phase.set(LoadingPhase::Ready);
}
