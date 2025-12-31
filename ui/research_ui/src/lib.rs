use {
    bevy::prelude::*,
    research::{ResearchLibrary, ResearchState, StartResearchRequest},
    states::GameState,
    wallet::Wallet,
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
struct ResearchCard;

fn setup_research_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(300.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            ResearchUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Research"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Scrollable area for research items
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    flex_grow: 1.0,
                    ..default()
                },
                ResearchItemsContainer,
            ));
        });
}

#[derive(Component)]
struct ResearchItemsContainer;

fn update_research_ui(
    mut commands: Commands,
    library: Res<ResearchLibrary>,
    state: Res<ResearchState>,
    wallet: Res<Wallet>,
    container_query: Query<(Entity, Option<&Children>), With<ResearchItemsContainer>>,
) {
    let Ok((container_entity, children)) = container_query.single() else {
        return;
    };

    // Simple approach: Clear and rebuild the list
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let mut sorted_techs: Vec<_> = library.available.iter().collect();
    sorted_techs.sort_by_key(|(id, _)| *id);

    commands.entity(container_entity).with_children(|parent| {
        for (id, def) in sorted_techs {
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

            // Prerequisites check
            let prereqs_met = def.prerequisites.iter().all(|p| state.is_researched(p));
            if !prereqs_met {
                continue; // Don't even show if prereqs not met? Or show as locked?
                // For now, let's just show it if it's in the library.
            }

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
                    ResearchCard,
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(&def.name),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                    ));

                    card.spawn((
                        Text::new(&def.description),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    ));

                    if !is_completed {
                        card.spawn((
                            Text::new(&format!("Time: {}s", def.time_required)),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 1.0, 1.0)),
                        ));

                        card.spawn((
                            Text::new(&cost_str),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(if can_afford {
                                Color::srgba(0.7, 1.0, 0.7, 1.0)
                            } else {
                                Color::srgba(1.0, 0.7, 0.7, 1.0)
                            }),
                        ));
                    }

                    // Button
                    let (btn_text, btn_color, btn_border) = if is_completed {
                        (
                            "Completed",
                            Color::srgba(1.0, 1.0, 1.0, 1.0),
                            Color::srgba(1.0, 1.0, 1.0, 1.0),
                        )
                    } else if is_researching {
                        (
                            "Researching...",
                            Color::srgba(0.7, 0.7, 1.0, 1.0),
                            Color::srgba(0.4, 0.4, 1.0, 1.0),
                        )
                    } else if can_afford {
                        (
                            "Start",
                            Color::srgba(0.5, 1.0, 0.5, 1.0),
                            Color::srgba(0.0, 1.0, 0.0, 1.0),
                        )
                    } else {
                        (
                            "Start",
                            Color::srgba(0.5, 0.5, 0.5, 1.0),
                            Color::srgba(0.5, 0.5, 0.5, 1.0),
                        )
                    };

                    card.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(30.0),
                            margin: UiRect::top(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(btn_border),
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                        ResearchButton { id: id.clone() },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(btn_text),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(btn_color),
                        ));
                    });
                });
        }
    });
}

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

            // Re-check conditions briefly before sending request
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
