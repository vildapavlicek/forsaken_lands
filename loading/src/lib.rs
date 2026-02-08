mod resources;

use {
    crate::resources::{
        BlessingsFolderHandle, BonusStatsFolderHandle, EnemyPrefabsFolderHandle,
        RecipesFolderHandle, ResearchFolderHandle, SkillsFolderHandle, UnlocksFolderHandle,
        WeaponsFolderHandle,
    },
    bevy::{
        asset::LoadedFolder, ecs::system::SystemParam, platform::collections::HashMap, prelude::*,
    },
    blessings::BlessingDefinition,
    crafting_resources::RecipeMap,
    divinity_components::Divinity,
    enemy_components::MonsterId,
    portal_assets::SpawnTable,
    recipes_assets::RecipeDefinition,
    research::ResearchMap,
    research_assets::ResearchDefinition,
    serde::de::DeserializeSeed,
    skills_assets::{SkillDefinition, SkillMap},
    states::{GameState, LoadingPhase},
    std::{fs, path::Path},
    unlocks::{CompiledUnlock, TopicMap, UnlockState},
    unlocks_assets::UnlockDefinition,
    unlocks_events::{StatusCompleted, ValueChanged},
    village_components::{EnemyEncyclopedia, Village},
    wallet::Wallet,
    weapon_assets::{WeaponDefinition, WeaponMap},
};

pub struct LoadingManagerPlugin;

impl Plugin for LoadingManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingManager>()
            .init_resource::<LoadingStatus>()
            .init_resource::<SceneToLoad>()
            .init_state::<LoadingPhase>()
            // Phase: Assets - load all asset folders
            .add_systems(
                Startup,
                (
                    load_static_assets,
                    load_enemy_prefabs,
                    load_unlocks_assets,
                    load_research_assets,
                    load_recipes_assets,
                    load_weapons_assets,
                    load_blessings_assets,
                    load_bonus_stats_assets,
                    load_skills_assets,
                ),
            )
            .add_systems(OnEnter(LoadingPhase::Assets), update_scene_handle)
            .add_systems(
                Update,
                check_assets_loaded
                    .run_if(in_state(GameState::Loading).and(in_state(LoadingPhase::Assets))),
            )
            // Phase: SpawnEntities - spawn research and recipe entities
            .add_systems(OnEnter(LoadingPhase::SpawnEntities), spawn_all_entities)
            // Phase: CompileUnlocks - build unlock logic graphs
            .add_systems(
                OnEnter(LoadingPhase::CompileUnlocks),
                (
                    compile_unlocks,
                    compile_research_unlocks,
                    compile_recipe_unlocks,
                    compile_blessing_unlocks,
                    bonus_stats::plugin::compile_bonus_stats_unlocks,
                ),
            )
            .add_systems(
                Update,
                finish_compilation.run_if(in_state(LoadingPhase::CompileUnlocks)),
            )
            // Phase: EvaluateUnlocks - re-fire signals for satisfied conditions
            .add_systems(OnEnter(LoadingPhase::EvaluateUnlocks), evaluate_unlocks)
            // Phase: SpawnScene - spawn scene (startup or save)
            .add_systems(OnEnter(LoadingPhase::SpawnScene), spawn_scene)
            .add_systems(
                Update,
                check_scene_spawned.run_if(in_state(LoadingPhase::SpawnScene)),
            )
            // Phase: Ready - transition to Running
            .add_systems(OnEnter(LoadingPhase::Ready), finish_loading)
            // Loading UI
            .add_systems(
                OnEnter(GameState::Loading),
                (setup_loading_ui, reset_loading_phase, clear_unlock_state),
            )
            .add_systems(
                Update,
                update_loading_ui.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), cleanup_loading_ui);
    }
}

// --- Resources ---

#[derive(Resource)]
pub struct SceneToLoad {
    pub path: String,
    pub is_save: bool,
}

