use {
    bevy::prelude::*,
    research::{
        Available, Completed, InProgress, ResearchDefinition, ResearchMap, ResearchNode,
        StartResearchRequest,
    },
    states::GameState,
    wallet::Wallet,
    widgets::{
        spawn_action_button, spawn_card_title, spawn_description_text, spawn_scrollable_container,
        spawn_tab_bar, spawn_tab_button, spawn_timer_text, UiTheme,
    },
};

pub struct ResearchUiPlugin;

impl Plugin for ResearchUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_research_close_button,
                handle_tab_switch,
                update_research_ui,
                handle_research_button,
            )
                .run_if(in_state(GameState::Running)),
        );
    }
}

// ============================================================================
// Components
// ============================================================================

#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum ResearchTab {
    Available,
    Completed,
}

/// Root of the research popup card
#[derive(Component)]
pub struct ResearchUiRoot {
    pub active_tab: ResearchTab,
}

/// Close button marker
#[derive(Component)]
pub struct ResearchCloseButton;

/// Tab button with category
#[derive(Component)]
pub struct ResearchTabButton {
    pub tab: ResearchTab,
}

#[derive(Component)]
pub struct ResearchButton {
    pub id: String,
}

#[derive(Component)]
pub struct ResearchItemsContainer;

// ============================================================================
// Research Data Builder (for external use)
// ============================================================================

/// Data needed to display research content
pub struct ResearchData {
    pub active_tab: ResearchTab,
    pub items: Vec<ResearchDisplayData>,
}

/// Display data for a single research item
pub struct ResearchDisplayData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub time: f32,
    pub cost_str: String,
    pub can_afford: bool,
    pub is_completed: bool,
    pub btn_text: String,
    pub btn_color: Color,
    pub btn_border: Color,
}

/// Builds research display data from entity queries
pub fn build_research_data(
    assets: &Assets<ResearchDefinition>,
    wallet: &Wallet,
    available_query: &[(Entity, &ResearchNode)],
    in_progress_query: &[(Entity, &ResearchNode, &InProgress)],
    completed_query: &[(Entity, &ResearchNode)],
) -> ResearchData {
    let active_tab = ResearchTab::Available;
    let items = build_research_list(
        assets,
        wallet,
        active_tab,
        available_query,
        in_progress_query,
        completed_query,
    );
    ResearchData { active_tab, items }
}

