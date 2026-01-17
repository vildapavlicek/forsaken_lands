//! Post-load reconstruction systems for the save/load system.
//!
//! These systems run in sequence after a save file is loaded to:
//! 1. Reconstruct weapon entities from WeaponInventory
//! 2. Relink in-progress research to research entities
//! 3. Reconstruct resource rates from completed research

use {
    bevy::prelude::*,
    hero_components::WeaponId,
    research::{ResearchCompletionCount, ResearchMap, ResearchNode},
    states::LoadingPhase,
    village_components::{Village, WeaponInventory},
    wallet::ResourceRates,
};

/// Reconstructs weapon entities from the WeaponInventory.
/// Weapons owned but not equipped need to be spawned from prefabs.
pub fn reconstruct_weapons_from_inventory(
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    village_query: Query<&WeaponInventory, With<Village>>,
    existing_weapons: Query<&WeaponId>,
) {
    let Ok(inventory) = village_query.single() else {
        warn!("No village found, skipping weapon reconstruction");
        return;
    };

    // Collect existing weapon IDs to avoid duplicates
    let existing_ids: std::collections::HashSet<_> = existing_weapons
        .iter()
        .map(|w| w.0.as_str())
        .collect();

    // Spawn weapons that are in inventory but not yet spawned
    for weapon_id in &inventory.weapons {
        if existing_ids.contains(weapon_id.as_str()) {
            debug!("Weapon '{}' already exists, skipping", weapon_id);
            continue;
        }

        // Load weapon prefab from recipes/prefabs/{id}.scn.ron
        let prefab_path = format!("recipes/prefabs/{}.scn.ron", weapon_id);
        let handle: Handle<DynamicScene> = asset_server.load(&prefab_path);
        
        // Spawn the weapon scene
        scene_spawner.spawn_dynamic(handle);
        info!("Spawning weapon '{}' from prefab", weapon_id);
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
            commands.entity(research_entity).insert(research::InProgress {
                research_id: in_progress.research_id.clone(),
                timer: in_progress.timer.clone(),
            });

            // Remove from wrong entity
            commands.entity(entity).remove::<research::InProgress>();

            // Also update state: remove Available/Locked, it's now in progress
            commands.entity(research_entity).remove::<research::Available>();
            commands.entity(research_entity).remove::<research::Locked>();

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

/// Reconstructs ResourceRates from completed research.
/// 
/// Note: Currently a placeholder since ResearchDefinition doesn't store effects.
/// Effects are applied via observers when research completes. For a full implementation,
/// we would need to either:
/// 1. Store effects in ResearchDefinition
/// 2. Replay the research completion events
/// 3. Store ResourceRates in the save file directly
pub fn reconstruct_resource_rates(
    mut rates: ResMut<ResourceRates>,
    research_query: Query<(&ResearchNode, &ResearchCompletionCount)>,
) {
    info!("Reconstructing resource rates from completed research");

    // Reset rates to default
    rates.rates.clear();

    // Log completed research for debugging
    for (node, count) in research_query.iter() {
        if count.0 > 0 {
            debug!(
                "Research '{}' completed {} times - effects would be applied here",
                node.id, count.0
            );
        }
    }

    // TODO: Implement effect replay when ResearchDefinition includes effects,
    // or consider saving ResourceRates directly in the save file.
    info!("Resource rate reconstruction complete (placeholder - effects not stored in definitions)");
}

/// Finishes the reconstruction phase and transitions to Ready.
pub fn finish_reconstruction(mut next_phase: ResMut<NextState<LoadingPhase>>) {
    info!("Reconstruction complete, transitioning to LoadingPhase::Ready");
    next_phase.set(LoadingPhase::Ready);
}