impl Default for SceneToLoad {
    fn default() -> Self {
        Self {
            path: "startup.scn.ron".to_string(),
            is_save: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct LoadingManager {
    pub startup_scene: Handle<DynamicScene>,
    pub spawn_tables: HashMap<String, Handle<SpawnTable>>,
    pub enemies: HashMap<String, Handle<DynamicScene>>,
}

// Keep GameAssets as alias for backwards compatibility
pub type GameAssets = LoadingManager;

#[derive(Resource, Default)]
pub struct LoadingStatus {
    pub current_phase: String,
    pub detail: String,
}

// --- Phase: Assets ---

fn update_scene_handle(
    mut assets: ResMut<LoadingManager>,
    asset_server: Res<AssetServer>,
    scene_to_load: Res<SceneToLoad>,
) {
    info!(
        "Starting asset load phase. target scene: {}",
        scene_to_load.path
    );
    if !scene_to_load.is_save {
        assets.startup_scene = asset_server.load(&scene_to_load.path);
    } else {
        info!("Skipping asset server load for save file (will be loaded manually)");
    }
}

fn load_static_assets(mut assets: ResMut<LoadingManager>, asset_server: Res<AssetServer>) {
    info!("Loading static assets (spawn tables, etc)");
    let default_spawn_table = asset_server.load("default.spawn_table.ron");

    assets
        .spawn_tables
        .insert(String::from("default"), default_spawn_table);
}

fn load_enemy_prefabs(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("prefabs/enemies");
    cmd.insert_resource(EnemyPrefabsFolderHandle(handle));
}

fn load_unlocks_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("unlocks");
    cmd.insert_resource(UnlocksFolderHandle(handle));
}

fn load_research_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("research");
    cmd.insert_resource(ResearchFolderHandle(handle));
}

fn load_recipes_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("recipes");
    cmd.insert_resource(RecipesFolderHandle(handle));
}

fn load_weapons_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("weapons");
    cmd.insert_resource(WeaponsFolderHandle(handle));
}

fn load_blessings_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("blessings");
    cmd.insert_resource(BlessingsFolderHandle(handle));
}

fn load_bonus_stats_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("stats");
    cmd.insert_resource(BonusStatsFolderHandle(handle));
}

fn load_skills_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("skills");
    cmd.insert_resource(SkillsFolderHandle(handle));
}

#[derive(SystemParam)]
struct FolderHandles<'w> {
    enemy_prefabs: Res<'w, EnemyPrefabsFolderHandle>,
    unlocks: Res<'w, UnlocksFolderHandle>,
    research: Res<'w, ResearchFolderHandle>,
    recipes: Res<'w, RecipesFolderHandle>,
    weapons: Res<'w, WeaponsFolderHandle>,
    blessings: Res<'w, BlessingsFolderHandle>,
    bonus_stats: Res<'w, BonusStatsFolderHandle>,
    skills: Res<'w, SkillsFolderHandle>,
}

