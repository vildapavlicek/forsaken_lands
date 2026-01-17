//! Post-load reconstruction systems for the save/load system.
//!
//! These systems run in sequence after a save file is loaded to:
//! 1. Wait for the scene to spawn
//! 2. Reconstruct weapon entities from WeaponInventory
//! 3. Rebuild Research and Recipe entity maps
//! 4. Relink in-progress research to research entities
//! 5. Reconstruct resource rates from completed research

use {
    bevy::prelude::*,
    crafting_resources::RecipeMap,
    hero_components::WeaponId,
    recipes_assets::RecipeDefinition,
    research::{ResearchCompletionCount, ResearchMap, ResearchNode},
    research_assets::ResearchDefinition,
    states::LoadingSavePhase,
    unlocks_resources::UnlockState,
    village_components::{Village, WeaponInventory},
    wallet::ResourceRates,
};

use crate::LoadingSaveHandle;

/// Spawns the save scene.
pub fn spawn_save_scene(save_handle: Res<LoadingSaveHandle>, mut scene_spawner: ResMut<SceneSpawner>) {
    let Some(handle) = &save_handle.0 else {
        warn!("No save handle set, cannot spawn scene");
        return;
    };

    info!("Spawning save scene");
    scene_spawner.spawn_dynamic(handle.clone());
}

/// Checks if the save scene is fully spawned and transitions to next phase.
pub fn check_scene_loaded(
    mut next_phase: ResMut<NextState<LoadingSavePhase>>,
    village_query: Query<&Transform, With<Village>>,
) {
    // Check if village entity exists AND has Transform (scene is fully spawned)
    if let Ok(transform) = village_query.single() {
        info!("Save scene spawned. Village found with Transform at {:?}. Proceeding to weapon reconstruction.", transform.translation);
        next_phase.set(LoadingSavePhase::ReconstructingWeapons);
    }
}

/// Reconstructs weapon entities from the WeaponInventory.
/// Weapons owned but not equipped need to be spawned from prefabs.
pub fn reconstruct_weapons_from_inventory(
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    village_query: Query<&WeaponInventory, With<Village>>,
    existing_weapons: Query<&WeaponId>,
    mut next_phase: ResMut<NextState<LoadingSavePhase>>,
) {
    let Ok(inventory) = village_query.single() else {
        warn!("No village found, skipping weapon reconstruction");
        next_phase.set(LoadingSavePhase::RebuildingMaps);
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

    info!("Weapon reconstruction complete, proceeding to map rebuilding");
    next_phase.set(LoadingSavePhase::RebuildingMaps);
}

/// Rebuilds the ResearchMap and RecipeMap from assets.
#[allow(clippy::too_many_arguments)]
pub fn rebuild_research_recipe_maps(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut recipe_map: ResMut<RecipeMap>,
    mut research_assets: ResMut<Assets<ResearchDefinition>>,
    mut recipe_assets: ResMut<Assets<RecipeDefinition>>,
    unlock_state: Res<UnlockState>,
    mut next_phase: ResMut<NextState<LoadingSavePhase>>,
) {
    info!("Rebuilding research and recipe maps from assets");

    // Collect research IDs first to avoid borrow conflicts
    let research_ids: Vec<_> = research_assets.ids().collect();

    for id in research_ids {
        let (def_id, already_unlocked) = {
            let Some(def) = research_assets.get(id) else {
                continue;
            };

            if research_map.entities.contains_key(&def.id) {
                continue;
            }

            let already_unlocked = unlock_state.completed.iter().any(|unlock_id| {
                unlock_id.ends_with(&format!("{}_unlock", def.id))
                    || unlock_id.starts_with(&format!("research_{}", def.id))
            });

            (def.id.clone(), already_unlocked)
        };

        let Some(handle) = research_assets.get_strong_handle(id) else {
            warn!("Could not get strong handle for research '{}'", def_id);
            continue;
        };

        let entity = if already_unlocked {
            commands
                .spawn((
                    ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    research::Available,
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
                    research::Locked,
                    ResearchCompletionCount(0),
                ))
                .id()
        };

        research_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned research entity: {} -> {:?}", def_id, entity);
    }

    // Collect recipe IDs first to avoid borrow conflicts
    let recipe_ids: Vec<_> = recipe_assets.ids().collect();

    for id in recipe_ids {
        let (def_id, already_unlocked) = {
            let Some(def) = recipe_assets.get(id) else {
                continue;
            };

            if recipe_map.entities.contains_key(&def.id) {
                continue;
            }

            let already_unlocked = unlock_state.completed.iter().any(|unlock_id| {
                unlock_id == &format!("recipe_{}_unlock", def.id)
                    || unlock_id.starts_with(&format!("recipe_{}", def.id))
            });

            (def.id.clone(), already_unlocked)
        };

        let Some(handle) = recipe_assets.get_strong_handle(id) else {
            warn!("Could not get strong handle for recipe '{}'", def_id);
            continue;
        };

        let entity = if already_unlocked {
            commands
                .spawn((
                    crafting::RecipeNode {
                        id: def_id.clone(),
                        handle,
                    },
                    unlock_states::Available,
                ))
                .id()
        } else {
            commands
                .spawn((
                    crafting::RecipeNode {
                        id: def_id.clone(),
                        handle,
                    },
                    unlock_states::Locked,
                ))
                .id()
        };

        recipe_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned recipe entity: {} -> {:?}", def_id, entity);
    }

    info!("Map rebuilding complete, proceeding to research relinking");
    next_phase.set(LoadingSavePhase::RelinkingResearch);
}

/// Relinks in-progress research components to the correct research entities.
pub fn relink_in_progress_research(
    mut commands: Commands,
    research_map: Res<ResearchMap>,
    in_progress_query: Query<(Entity, &research::InProgress)>,
    research_nodes: Query<Entity, With<ResearchNode>>,
    mut next_phase: ResMut<NextState<LoadingSavePhase>>,
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

    info!("Research relinking complete, proceeding to rate reconstruction");
    next_phase.set(LoadingSavePhase::ReconstructingRates);
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
    mut next_phase: ResMut<NextState<LoadingSavePhase>>,
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
    next_phase.set(LoadingSavePhase::Complete);
}
