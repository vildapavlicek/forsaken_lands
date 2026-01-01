use {
    bevy::prelude::*,
    crafting_resources::RecipesLibrary,
    research::ResearchState,
    states::GameState,
    wallet::Wallet,
};

pub struct CraftingUiPlugin;

impl Plugin for CraftingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Running), setup_crafting_ui)
            .add_systems(
                Update,
                (
                    update_crafting_ui.run_if(
                        resource_changed::<RecipesLibrary>
                            .or(resource_changed::<Wallet>)
                            .or(resource_changed::<ResearchState>),
                    ),
                    handle_crafting_button,
                )
                    .run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Component)]
struct CraftingUiRoot;

#[derive(Component)]
struct CraftingItemsContainer;

#[derive(Component)]
struct CraftingButton {
    recipe_id: String,
}

fn setup_crafting_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(300.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            CraftingUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Crafting"),
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

            // Scrollable area for crafting items
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    flex_grow: 1.0,
                    ..default()
                },
                CraftingItemsContainer,
            ));
        });
}

fn update_crafting_ui(
    mut commands: Commands,
    library: Res<RecipesLibrary>,
    wallet: Res<Wallet>,
    research_state: Res<ResearchState>,
    container_query: Query<(Entity, Option<&Children>), With<CraftingItemsContainer>>,
) {
    let Ok((container_entity, children)) = container_query.single() else {
        return;
    };

    // Clear existing items
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let mut sorted_recipes: Vec<_> = library.recipes.iter().collect();
    sorted_recipes.sort_by_key(|(id, _)| *id);

    commands.entity(container_entity).with_children(|parent| {
        for (id, recipe) in sorted_recipes {
            // Check research requirements
            if let Some(req) = &recipe.required_research {
                if !research_state.is_researched(req) {
                    continue;
                }
            }

            let mut can_afford = true;
            let mut cost_str = String::from("Cost: ");
            
            let mut cost_items: Vec<_> = recipe.cost.iter().collect();
            cost_items.sort_by_key(|(res_id, _)| *res_id);

            for (res_id, amt) in cost_items {
                let current = wallet.resources.get(res_id).copied().unwrap_or(0);
                cost_str.push_str(&format!("{}: {}/{} ", res_id, current, amt));
                if current < *amt {
                    can_afford = false;
                }
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
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(&recipe.display_name),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                    ));

                    card.spawn((
                        Text::new(&format!("Time: {}s", recipe.craft_time)),
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

                    // Button
                    let (btn_text, btn_color, btn_border) = if can_afford {
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
                        CraftingButton { recipe_id: id.clone() },
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

fn handle_crafting_button(
    mut wallet: ResMut<Wallet>,
    library: Res<RecipesLibrary>,
    interaction_query: Query<(&Interaction, &CraftingButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(recipe) = library.recipes.get(&btn.recipe_id) {
                // Check if can afford again (safety)
                let can_afford = recipe.cost.iter().all(|(res_id, amt)| {
                    wallet.resources.get(res_id).copied().unwrap_or(0) >= *amt
                });

                if can_afford {
                    // Subtract resources
                    for (res_id, amt) in &recipe.cost {
                        if let Some(current) = wallet.resources.get_mut(res_id) {
                            *current -= *amt;
                        }
                    }
                    info!("Crafting started for: {} (Resources subtracted)", recipe.display_name);
                    // TODO: Trigger actual crafting timer/spawning if needed
                }
            }
        }
    }
}