#[allow(clippy::too_many_arguments)]
fn check_assets_loaded(
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut loading_manager: ResMut<LoadingManager>,
    mut weapon_map: ResMut<WeaponMap>,
    mut skill_map: ResMut<SkillMap>,
    mut status: ResMut<LoadingStatus>,
    asset_server: Res<AssetServer>,
    folders: FolderHandles,
    folder: Res<Assets<LoadedFolder>>,
    weapon_assets: Res<Assets<WeaponDefinition>>,
    skill_assets: Res<Assets<SkillDefinition>>,
    scenes: Res<Assets<DynamicScene>>,
    type_registry: Res<AppTypeRegistry>,
) {
    status.current_phase = "Loading Assets".into();
    status.detail = "Loading files from disk...".into();

    let spawn_tables_loaded = loading_manager
        .spawn_tables
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    // For the scene file, we only need to check if it's loaded.
    // However, if we are loading a save, we might have already loaded assets in a previous run.
    // But `asset_server.is_loaded_with_dependencies` is generally cheap if already loaded.
    if asset_server.is_loaded_with_dependencies(&loading_manager.startup_scene)
        && spawn_tables_loaded
        && asset_server.is_loaded_with_dependencies(folders.enemy_prefabs.0.id())
        && asset_server.is_loaded_with_dependencies(folders.unlocks.0.id())
        && asset_server.is_loaded_with_dependencies(folders.research.0.id())
        && asset_server.is_loaded_with_dependencies(folders.recipes.0.id())
        && asset_server.is_loaded_with_dependencies(folders.weapons.0.id())
        && asset_server.is_loaded_with_dependencies(folders.blessings.0.id())
        && asset_server.is_loaded_with_dependencies(folders.bonus_stats.0.id())
        && asset_server.is_loaded_with_dependencies(folders.skills.0.id())
    {
        info!("assets loaded");

        let Some(enemy_prefabs_folder) = folder.get(folders.enemy_prefabs.0.id()) else {
            panic!("folder not loaded even tho asset server said it is")
        };

        for untyped_handle in enemy_prefabs_folder.handles.iter().cloned() {
            let Some(asset_path) = asset_server.get_path(untyped_handle.id()) else {
                continue;
            };

            let path = asset_path.path().display().to_string();

            let Ok(handle) = untyped_handle.try_typed::<DynamicScene>() else {
                continue;
            };

            // Extract MonsterId from the loaded scene
            let key = extract_monster_id(&scenes, &handle, &type_registry).unwrap_or_else(|| {
                panic!("MonsterId component not found in enemy prefab: {}", path)
            });

            debug!(%key, %path, "loaded enemy prefab with MonsterId");
            loading_manager.enemies.insert(key, handle);
        }

        // Populate WeaponMap from loaded weapon assets
        let Some(weapons_folder) = folder.get(folders.weapons.0.id()) else {
            panic!("weapons folder not loaded even though asset server said it is")
        };

        for untyped_handle in weapons_folder.handles.iter().cloned() {
            let Ok(handle) = untyped_handle.try_typed::<WeaponDefinition>() else {
                continue;
            };

            if let Some(def) = weapon_assets.get(&handle) {
                debug!("Loaded weapon definition: {}", def.id);
                weapon_map.handles.insert(def.id.clone(), handle);
            }
        }

        // Populate SkillMap from loaded skill assets
        let Some(skills_folder) = folder.get(folders.skills.0.id()) else {
            panic!("skills folder not loaded even though asset server said it is")
        };

        for untyped_handle in skills_folder.handles.iter().cloned() {
            let Ok(handle) = untyped_handle.try_typed::<SkillDefinition>() else {
                continue;
            };

            if let Some(def) = skill_assets.get(&handle) {
                debug!("Loaded skill definition: {}", def.id);
                skill_map.handles.insert(def.id.clone(), handle);
            }
        }

        next_phase.set(LoadingPhase::SpawnScene);
    }
}

/// Extracts the MonsterId component value from a loaded DynamicScene.
fn extract_monster_id(
    scenes: &Assets<DynamicScene>,
    handle: &Handle<DynamicScene>,
    _type_registry: &AppTypeRegistry,
) -> Option<String> {
    let scene = scenes.get(handle)?;

    for entity in &scene.entities {
        for component in &entity.components {
            // Check if this component is a MonsterId
            let type_info = component.try_as_reflect()?.get_represented_type_info()?;
            if type_info.type_id() == std::any::TypeId::of::<MonsterId>() {
                // MonsterId is a tuple struct: MonsterId(String)
                // Access field 0 to get the inner String
                if let bevy::reflect::ReflectRef::TupleStruct(ts) = component.reflect_ref()
                    && let Some(field) = ts.field(0)
                    && let Some(s) = field.try_downcast_ref::<String>()
                {
                    return Some(s.clone());
                }
            }
        }
    }

    None
}

// --- Phase: SpawnEntities ---

