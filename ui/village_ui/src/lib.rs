use {
    bevy::{picking::prelude::*, prelude::*},
    crafting::{Available, RecipeNode},
    hero_components::{AttackRange, AttackSpeed, Damage, Hero, MeleeArc, MeleeWeapon, Weapon},
    hero_ui::{HeroUiRoot, spawn_hero_content},
    recipes_assets::{RecipeCategory, RecipeDefinition},
    research::{Completed, InProgress, ResearchCompletionCount, ResearchNode},
    research_assets::ResearchDefinition,
    shared_components::DisplayName,
    states::{EnemyEncyclopediaState, GameState},
    village_components::{EnemyEncyclopedia, Village},
    wallet::Wallet,
    widgets::{
        PanelWrapperRef, spawn_menu_button, spawn_menu_panel, spawn_panel_header_with_close,
    },
};

pub struct VillageUiPlugin;

impl Plugin for VillageUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_village_clicked).add_systems(
            Update,
            (handle_menu_button, handle_back_button, handle_close_button)
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(OnExit(GameState::Running), clean_up_village_ui);
    }
}

// ============================================================================
// Components
// ============================================================================

/// Current content displayed in the village UI
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum VillageContent {
    #[default]
    Menu,
    Crafting,
    Research,
    Encyclopedia,
    Heroes,
}

/// Root of the village UI
#[derive(Component)]
pub struct VillageUiRoot {
    pub content: VillageContent,
}

/// Close button marker
#[derive(Component)]
struct VillageCloseButton;

/// Container for switchable content
#[derive(Component)]
struct ContentContainer;

/// Menu button with target content
#[derive(Component)]
struct VillageMenuButton {
    target: VillageContent,
}

/// Back button to return to menu
#[derive(Component)]
struct VillageBackButton;

// ============================================================================
// Village Click Observer
// ============================================================================

fn on_village_clicked(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    village_query: Query<(), With<Village>>,
    existing_ui: Query<(Entity, Option<&PanelWrapperRef>), With<VillageUiRoot>>,
) {
    // Verify this is a village entity
    let clicked_entity = trigger.entity;
    if village_query.get(clicked_entity).is_err() {
        return;
    }

    // Toggle: if UI exists, close it; otherwise open
    if let Ok((ui_entity, wrapper_ref)) = existing_ui.single() {
        // Despawn wrapper if it exists, otherwise just despawn the panel
        if let Some(wrapper) = wrapper_ref {
            commands.entity(wrapper.0).despawn();
        } else {
            commands.entity(ui_entity).despawn();
        }
        return;
    }

    spawn_village_ui(&mut commands);
}

// ============================================================================
// Spawn Village UI
// ============================================================================

fn spawn_village_ui(commands: &mut Commands) {
    let panel = spawn_menu_panel(
        commands,
        VillageUiRoot {
            content: VillageContent::Menu,
        },
    );

    commands.entity(panel).with_children(|parent| {
        // Header with close button
        spawn_panel_header_with_close(parent, "Village", VillageCloseButton);

        // Content container
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::clip(),
                ..default()
            },
            ContentContainer,
        ));
    });

    // Populate with menu content
    commands.queue(SpawnMenuContentCommand);
}

// ============================================================================
// Menu Content Command
// ============================================================================

struct SpawnMenuContentCommand;

impl Command for SpawnMenuContentCommand {
    fn apply(self, world: &mut World) {
        let mut query =
            world.query_filtered::<(Entity, Option<&Children>), With<ContentContainer>>();

        let Some((container, children)) = query.iter(world).next() else {
            return;
        };

        // Despawn existing children
        let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
        for child in to_despawn {
            world.commands().entity(child).despawn();
        }

        // Spawn menu buttons
        world.commands().entity(container).with_children(|parent| {
            spawn_menu_button(
                parent,
                "⚒ Crafting",
                VillageMenuButton {
                    target: VillageContent::Crafting,
                },
            );
            spawn_menu_button(
                parent,
                "🔬 Research",
                VillageMenuButton {
                    target: VillageContent::Research,
                },
            );
            spawn_menu_button(
                parent,
                "📖 Encyclopedia",
                VillageMenuButton {
                    target: VillageContent::Encyclopedia,
                },
            );
            spawn_menu_button(
                parent,
                "🦸 Heroes",
                VillageMenuButton {
                    target: VillageContent::Heroes,
                },
            );
        });
    }
}

// ============================================================================
// Crafting Content Command
// ============================================================================

struct SpawnCraftingContentCommand;

