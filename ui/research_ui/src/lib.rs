use {
    bevy::{platform::collections::HashMap, prelude::*},
    research::{ResearchLibrary, ResearchState, StartResearchRequest},
    states::GameState,
    wallet::Wallet,
    widgets::{
        spawn_action_button, spawn_card_title, spawn_description_text, spawn_scrollable_container,
        spawn_timer_text, spawn_ui_panel, PanelPosition, UiTheme,
    },
};

pub struct ResearchUiPlugin;

impl Plugin for ResearchUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Running), setup_research_ui)
            .add_systems(
                Update,
                (
                    update_research_ui.run_if(
                        resource_changed::<ResearchState>
                            .or(resource_changed::<Wallet>)
                            .or(resource_changed::<ResearchLibrary>),
                    ),
                    handle_research_button,
                )
                    .run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Component)]
struct ResearchUiRoot;

#[derive(Component)]
struct ResearchButton {
    id: String,
}

#[derive(Component)]
struct ResearchCard {
    research_id: String,
}

#[derive(Component)]
struct ResearchCostText;

#[derive(Component)]
struct ResearchItemsContainer;

fn setup_research_ui(mut commands: Commands) {
    let panel = spawn_ui_panel(
        &mut commands,
        PanelPosition::Right(10.0),
        300.0,
        Val::Percent(90.0),
        ResearchUiRoot,
    );

    commands.entity(panel).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Research"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(UiTheme::TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // Scrollable container
        spawn_scrollable_container(parent, ResearchItemsContainer);
    });
}