#[allow(clippy::too_many_arguments)]
fn spawn_all_entities(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut recipe_map: ResMut<RecipeMap>,
    mut research_assets: ResMut<Assets<ResearchDefinition>>,
    mut recipe_assets: ResMut<Assets<RecipeDefinition>>,
    research_state: Res<research::ResearchState>,
    unlock_state: Res<UnlockState>,
    constructed_buildings: Res<crafting_resources::ConstructedBuildings>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Spawning Entities".into();
    status.detail = "Creating research and recipe nodes...".into();

    // Spawn research entities using persisted ResearchState + asset data
    // UnlockState is NOT used here - it's reconstructed during evaluate_unlocks
    debug!("Spawning research entities...");
    let research_ids: Vec<_> = research_assets.ids().collect();

    for id in research_ids {
        // Get definition info first and clone what we need
        let (def_id, max_repeats) = {
            let Some(def) = research_assets.get(id) else {
                continue;
            };

            if research_map.entities.contains_key(&def.id) {
                continue;
            }

            (def.id.clone(), def.max_repeats)
        };

        // Get persisted completion count (0 if not found)
        let saved_count = research_state
            .completion_counts
            .get(&def_id)
            .copied()
            .unwrap_or(0);

        let handle = research_assets.get_strong_handle(id).unwrap();

        // Determine state based on saved count vs asset max_repeats
        // - Completed: count >= max_repeats
        // - Available: count > 0 (was researched before, may have more repeats)
        // - Locked: count == 0 (the unlock system will transition to Available during evaluate_unlocks)
        let entity = if saved_count >= max_repeats {
            debug!(
                "Research '{}' fully completed ({}/{}), spawning as Completed",
                def_id, saved_count, max_repeats
            );
            commands
                .spawn((
                    research::ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    research::Completed,
                    research::ResearchCompletionCount(saved_count),
                ))
                .id()
        } else if saved_count > 0 {
            debug!(
                "Research '{}' partially completed ({}/{}), spawning as Available",
                def_id, saved_count, max_repeats
            );
            commands
                .spawn((
                    research::ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    research::Available,
                    research::ResearchCompletionCount(saved_count),
                ))
                .id()
        } else {
            // Locked - the unlock system will mark Available during evaluate_unlocks if conditions met
            commands
                .spawn((
                    research::ResearchNode {
                        id: def_id.clone(),
                        handle,
                    },
                    research::Locked,
                    research::ResearchCompletionCount(0),
                ))
                .id()
        };

        research_map.entities.insert(def_id.clone(), entity);
        debug!("Spawned research entity: {} -> {:?}", def_id, entity);
    }

    // Spawn recipe entities
    debug!("Spawning recipe entities...");
    crafting::spawn_recipe_entities(
        &mut commands,
        &mut recipe_map,
        &mut recipe_assets,
        &unlock_state,
        &constructed_buildings,
    );

    next_phase.set(LoadingPhase::CompileUnlocks);
}

// --- Phase: CompileUnlocks ---
use unlocks_resources::UnlockProgress;

fn compile_unlocks(
    mut commands: Commands,
    unlock_assets: Res<Assets<UnlockDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    unlock_state: Res<UnlockState>,
    unlock_progress: Res<UnlockProgress>,
    compiled: Query<&CompiledUnlock>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Compiling Unlocks".into();
    status.detail = "Building logic graphs...".into();

    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    for (_, definition) in unlock_assets.iter() {
        debug!(%definition.id, "compiling unlock");
        unlocks::compile_unlock_definition(
            &mut commands,
            &mut topic_map,
            definition,
            &compiled_ids,
            &unlock_state,
            &unlock_progress,
        );
    }
}

fn compile_research_unlocks(
    mut commands: Commands,
    research_assets: Res<Assets<ResearchDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    unlock_state: Res<UnlockState>,
    unlock_progress: Res<UnlockProgress>,
    compiled: Query<&CompiledUnlock>,
) {
    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    for (_, research) in research_assets.iter() {
        if let Some(unlock) = &research.unlock {
            unlocks::compile_unlock_definition(
                &mut commands,
                &mut topic_map,
                unlock,
                &compiled_ids,
                &unlock_state,
                &unlock_progress,
            );
        }
    }
}