impl Command for SpawnCraftingContentCommand {
    fn apply(self, world: &mut World) {
        let mut query =
            world.query_filtered::<(Entity, Option<&Children>), With<ContentContainer>>();

        let Some((container, children)) = query.iter(world).next() else {
            return;
        };

        // Despawn existing children
        let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
        for child in to_despawn {
            world.commands().entity(child).despawn();
        }

        // Query available recipe entities and collect ids and handles
        let mut recipe_query = world.query_filtered::<&RecipeNode, With<Available>>();
        let recipe_data: Vec<(String, bevy::asset::Handle<RecipeDefinition>)> = recipe_query
            .iter(world)
            .map(|node| (node.id.clone(), node.handle.clone()))
            .collect();

        let assets = world.resource::<Assets<RecipeDefinition>>();
        let wallet = world.resource::<Wallet>();

        // Build crafting display data from available recipes
        let active_tab = RecipeCategory::Weapons;
        let mut recipes = Vec::new();

        for (id, handle) in &recipe_data {
            let Some(def) = assets.get(handle) else {
                continue;
            };

            // Filter by active tab (default to Weapons)
            if def.category != active_tab {
                continue;
            }

            // Calculate cost string and affordability
            let mut can_afford = true;
            let mut cost_str = String::from("Cost: ");

            let mut cost_items: Vec<_> = def.cost.iter().collect();
            cost_items.sort_by_key(|(res_id, _)| *res_id);

            for (res_id, amt) in cost_items {
                let current = wallet.resources.get(res_id).copied().unwrap_or(0);
                cost_str.push_str(&format!("{}: {}/{} ", res_id, current, amt));
                if current < *amt {
                    can_afford = false;
                }
            }

            recipes.push(crafting_ui::RecipeDisplayData {
                id: id.clone(),
                display_name: def.display_name.clone(),
                craft_time: def.craft_time,
                cost_str,
                can_afford,
            });
        }

        let crafting_data = crafting_ui::CraftingData {
            active_tab,
            recipes,
        };

        // Spawn back button and crafting content
        world.commands().entity(container).with_children(|parent| {
            // Back button
            spawn_menu_button(parent, "← Back", VillageBackButton);

            // Spawn crafting content
            crafting_ui::spawn_crafting_content(parent, crafting_data);
        });
    }
}

// ============================================================================
// Research Content Command
// ============================================================================

struct SpawnResearchContentCommand;

impl Command for SpawnResearchContentCommand {
    fn apply(self, world: &mut World) {
        let mut query =
            world.query_filtered::<(Entity, Option<&Children>), With<ContentContainer>>();

        let Some((container, children)) = query.iter(world).next() else {
            return;
        };

        // Despawn existing children
        let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
        for child in to_despawn {
            world.commands().entity(child).despawn();
        }

        // Query research entities by state FIRST - collect into owned data
        let mut available_query = world.query_filtered::<(
            Entity,
            &ResearchNode,
            &ResearchCompletionCount,
        ), With<research::Available>>();
        let available_ids: Vec<(Entity, String, u32)> = available_query
            .iter(world)
            .map(|(e, n, c)| (e, n.id.clone(), c.0))
            .collect();

        let mut in_progress_query =
            world.query::<(Entity, &ResearchNode, &InProgress, &ResearchCompletionCount)>();
        let in_progress_ids: Vec<(Entity, String, u32)> = in_progress_query
            .iter(world)
            .map(|(e, n, _, c)| (e, n.id.clone(), c.0))
            .collect();

        let mut completed_query = world
            .query_filtered::<(Entity, &ResearchNode, &ResearchCompletionCount), With<Completed>>();

        // Now get resources needed for research content
        let assets = world.resource::<Assets<ResearchDefinition>>();
        let wallet = world.resource::<Wallet>();

        // Build research data directly using the assets
        let mut items = Vec::new();

        // Available research
        for (_, id, count) in &available_ids {
            if let Some((_handle, def)) = assets.iter().find(|(_, d)| &d.id == id) {
                let mut can_afford = true;
                let mut cost_str = String::from("Cost: ");
                for (res, amt) in &def.cost {
                    let current = wallet.resources.get(res).copied().unwrap_or(0);
                    cost_str.push_str(&format!("{}: {}/{} ", res, current, amt));
                    if current < *amt {
                        can_afford = false;
                    }
                }

                // Build progress info for repeatable research
                let progress_info = if def.max_repeats > 1 {
                    Some(format!("{}/{}", count, def.max_repeats))
                } else {
                    None
                };

                items.push(research_ui::ResearchDisplayData {
                    id: id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    time: def.time_required,
                    cost_str,
                    can_afford,
                    is_completed: false,
                    btn_text: if can_afford {
                        "Start".to_string()
                    } else {
                        "Start".to_string()
                    },
                    btn_color: if can_afford {
                        widgets::UiTheme::AFFORDABLE
                    } else {
                        widgets::UiTheme::BORDER_DISABLED
                    },
                    btn_border: if can_afford {
                        widgets::UiTheme::BORDER_SUCCESS
                    } else {
                        widgets::UiTheme::BORDER_DISABLED
                    },
                    progress_info,
                });
            }
        }

        // In-progress research
        for (_, id, count) in &in_progress_ids {
            if let Some((_handle, def)) = assets.iter().find(|(_, d)| &d.id == id) {
                // Build progress info for repeatable research
                let progress_info = if def.max_repeats > 1 {
                    Some(format!("{}/{}", count, def.max_repeats))
                } else {
                    None
                };

                items.push(research_ui::ResearchDisplayData {
                    id: id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    time: def.time_required,
                    cost_str: String::new(),
                    can_afford: true,
                    is_completed: false,
                    btn_text: "Researching...".to_string(),
                    btn_color: widgets::UiTheme::TEXT_INFO,
                    btn_border: bevy::color::Color::srgba(0.4, 0.4, 1.0, 1.0),
                    progress_info,
                });
            }
        }

        // Sort by name
        items.sort_by(|a, b| a.name.cmp(&b.name));

        let research_data = research_ui::ResearchData {
            active_tab: research_ui::ResearchTab::Available,
            items,
        };

        // Spawn back button and research content
        world.commands().entity(container).with_children(|parent| {
            // Back button
            spawn_menu_button(parent, "← Back", VillageBackButton);

            // Spawn research content
            research_ui::spawn_research_content(parent, research_data);
        });
    }
}