/// Data for spawning a new research card
struct NewCardData {
    id: String,
    name: String,
    description: String,
    time_required: f32,
    cost_str: String,
    can_afford: bool,
    is_completed: bool,
    btn_text: &'static str,
    btn_color: Color,
    btn_border: Color,
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn update_research_ui(
    mut commands: Commands,
    library: Res<ResearchLibrary>,
    state: Res<ResearchState>,
    wallet: Res<Wallet>,
    container_query: Query<(Entity, Option<&Children>), With<ResearchItemsContainer>>,
    card_query: Query<(Entity, &ResearchCard, &Children)>,
    mut cost_text_query: Query<
        (&mut Text, &mut TextColor),
        (With<ResearchCostText>, Without<ResearchButton>),
    >,
    mut button_query: Query<
        (
            &mut BorderColor,
            &mut BackgroundColor,
            &Children,
            &ResearchButton,
        ),
        With<Button>,
    >,
    mut button_text_query: Query<
        (&mut Text, &mut TextColor),
        (Without<ResearchCostText>, Without<ResearchButton>),
    >,
) {
    let Ok((container_entity, children)) = container_query.single() else {
        return;
    };

    // 1. Map existing cards for diffing
    let mut existing_cards: HashMap<String, Entity> = HashMap::default();
    if let Some(children) = children {
        for child in children.iter() {
            if let Ok((_, card, _)) = card_query.get(child) {
                existing_cards.insert(card.research_id.clone(), child);
            }
        }
    }

    // 2. Sort available research by ID
    let mut sorted_techs: Vec<_> = library.available.iter().collect();
    sorted_techs.sort_by_key(|(_, def)| def.id);

    let mut sorted_entities = Vec::new();
    let mut new_cards_to_spawn: Vec<NewCardData> = Vec::new();

    for (id, def) in sorted_techs {
        // Prerequisites check
        let prereqs_met = def.prerequisites.iter().all(|p| state.is_researched(p));
        if !prereqs_met {
            continue;
        }

        let is_completed = state.is_researched(id);
        let is_researching = state.is_researching(id);

        let mut can_afford = true;
        let mut cost_str = String::from("Cost: ");
        for (res, amt) in &def.cost {
            let current = wallet.resources.get(res).copied().unwrap_or(0);
            cost_str.push_str(&format!("{}: {}/{} ", res, current, amt));
            if current < *amt {
                can_afford = false;
            }
        }

        let (btn_text_str, btn_color, btn_border) = if is_completed {
            ("Completed", UiTheme::TEXT_PRIMARY, UiTheme::TEXT_PRIMARY)
        } else if is_researching {
            (
                "Researching...",
                UiTheme::TEXT_INFO,
                Color::srgba(0.4, 0.4, 1.0, 1.0),
            )
        } else if can_afford {
            ("Start", UiTheme::AFFORDABLE, UiTheme::BORDER_SUCCESS)
        } else {
            ("Start", UiTheme::BORDER_DISABLED, UiTheme::BORDER_DISABLED)
        };

        if let Some(&entity) = existing_cards.get(id) {
            // Update Existing
            existing_cards.remove(id);

            if let Ok((_, _, children)) = card_query.get(entity) {
                for child in children.iter() {
                    // Try to update cost text
                    if let Ok((mut text, mut color)) = cost_text_query.get_mut(child) {
                        text.0 = cost_str.clone();
                        color.0 = if can_afford {
                            UiTheme::AFFORDABLE
                        } else {
                            UiTheme::NOT_AFFORDABLE
                        };
                    }

                    // Try to update button
                    if let Ok((mut border, _, btn_children, _)) = button_query.get_mut(child) {
                        *border = BorderColor::all(btn_border);

                        // Update button text
                        if let Some(&text_entity) = btn_children.first()
                            && let Ok((mut text, mut color)) =
                                button_text_query.get_mut(text_entity)
                        {
                            text.0 = btn_text_str.to_string();
                            color.0 = btn_color;
                        }
                    }
                }
            }
            sorted_entities.push(entity);
        } else {
            // Queue for spawning
            new_cards_to_spawn.push(NewCardData {
                id: id.clone(),
                name: def.name.clone(),
                description: def.description.clone(),
                time_required: def.time_required,
                cost_str,
                can_afford,
                is_completed,
                btn_text: btn_text_str,
                btn_color,
                btn_border,
            });
        }
    }

    // Spawn new cards
    for card_data in new_cards_to_spawn {
        let card_entity = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(UiTheme::CARD_BORDER),
                BackgroundColor(UiTheme::CARD_BG),
                ResearchCard {
                    research_id: card_data.id.clone(),
                },
            ))
            .with_children(|card| {
                spawn_card_title(card, &card_data.name);
                spawn_description_text(card, &card_data.description);

                if !card_data.is_completed {
                    spawn_timer_text(card, card_data.time_required);

                    // Cost text with marker
                    card.spawn((
                        Text::new(&card_data.cost_str),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(if card_data.can_afford {
                            UiTheme::AFFORDABLE
                        } else {
                            UiTheme::NOT_AFFORDABLE
                        }),
                        ResearchCostText,
                    ));
                }

                spawn_action_button(
                    card,
                    card_data.btn_text,
                    card_data.btn_color,
                    card_data.btn_border,
                    ResearchButton {
                        id: card_data.id.clone(),
                    },
                );
            })
            .id();

        sorted_entities.push(card_entity);
    }

    commands
        .entity(container_entity)
        .replace_children(&sorted_entities);

    // Despawn remaining
    for (_, entity) in existing_cards {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
fn handle_research_button(
    mut events: MessageWriter<StartResearchRequest>,
    library: Res<ResearchLibrary>,
    state: Res<ResearchState>,
    wallet: Res<Wallet>,
    interaction_query: Query<(&Interaction, &ResearchButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let id = &btn.id;

            if state.is_researched(id) || state.is_researching(id) {
                continue;
            }

            if let Some(def) = library.available.get(id) {
                let prereqs_met = def.prerequisites.iter().all(|p| state.is_researched(p));
                let can_afford = def
                    .cost
                    .iter()
                    .all(|(res, amt)| wallet.resources.get(res).copied().unwrap_or(0) >= *amt);

                if prereqs_met && can_afford {
                    events.write(StartResearchRequest(id.clone()));
                }
            }
        }
    }
}

