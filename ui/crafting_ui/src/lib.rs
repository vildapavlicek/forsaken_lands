use {
    bevy::prelude::*,
    crafting_events::StartCraftingRequest,
    crafting_resources::RecipesLibrary,
    research::ResearchState,
    states::GameState,
    wallet::Wallet,
    widgets::{
        spawn_action_button, spawn_card_title, spawn_cost_text, spawn_scrollable_container,
        spawn_timer_text, spawn_ui_panel, PanelPosition, UiTheme,
    },
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
    let panel = spawn_ui_panel(
        &mut commands,
        PanelPosition::Left(10.0),
        300.0,
        Val::Percent(90.0),
        CraftingUiRoot,
    );

    commands.entity(panel).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Crafting"),
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

        // Scrollable container for items
        spawn_scrollable_container(parent, CraftingItemsContainer);
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

    // Collect card data first to avoid borrow issues
    let mut cards_to_spawn: Vec<(String, String, f32, String, bool)> = Vec::new();

    for (id, recipe) in sorted_recipes {
        // Check research requirements
        if let Some(req) = &recipe.required_research
            && !research_state.is_researched(req)
        {
            continue;
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

        cards_to_spawn.push((
            id.clone(),
            recipe.display_name.clone(),
            recipe.craft_time,
            cost_str,
            can_afford,
        ));
    }

    // Now spawn all cards
    commands.entity(container_entity).with_children(|parent| {
        for (recipe_id, display_name, craft_time, cost_str, can_afford) in cards_to_spawn {
            parent
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
                ))
                .with_children(|card| {
                    spawn_card_title(card, &display_name);
                    spawn_timer_text(card, craft_time);
                    spawn_cost_text(card, &cost_str, can_afford);

                    // Button
                    let (btn_text, btn_color, btn_border) = if can_afford {
                        ("Start", UiTheme::AFFORDABLE, UiTheme::BORDER_SUCCESS)
                    } else {
                        ("Start", UiTheme::BORDER_DISABLED, UiTheme::BORDER_DISABLED)
                    };

                    spawn_action_button(
                        card,
                        btn_text,
                        btn_color,
                        btn_border,
                        CraftingButton {
                            recipe_id: recipe_id.clone(),
                        },
                    );
                });
        }
    });
}

#[allow(clippy::type_complexity)]
fn handle_crafting_button(
    mut commands: Commands,
    mut wallet: ResMut<Wallet>,
    library: Res<RecipesLibrary>,
    interaction_query: Query<(&Interaction, &CraftingButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Some(recipe) = library.recipes.get(&btn.recipe_id)
        {
            // Check if can afford
            let can_afford = recipe.cost.iter().all(|(res_id, amt)| {
                wallet.resources.get(res_id).copied().unwrap_or(0) >= *amt
            });

            if can_afford {
                // Deduct resources
                for (res_id, amt) in &recipe.cost {
                    if let Some(current) = wallet.resources.get_mut(res_id) {
                        *current -= *amt;
                    }
                }

                // Trigger the crafting request event (observer pattern)
                commands.trigger(StartCraftingRequest {
                    recipe_id: btn.recipe_id.clone(),
                });
                info!("Sent crafting request for: {}", recipe.display_name);
            }
        }
    }
}