// ============================================================================
// Encyclopedia Content Command
// ============================================================================

struct SpawnEncyclopediaContentCommand;

impl Command for SpawnEncyclopediaContentCommand {
    fn apply(self, world: &mut World) {
        let mut query =
            world.query_filtered::<(Entity, Option<&Children>), With<ContentContainer>>();

        let Some((container, children)) = query.iter(world).next() else {
            return;
        };

        // Despawn existing children
        let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
        for child in to_despawn {
            world.commands().entity(child).despawn();
        }

        // Get encyclopedia from village
        let mut village_query = world.query::<&EnemyEncyclopedia>();
        let Some(encyclopedia) = village_query.iter(world).next() else {
            return;
        };

        let encyclopedia = encyclopedia.clone();

        // Spawn back button and encyclopedia content
        world.commands().entity(container).with_children(|parent| {
            // Back button
            spawn_menu_button(parent, "← Back", VillageBackButton);

            // Spawn encyclopedia content
            enemy_encyclopedia::spawn_enemy_encyclopedia_content(parent, &encyclopedia);
        });
    }
}

// ============================================================================
// Heroes Content Command
// ============================================================================

struct SpawnHeroesContentCommand;

impl Command for SpawnHeroesContentCommand {
    fn apply(self, world: &mut World) {
        let mut query =
            world.query_filtered::<(Entity, Option<&Children>), With<ContentContainer>>();

        let Some((container, children)) = query.iter(world).next() else {
            return;
        };

        // Despawn existing children
        let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
        for child in to_despawn {
            world.commands().entity(child).despawn();
        }

        // Query all heroes
        let mut hero_query = world.query_filtered::<Entity, With<Hero>>();
        let hero_entities: Vec<Entity> = hero_query.iter(world).collect();

        // Build display data for each hero
        let mut heroes_data: Vec<(Entity, hero_ui::HeroDisplayData)> = Vec::new();

        for hero_entity in &hero_entities {
            // Placeholder hero name (heroes don't have names yet)
            let name = "Hero".to_string();

            // Find weapon child
            let mut children_query = world.query::<&Children>();
            let weapon_children: Vec<Entity> = children_query
                .get(world, *hero_entity)
                .map(|c| c.iter().collect())
                .unwrap_or_default();

            let mut weapon_data = None;
            for child in weapon_children {
                let mut weapon_query = world.query_filtered::<(
                    Option<&DisplayName>,
                    &Damage,
                    &AttackRange,
                    &AttackSpeed,
                    Option<&MeleeArc>,
                ), With<Weapon>>();

                if let Ok((display_name, damage, range, speed, melee_arc)) =
                    weapon_query.get(world, child)
                {
                    // Extract all values before doing melee check to end the immutable borrow
                    let weapon_name = display_name
                        .map(|d| d.0.clone())
                        .unwrap_or_else(|| "Unknown Weapon".to_string());
                    let speed_secs = speed.timer.duration().as_secs_f32();
                    let damage_val = damage.0;
                    let range_val = range.0;
                    let arc_radians = melee_arc.map(|arc| arc.width);

                    // Check if it's a melee weapon for arc display
                    let mut melee_check = world.query_filtered::<(), With<MeleeWeapon>>();
                    let arc_degrees = if melee_check.get(world, child).is_ok() {
                        arc_radians.map(|arc| arc.to_degrees())
                    } else {
                        None
                    };

                    weapon_data = Some(hero_ui::WeaponDisplayData {
                        entity: child,
                        name: weapon_name,
                        damage: damage_val,
                        range: range_val,
                        speed_secs,
                        melee_arc: arc_degrees,
                    });
                    break;
                }
            }

            heroes_data.push((
                *hero_entity,
                hero_ui::HeroDisplayData {
                    entity: *hero_entity,
                    name,
                    weapon: weapon_data,
                },
            ));
        }

        // Spawn back button and heroes content
        world.commands().entity(container).with_children(|parent| {
            // Back button
            spawn_menu_button(parent, "← Back", VillageBackButton);

            // Add HeroUiRoot marker for state tracking
            parent.spawn(HeroUiRoot);

            // Spawn hero content
            spawn_hero_content(parent, heroes_data, 0);
        });
    }
}