fn compile_recipe_unlocks(
    mut commands: Commands,
    recipe_assets: Res<Assets<RecipeDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    unlock_state: Res<UnlockState>,
    unlock_progress: Res<UnlockProgress>,
    compiled: Query<&CompiledUnlock>,
) {
    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    for (_, recipe) in recipe_assets.iter() {
        if let Some(unlock) = &recipe.unlock {
            unlocks::compile_unlock_definition(
                &mut commands,
                &mut topic_map,
                unlock,
                &compiled_ids,
                &unlock_state,
                &unlock_progress,
            );
        }
    }
}

fn compile_blessing_unlocks(
    mut commands: Commands,
    blessing_assets: Res<Assets<BlessingDefinition>>,
    mut topic_map: ResMut<TopicMap>,
    unlock_state: Res<UnlockState>,
    unlock_progress: Res<UnlockProgress>,
    compiled: Query<&CompiledUnlock>,
) {
    let compiled_ids: std::collections::HashSet<_> =
        compiled.iter().map(|c| c.definition_id.as_str()).collect();

    for (_, blessing) in blessing_assets.iter() {
        if let Some(unlock) = &blessing.unlock {
            unlocks::compile_unlock_definition(
                &mut commands,
                &mut topic_map,
                unlock,
                &compiled_ids,
                &unlock_state,
                &unlock_progress,
            );
        }
    }
}

/// Transition phase after all compilation systems have run
fn finish_compilation(mut next_phase: ResMut<NextState<LoadingPhase>>) {
    next_phase.set(LoadingPhase::EvaluateUnlocks);
}

// --- Phase: EvaluateUnlocks ---

/// After all unlock logic graphs are compiled, trigger hydration events to update sensors.
/// This replaces the old approach of re-firing LogicSignalEvents.
fn evaluate_unlocks(
    mut commands: Commands,
    wallet: Res<Wallet>,
    encyclopedia_query: Query<&EnemyEncyclopedia, With<Village>>,
    divinity_query: Query<&Divinity, With<Village>>,
    claimed_divinity: Res<village_resources::DivinityUnlockState>,
    unlock_assets: Res<Assets<UnlockDefinition>>,
    research_state: Res<research::ResearchState>,
    research_query: Query<&research::ResearchNode>,
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    mut status: ResMut<LoadingStatus>,
) {
    status.current_phase = "Evaluating Unlocks".into();
    status.detail = "Hydrating state...".into();

    // Trigger ValueChanged for all wallet resources
    for (resource_id, &amount) in wallet.resources.iter() {
        commands.trigger(ValueChanged {
            topic: format!("resource:{}", resource_id),
            value: amount as f32,
        });
    }

    // Trigger ValueChanged for all kill/escape counts from encyclopedia
    if let Ok(encyclopedia) = encyclopedia_query.single() {
        for (monster_id, entry) in encyclopedia.inner.iter() {
            commands.trigger(ValueChanged {
                topic: format!("kills:{}", monster_id),
                value: entry.kill_count as f32,
            });
            commands.trigger(ValueChanged {
                topic: format!("escapes:{}", monster_id),
                value: entry.escape_count as f32,
            });
        }
    }

    // Trigger ValueChanged for village divinity
    if let Ok(divinity) = divinity_query.single() {
        // Encode divinity as tier*100 + level for comparison
        commands.trigger(ValueChanged {
            topic: "divinity".to_string(),
            value: (divinity.tier * 100 + divinity.level) as f32,
        });
    }

    // Trigger StatusCompleted for research that was completed (count > 0)
    for node in &research_query {
        if let Some(&count) = research_state.completion_counts.get(&node.id)
            && count > 0
        {
            debug!(research_id = %node.id, count = count, "Firing StatusCompleted for loaded research");
            commands.trigger(StatusCompleted {
                topic: format!("research:{}", node.id),
            });
        }
    }

    // Trigger UnlockAchieved for claimed divinity levels
    // This allows subsequent divinity levels (which depend on previous ones) to be evaluated correctly
    for unlock_id in &claimed_divinity.claimed {
        // Find definitions to get correct display name and reward id
        // Inefficient lookup but acceptable for loading screen with < 1000 items
        if let Some(_def) = unlock_assets
            .iter()
            .map(|(_, def)| def)
            .find(|d| &d.id == unlock_id)
        {
            debug!(unlock_id = %unlock_id, "Restoring claimed divinity status");
            commands.trigger(unlocks_events::StatusCompleted {
                topic: format!("divinity:{}", unlock_id),
            });
        }
    }

    info!("Hydrated unlock state from saved data");
    next_phase.set(LoadingPhase::PostLoadReconstruction);
}