fn build_research_list(
    assets: &Assets<ResearchDefinition>,
    wallet: &Wallet,
    active_tab: ResearchTab,
    available_query: &[(Entity, &ResearchNode)],
    in_progress_query: &[(Entity, &ResearchNode, &InProgress)],
    completed_query: &[(Entity, &ResearchNode)],
) -> Vec<ResearchDisplayData> {
    let mut research_data = Vec::new();

    match active_tab {
        ResearchTab::Available => {
            // Available research (can be started)
            for (_entity, node) in available_query {
                let Some(def) = assets.get(&node.handle) else {
                    continue;
                };

                let mut can_afford = true;
                let mut cost_str = String::from("Cost: ");
                for (res, amt) in &def.cost {
                    let current = wallet.resources.get(res).copied().unwrap_or(0);
                    cost_str.push_str(&format!("{}: {}/{} ", res, current, amt));
                    if current < *amt {
                        can_afford = false;
                    }
                }

                let (btn_text, btn_color, btn_border) = if can_afford {
                    (
                        "Start".to_string(),
                        UiTheme::AFFORDABLE,
                        UiTheme::BORDER_SUCCESS,
                    )
                } else {
                    (
                        "Start".to_string(),
                        UiTheme::BORDER_DISABLED,
                        UiTheme::BORDER_DISABLED,
                    )
                };

                research_data.push(ResearchDisplayData {
                    id: node.id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    time: def.time_required,
                    cost_str,
                    can_afford,
                    is_completed: false,
                    btn_text,
                    btn_color,
                    btn_border,
                });
            }

            // In-progress research
            for (_entity, node, _progress) in in_progress_query {
                let Some(def) = assets.get(&node.handle) else {
                    continue;
                };

                research_data.push(ResearchDisplayData {
                    id: node.id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    time: def.time_required,
                    cost_str: String::new(),
                    can_afford: true,
                    is_completed: false,
                    btn_text: "Researching...".to_string(),
                    btn_color: UiTheme::TEXT_INFO,
                    btn_border: Color::srgba(0.4, 0.4, 1.0, 1.0),
                });
            }

            // Sort by name
            research_data.sort_by(|a, b| a.name.cmp(&b.name));
        }
        ResearchTab::Completed => {
            for (_entity, node) in completed_query {
                let Some(def) = assets.get(&node.handle) else {
                    continue;
                };

                research_data.push(ResearchDisplayData {
                    id: node.id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    time: 0.0,
                    cost_str: String::new(),
                    can_afford: true,
                    is_completed: true,
                    btn_text: "Completed".to_string(),
                    btn_color: UiTheme::TEXT_PRIMARY,
                    btn_border: UiTheme::TEXT_PRIMARY,
                });
            }
            research_data.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }

    research_data
}

// ============================================================================
// Spawn Research Content (for embedding in village UI)
// ============================================================================

/// Spawns the research content (tabs + research list) into a parent container.
/// This does NOT include the outer panel or header.
pub fn spawn_research_content(parent: &mut ChildSpawnerCommands, data: ResearchData) {
    // Create a container for the research content
    let research_root = parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ResearchUiRoot {
                active_tab: data.active_tab,
            },
        ))
        .id();

    parent
        .commands()
        .entity(research_root)
        .with_children(|content| {
            // Tab bar
            let tab_bar = spawn_tab_bar(content);
            content.commands().entity(tab_bar).with_children(|tabs| {
                spawn_tab_button(
                    tabs,
                    "Available",
                    data.active_tab == ResearchTab::Available,
                    ResearchTabButton {
                        tab: ResearchTab::Available,
                    },
                );
                spawn_tab_button(
                    tabs,
                    "Completed",
                    data.active_tab == ResearchTab::Completed,
                    ResearchTabButton {
                        tab: ResearchTab::Completed,
                    },
                );
            });

            // Scrollable container for research items
            spawn_scrollable_container(content, ResearchItemsContainer);
        });

    // Populate with initial research (queue command)
    parent.commands().queue(PopulateResearchDirectCommand {
        research_data: data
            .items
            .into_iter()
            .map(|r| {
                (
                    r.id,
                    r.name,
                    r.description,
                    r.time,
                    r.cost_str,
                    r.can_afford,
                    r.is_completed,
                    r.btn_text,
                    r.btn_color,
                    r.btn_border,
                )
            })
            .collect(),
    });
}

// ============================================================================
// Close Button Handler
// ============================================================================

fn handle_research_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ResearchCloseButton>)>,
    ui_query: Query<Entity, With<ResearchUiRoot>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for ui_entity in ui_query.iter() {
                commands.entity(ui_entity).despawn();
            }
        }
    }
}

// ============================================================================
// Tab Switch Handler
// ============================================================================

#[allow(clippy::type_complexity)]
fn handle_tab_switch(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &ResearchTabButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ui_query: Query<&mut ResearchUiRoot>,
    mut tab_buttons: Query<(&ResearchTabButton, &mut BackgroundColor)>,
    assets: Res<Assets<ResearchDefinition>>,
    wallet: Res<Wallet>,
    available_query: Query<(Entity, &ResearchNode), With<Available>>,
    in_progress_query: Query<(Entity, &ResearchNode, &InProgress)>,
    completed_query: Query<(Entity, &ResearchNode), With<Completed>>,
) {
    for (interaction, tab_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut ui_root) = ui_query.single_mut() {
                if ui_root.active_tab != tab_btn.tab {
                    ui_root.active_tab = tab_btn.tab;

                    // Update tab button styling
                    for (btn, mut bg_color) in tab_buttons.iter_mut() {
                        if btn.tab == ui_root.active_tab {
                            *bg_color = BackgroundColor(UiTheme::TAB_ACTIVE_BG);
                        } else {
                            *bg_color = BackgroundColor(UiTheme::TAB_INACTIVE_BG);
                        }
                    }

                    // Collect query results
                    let available: Vec<_> = available_query.iter().collect();
                    let in_progress: Vec<_> = in_progress_query.iter().collect();
                    let completed: Vec<_> = completed_query.iter().collect();

                    // Repopulate research
                    let items = build_research_list(
                        &assets,
                        &wallet,
                        ui_root.active_tab,
                        &available,
                        &in_progress,
                        &completed,
                    );
                    commands.queue(PopulateResearchDirectCommand {
                        research_data: items
                            .into_iter()
                            .map(|r| {
                                (
                                    r.id,
                                    r.name,
                                    r.description,
                                    r.time,
                                    r.cost_str,
                                    r.can_afford,
                                    r.is_completed,
                                    r.btn_text,
                                    r.btn_color,
                                    r.btn_border,
                                )
                            })
                            .collect(),
                    });
                }
            }
        }
    }
}