// ============================================================================
// Button Handlers
// ============================================================================

#[allow(clippy::type_complexity)]
fn handle_menu_button(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &VillageMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ui_query: Query<&mut VillageUiRoot>,
    mut next_state: ResMut<NextState<EnemyEncyclopediaState>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut ui_root) = ui_query.single_mut() {
                ui_root.content = btn.target;

                match btn.target {
                    VillageContent::Crafting => {
                        next_state.set(EnemyEncyclopediaState::Closed);
                        commands.queue(SpawnCraftingContentCommand);
                    }
                    VillageContent::Research => {
                        next_state.set(EnemyEncyclopediaState::Closed);
                        commands.queue(SpawnResearchContentCommand);
                    }
                    VillageContent::Encyclopedia => {
                        next_state.set(EnemyEncyclopediaState::Open);
                        commands.queue(SpawnEncyclopediaContentCommand);
                    }
                    VillageContent::Menu => {
                        next_state.set(EnemyEncyclopediaState::Closed);
                        commands.queue(SpawnMenuContentCommand);
                    }
                    VillageContent::Heroes => {
                        next_state.set(EnemyEncyclopediaState::Closed);
                        commands.queue(SpawnHeroesContentCommand);
                    }
                }
            }
        }
    }
}

fn handle_back_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<VillageBackButton>)>,
    mut ui_query: Query<&mut VillageUiRoot>,
    mut next_state: ResMut<NextState<EnemyEncyclopediaState>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut ui_root) = ui_query.single_mut() {
                ui_root.content = VillageContent::Menu;
                next_state.set(EnemyEncyclopediaState::Closed);
                commands.queue(SpawnMenuContentCommand);
            }
        }
    }
}

fn handle_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<VillageCloseButton>)>,
    ui_query: Query<(Entity, Option<&PanelWrapperRef>), With<VillageUiRoot>>,
    mut next_state: ResMut<NextState<EnemyEncyclopediaState>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for (ui_entity, wrapper_ref) in ui_query.iter() {
                next_state.set(EnemyEncyclopediaState::Closed);
                // Despawn wrapper if it exists, otherwise just despawn the panel
                if let Some(wrapper) = wrapper_ref {
                    commands.entity(wrapper.0).despawn();
                } else {
                    commands.entity(ui_entity).despawn();
                }
            }
        }
    }
}

pub fn clean_up_village_ui(
    mut commands: Commands,
    menu_roots: Query<(Entity, Option<&PanelWrapperRef>), With<VillageUiRoot>>,
) {
    debug!("Cleaning up village UI");
    for (entity, wrapper_ref) in menu_roots.iter() {
        if let Some(wrapper) = wrapper_ref {
            commands.entity(wrapper.0).despawn();
        } else {
            commands.entity(entity).despawn();
        }
    }
}

