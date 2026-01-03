use {
    bevy::prelude::*,
    crafting_events::StartCraftingRequest,
    crafting_resources::{RecipeCategory, RecipesLibrary},
    research::ResearchState,
    states::GameState,
    wallet::Wallet,
    widgets::{
        spawn_action_button, spawn_card_title, spawn_cost_text, spawn_icon_button,
        spawn_panel_header_with_close, spawn_scrollable_container, spawn_tab_bar,
        spawn_tab_button, spawn_timer_text, spawn_ui_panel, PanelPosition, UiTheme,
    },
};

pub struct CraftingUiPlugin;

impl Plugin for CraftingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Running), setup_recipes_spawn_button)
            .add_systems(
                Update,
                (
                    handle_open_recipes_button,
                    handle_recipes_close_button,
                    handle_tab_switch,
                    update_recipes_ui.run_if(
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

// ============================================================================
// Components
// ============================================================================

/// Marker for the "Open Recipes" button
#[derive(Component)]
struct OpenRecipesButton;

/// Root of the recipes popup card
#[derive(Component)]
struct RecipesUiRoot {
    active_tab: RecipeCategory,
}

/// Close button marker
#[derive(Component)]
struct RecipesCloseButton;

/// Tab button with category
#[derive(Component)]
struct RecipeTabButton {
    category: RecipeCategory,
}

/// Container for recipe cards
#[derive(Component)]
struct RecipesItemsContainer;

/// Crafting action button
#[derive(Component)]
struct CraftingButton {
    recipe_id: String,
}

// ============================================================================
// Spawn Button Setup
// ============================================================================

fn setup_recipes_spawn_button(mut commands: Commands) {
    // Spawn a button bar container at bottom-left for icon buttons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            spawn_icon_button(parent, "⚒", OpenRecipesButton);
        });
}

// ============================================================================
// Open Recipes Button Handler
// ============================================================================

fn handle_open_recipes_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<OpenRecipesButton>)>,
    existing_ui: Query<Entity, With<RecipesUiRoot>>,
    library: Res<RecipesLibrary>,
    wallet: Res<Wallet>,
    research_state: Res<ResearchState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Toggle: if UI exists, close it; otherwise open
            if let Ok(ui_entity) = existing_ui.single() {
                commands.entity(ui_entity).despawn();
                return;
            }

            // Spawn the recipes popup
            spawn_recipes_popup(&mut commands, &library, &wallet, &research_state);
        }
    }
}

fn spawn_recipes_popup(
    commands: &mut Commands,
    library: &RecipesLibrary,
    wallet: &Wallet,
    research_state: &ResearchState,
) {
    let active_tab = RecipeCategory::Weapons;

    let panel = spawn_ui_panel(
        commands,
        PanelPosition::CenterPopup { top: 50.0 },
        400.0,
        Val::Px(500.0),
        RecipesUiRoot {
            active_tab: active_tab.clone(),
        },
    );

    commands.entity(panel).with_children(|parent| {
        // Header with close button
        spawn_panel_header_with_close(parent, "Recipes", RecipesCloseButton);

        // Tab bar
        let tab_bar = spawn_tab_bar(parent);
        parent.commands().entity(tab_bar).with_children(|tabs| {
            spawn_tab_button(
                tabs,
                "Weapons",
                active_tab == RecipeCategory::Weapons,
                RecipeTabButton {
                    category: RecipeCategory::Weapons,
                },
            );
            spawn_tab_button(
                tabs,
                "Idols",
                active_tab == RecipeCategory::Idols,
                RecipeTabButton {
                    category: RecipeCategory::Idols,
                },
            );
        });

        // Scrollable container for recipe items
        spawn_scrollable_container(parent, RecipesItemsContainer);
    });

    // Populate with initial recipes
    populate_recipes(commands, library, wallet, research_state, &active_tab);
}

// ============================================================================
// Close Button Handler
// ============================================================================