// --- Phase: SpawnScene ---

fn spawn_scene(
    mut scene_spawner: ResMut<SceneSpawner>,
    mut dynamic_scenes: ResMut<Assets<DynamicScene>>,
    loading_manager: Res<LoadingManager>,
    mut status: ResMut<LoadingStatus>,
    scene_to_load: Res<SceneToLoad>,
    type_registry: Res<AppTypeRegistry>,
) {
    status.current_phase = "Spawning Scene".into();
    status.detail = "Loading world...".into();

    info!("spawning scene");

    if scene_to_load.is_save {
        // MANUAL LOAD for save files
        // Bypasses AssetServer to prevent hot-reloading when the save file is overwritten
        let path = Path::new("saves").join(&scene_to_load.path);
        info!("Manually loading save file from: {}", path.display());

        match fs::read(&path) {
            Ok(bytes) => {
                let type_registry = type_registry.read();
                let scene_deserializer = bevy::scene::serde::SceneDeserializer {
                    type_registry: &type_registry,
                };

                let mut deserializer =
                    ron::Deserializer::from_bytes(&bytes).expect("Failed to create deserializer");

                match scene_deserializer.deserialize(&mut deserializer) {
                    Ok(dynamic_scene) => {
                        info!("Successfully manually deserialized save scene");
                        let handle = dynamic_scenes.add(dynamic_scene);
                        scene_spawner.spawn_dynamic(handle);
                    }
                    Err(e) => error!("Failed to deserialize save scene: {}", e),
                }
            }
            Err(e) => error!("Failed to read save file {}: {}", path.display(), e),
        }
    } else {
        scene_spawner.spawn_dynamic(loading_manager.startup_scene.clone());
    }
}

fn check_scene_spawned(
    mut next_phase: ResMut<NextState<LoadingPhase>>,
    query: Query<(), With<Village>>,
    scene_to_load: Res<SceneToLoad>,
) {
    if !query.is_empty() {
        info!("Scene spawned and validated");
        if scene_to_load.is_save {
            info!("Save loaded - now spawning entities with loaded EnemyEncyclopedia");
        } else {
            info!("New game - now spawning entities");
        }
        next_phase.set(LoadingPhase::SpawnEntities);
    }
}

// --- Phase: Ready ---

fn finish_loading(mut next_state: ResMut<NextState<GameState>>) {
    info!("Loading complete, transitioning to Running");
    next_state.set(GameState::Running);
}

// --- Loading UI ---

#[derive(Component)]
struct LoadingUi;

fn setup_loading_ui(mut commands: Commands) {
    info!("spawning loading ui");
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        LoadingUi,
    ));
}

fn update_loading_ui(status: Res<LoadingStatus>, mut query: Query<&mut Text, With<LoadingUi>>) {
    if let Ok(mut text) = query.single_mut() {
        *text = Text::new(format!(
            "Loading...\n{}\n{}",
            status.current_phase, status.detail
        ));
    }
}

fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingUi>>) {
    info!("cleaning up loading ui");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn reset_loading_phase(mut next_phase: ResMut<NextState<LoadingPhase>>) {
    info!("Resetting LoadingPhase to Assets");
    next_phase.set(LoadingPhase::Assets);
}

fn clear_unlock_state(mut unlock_state: ResMut<UnlockState>) {
    info!("Clearing UnlockState to prevent state leakage from previous sessions");
    unlock_state.completed.clear();
}
