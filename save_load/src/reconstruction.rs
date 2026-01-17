//! Post-load reconstruction systems for the save/load system.
//!
//! These systems run in sequence after a save file is loaded to:
//! 1. Reconstruct weapon entities from WeaponInventory
//! 2. Relink in-progress research to research entities
//! 3. Reconstruct resource rates from completed research

use {
    bevy::prelude::*,
    hero_components::{EquippedWeaponId, Hero, WeaponId},
    research::{ResearchCompletionCount, ResearchMap, ResearchNode},
    states::LoadingPhase,
    village_components::{Village, WeaponInventory},
    wallet::ResourceRates,
    weapon_assets::{spawn_weapon, spawn_weapon_as_child, WeaponDefinition, WeaponMap},
};

/// Reconstructs weapon entities from the WeaponInventory and EquippedWeaponId.
/// 
/// 1. Spawns equipped weapons directly as children of Heroes.
/// 2. Spawns remaining unequipped weapons from inventory as loose entities.
pub fn reconstruct_weapons_from_inventory(
    mut commands: Commands,
    weapon_map: Res<WeaponMap>,
    weapon_assets: Res<Assets<WeaponDefinition>>,
    village_query: Query<&WeaponInventory, With<Village>>,
    // We iterate heroes to find what they should have equipped
    hero_query: Query<(Entity, &EquippedWeaponId), With<Hero>>,
) {
    let Ok(inventory) = village_query.single() else {
        warn!("No village found, skipping weapon reconstruction");
        return;
    };

    // Track how many of each weapon we spawn for heroes
    let mut spawned_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // 1. Spawn equipped weapons for Heroes
    for (hero_entity, equipped) in hero_query.iter() {
        if let Some(weapon_id) = &equipped.0 {
            if let Some(handle) = weapon_map.handles.get(weapon_id) {
                if let Some(def) = weapon_assets.get(handle) {
                    spawn_weapon_as_child(&mut commands, def, hero_entity);
                    *spawned_counts.entry(weapon_id.clone()).or_insert(0) += 1;
                    info!("Spawning equipped weapon '{}' for hero {:?}", weapon_id, hero_entity);
                } else {
                    warn!("Weapon definition not loaded for '{}'", weapon_id);
                }
            } else {
                warn!("Weapon '{}' not found in WeaponMap", weapon_id);
            }
        }
    }

    // 2. Spawn remaining unequipped weapons from Inventory
    // TODO: Detailed tracking of equipped vs inventory items is out of scope. 
    // Currently equipped items remain in inventory count. Future refactor should 
    // remove equipped items from inventory and re-add them when unequipped.
    
    // Count total inventory
    let mut inventory_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for weapon_id in &inventory.weapons {
        *inventory_counts.entry(weapon_id.clone()).or_insert(0) += 1;
    }

    // Spawn difference
    for (weapon_id, total_count) in inventory_counts {
        let spawned = spawned_counts.get(&weapon_id).copied().unwrap_or(0);
        let remaining = total_count.saturating_sub(spawned);

        if remaining > 0 {
            info!("Spawning {} unequipped copies of '{}'", remaining, weapon_id);
            if let Some(handle) = weapon_map.handles.get(&weapon_id) {
                if let Some(def) = weapon_assets.get(handle) {
                    for _ in 0..remaining {
                        spawn_weapon(&mut commands, def);
                    }
                } else {
                    warn!("Weapon definition not loaded for '{}'", weapon_id);
                }
            } else {
                warn!("Weapon '{}' not found in WeaponMap", weapon_id);
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