fn handle_recipes_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RecipesCloseButton>)>,
    ui_query: Query<Entity, With<RecipesUiRoot>>,
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
        (&Interaction, &RecipeTabButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ui_query: Query<&mut RecipesUiRoot>,
    mut tab_buttons: Query<(&RecipeTabButton, &mut BackgroundColor)>,
    library: Res<RecipesLibrary>,
    wallet: Res<Wallet>,
    research_state: Res<ResearchState>,
) {
    for (interaction, tab_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Update active tab
            if let Ok(mut ui_root) = ui_query.single_mut() {
                if ui_root.active_tab != tab_btn.category {
                    ui_root.active_tab = tab_btn.category.clone();

                    // Update tab button styling
                    for (btn, mut bg_color) in tab_buttons.iter_mut() {
                        if btn.category == ui_root.active_tab {
                            *bg_color = BackgroundColor(UiTheme::TAB_ACTIVE_BG);
                        } else {
                            *bg_color = BackgroundColor(UiTheme::TAB_INACTIVE_BG);
                        }
                    }

                    // Repopulate recipes
                    populate_recipes(
                        &mut commands,
                        &library,
                        &wallet,
                        &research_state,
                        &tab_btn.category,
                    );
                }
            }
        }
    }
}

// ============================================================================
// Update Recipes UI (on resource change)
// ============================================================================

fn update_recipes_ui(
    mut commands: Commands,
    library: Res<RecipesLibrary>,
    wallet: Res<Wallet>,
    research_state: Res<ResearchState>,
    ui_query: Query<&RecipesUiRoot>,
) {
    if let Ok(ui_root) = ui_query.single() {
        populate_recipes(
            &mut commands,
            &library,
            &wallet,
            &research_state,
            &ui_root.active_tab,
        );
    }
}

// ============================================================================
// Populate Recipes Helper
// ============================================================================

fn populate_recipes(
    commands: &mut Commands,
    library: &RecipesLibrary,
    wallet: &Wallet,
    research_state: &ResearchState,
    active_category: &RecipeCategory,
) {
    // We need to query the container entity
    // This is tricky because we're in a helper function, so we'll use a system parameter approach
    // Instead, let's schedule this as a command
    let category = active_category.clone();
    let recipes_data: Vec<_> = library
        .recipes
        .iter()
        .filter(|(_, recipe)| {
            // Filter by category
            recipe.category == category
                // Check research requirements
                && recipe
                    .required_research
                    .as_ref()
                    .is_none_or(|req| research_state.is_researched(req))
        })
        .map(|(id, recipe)| {
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

            (
                id.clone(),
                recipe.display_name.clone(),
                recipe.craft_time,
                cost_str,
                can_afford,
            )
        })
        .collect();

    commands.queue(PopulateRecipesCommand { recipes_data });
}

/// Command to populate recipes (deferred execution)
struct PopulateRecipesCommand {
    recipes_data: Vec<(String, String, f32, String, bool)>,
}

impl Command for PopulateRecipesCommand {
    fn apply(self, world: &mut World) {
        // Find the container entity
        let mut container_query = world.query_filtered::<(Entity, Option<&Children>), With<RecipesItemsContainer>>();

        let Some((container_entity, children)) = container_query.iter(world).next() else {
            return;
        };

        // Collect children to despawn
        let children_to_despawn: Vec<Entity> = children
            .map(|c| c.iter().collect())
            .unwrap_or_default();

        // Despawn existing children
        for child in children_to_despawn {
            world.commands().entity(child).despawn();
        }

        // Spawn new recipe cards
        world.commands().entity(container_entity).with_children(|parent| {
            for (recipe_id, display_name, craft_time, cost_str, can_afford) in self.recipes_data {
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
                            ("Craft", UiTheme::AFFORDABLE, UiTheme::BORDER_SUCCESS)
                        } else {
                            ("Craft", UiTheme::BORDER_DISABLED, UiTheme::BORDER_DISABLED)
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
}

// ============================================================================
// Crafting Button Handler
// ============================================================================

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