// ============================================================================
// Update Research UI (on resource/state change)
// ============================================================================

fn update_research_ui(
    mut commands: Commands,
    assets: Res<Assets<ResearchDefinition>>,
    wallet: Res<Wallet>,
    research_map: Res<ResearchMap>,
    ui_query: Query<&ResearchUiRoot>,
    available_query: Query<(Entity, &ResearchNode), With<Available>>,
    in_progress_query: Query<(Entity, &ResearchNode, &InProgress)>,
    completed_query: Query<(Entity, &ResearchNode), With<Completed>>,
) {
    // Only update if wallet or research map changed
    if !wallet.is_changed() && !research_map.is_changed() {
        return;
    }

    if let Ok(ui_root) = ui_query.single() {
        let available: Vec<_> = available_query.iter().collect();
        let in_progress: Vec<_> = in_progress_query.iter().collect();
        let completed: Vec<_> = completed_query.iter().collect();

        let items = build_research_list(
            &assets,
            &wallet,
            ui_root.active_tab,
            &available,
            &in_progress,
            &completed,
        );
        commands.queue(PopulateResearchDirectCommand {
            research_data: items
                .into_iter()
                .map(|r| {
                    (
                        r.id,
                        r.name,
                        r.description,
                        r.time,
                        r.cost_str,
                        r.can_afford,
                        r.is_completed,
                        r.btn_text,
                        r.btn_color,
                        r.btn_border,
                    )
                })
                .collect(),
        });
    }
}

// ============================================================================
// Populate Research Command
// ============================================================================

/// Command to populate research (deferred execution)
struct PopulateResearchDirectCommand {
    #[allow(clippy::type_complexity)]
    research_data: Vec<(
        String,
        String,
        String,
        f32,
        String,
        bool,
        bool,
        String,
        Color,
        Color,
    )>,
}

impl Command for PopulateResearchDirectCommand {
    fn apply(self, world: &mut World) {
        let mut container_query = world
            .query_filtered::<(Entity, Option<&Children>), With<ResearchItemsContainer>>();

        let Some((container_entity, children)) = container_query.iter(world).next() else {
            return;
        };

        let children_to_despawn: Vec<Entity> =
            children.map(|c| c.iter().collect()).unwrap_or_default();

        for child in children_to_despawn {
            world.commands().entity(child).despawn();
        }

        world
            .commands()
            .entity(container_entity)
            .with_children(|parent| {
                for (
                    id,
                    name,
                    description,
                    time,
                    cost_str,
                    can_afford,
                    is_completed,
                    btn_text,
                    btn_color,
                    btn_border,
                ) in self.research_data
                {
                    let card_entity = widgets::spawn_item_card(parent, ());
                    parent
                        .commands()
                        .entity(card_entity)
                        .with_children(|card| {
                            spawn_card_title(card, &name);
                            spawn_description_text(card, &description);

                            if !is_completed {
                                spawn_timer_text(card, time);

                                if !cost_str.is_empty() {
                                    card.spawn((
                                        Text::new(cost_str),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(if can_afford {
                                            UiTheme::AFFORDABLE
                                        } else {
                                            UiTheme::NOT_AFFORDABLE
                                        }),
                                    ));
                                }
                            }

                            spawn_action_button(
                                card,
                                &btn_text,
                                btn_color,
                                btn_border,
                                ResearchButton { id: id.clone() },
                            );
                        });
                }
            });
    }
}

// ============================================================================
// Research Button Handler
// ============================================================================

#[allow(clippy::type_complexity)]
fn handle_research_button(
    mut events: MessageWriter<StartResearchRequest>,
    research_map: Res<ResearchMap>,
    assets: Res<Assets<ResearchDefinition>>,
    wallet: Res<Wallet>,
    available_query: Query<&ResearchNode, With<Available>>,
    interaction_query: Query<(&Interaction, &ResearchButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let id = &btn.id;

            // Check if research is available
            let Some(&entity) = research_map.entities.get(id) else {
                continue;
            };

            let Ok(node) = available_query.get(entity) else {
                continue; // Not available (locked, in progress, or completed)
            };

            let Some(def) = assets.get(&node.handle) else {
                continue;
            };

            // Check if can afford
            let can_afford = def
                .cost
                .iter()
                .all(|(res, amt)| wallet.resources.get(res).copied().unwrap_or(0) >= *amt);

            if can_afford {
                events.write(StartResearchRequest(id.clone()));
            }
        }
    }
}
