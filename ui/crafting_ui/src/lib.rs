use {
    bevy::prelude::*,
    crafting_events::StartCraftingRequest,
    crafting_resources::{RecipeCategory, RecipesLibrary},
    research::ResearchState,
    states::GameState,
    wallet::Wallet,
    widgets::{
        spawn_action_button, spawn_card_title, spawn_cost_text,
        spawn_scrollable_container, spawn_tab_bar,
        spawn_tab_button, spawn_timer_text, UiTheme,
    },
};

pub struct CraftingUiPlugin;

impl Plugin for CraftingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
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

/// Root of the recipes popup card
#[derive(Component)]
pub struct RecipesUiRoot {
    pub active_tab: RecipeCategory,
}

/// Close button marker
#[derive(Component)]
pub struct RecipesCloseButton;

/// Tab button with category
#[derive(Component)]
pub struct RecipeTabButton {
    pub category: RecipeCategory,
}

/// Container for recipe cards
#[derive(Component)]
pub struct RecipesItemsContainer;

/// Crafting action button
#[derive(Component)]
pub struct CraftingButton {
    pub recipe_id: String,
}

// ============================================================================
// Crafting Data Builder (for external use)
// ============================================================================

/// Data needed to display crafting content
pub struct CraftingData {
    pub active_tab: RecipeCategory,
    pub recipes: Vec<RecipeDisplayData>,
}

/// Display data for a single recipe
#[derive(PartialEq, Clone, Debug)]
pub struct RecipeDisplayData {
    pub id: String,
    pub display_name: String,
    pub craft_time: f32,
    pub cost_str: String,
    pub can_afford: bool,
}

/// Builds crafting display data from game resources
pub fn build_crafting_data(
    library: &RecipesLibrary,
    wallet: &Wallet,
    research_state: &ResearchState,
) -> CraftingData {
    let active_tab = RecipeCategory::Weapons;
    let recipes = build_recipe_list(library, wallet, research_state, &active_tab);
    CraftingData { active_tab, recipes }
}

fn build_recipe_list(
    library: &RecipesLibrary,
    wallet: &Wallet,
    research_state: &ResearchState,
    category: &RecipeCategory,
) -> Vec<RecipeDisplayData> {
    library
        .recipes
        .iter()
        .filter(|(_, recipe)| {
            recipe.category == *category
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

            RecipeDisplayData {
                id: id.clone(),
                display_name: recipe.display_name.clone(),
                craft_time: recipe.craft_time,
                cost_str,
                can_afford,
            }
        })
        .collect()
}

// ============================================================================
// Spawn Crafting Content (for embedding in village UI)
// ============================================================================

/// Spawns the crafting content (tabs + recipe list) into a parent container.
/// This does NOT include the outer panel or header.
pub fn spawn_crafting_content(parent: &mut ChildSpawnerCommands, data: CraftingData) {
    // Create a container for the crafting content
    let crafting_root = parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            RecipesUiRoot {
                active_tab: data.active_tab.clone(),
            },
        ))
        .id();

    parent.commands().entity(crafting_root).with_children(|content| {
        // Tab bar
        let tab_bar = spawn_tab_bar(content);
        content.commands().entity(tab_bar).with_children(|tabs| {
            spawn_tab_button(
                tabs,
                "Weapons",
                data.active_tab == RecipeCategory::Weapons,
                RecipeTabButton {
                    category: RecipeCategory::Weapons,
                },
            );
            spawn_tab_button(
                tabs,
                "Idols",
                data.active_tab == RecipeCategory::Idols,
                RecipeTabButton {
                    category: RecipeCategory::Idols,
                },
            );
        });

        // Scrollable container for recipe items
        spawn_scrollable_container(content, RecipesItemsContainer);
    });

    // Populate with initial recipes (queue command)
    parent.commands().queue(PopulateRecipesDirectCommand {
        recipes_data: data
            .recipes
            .into_iter()
            .map(|r| (r.id, r.display_name, r.craft_time, r.cost_str, r.can_afford))
            .collect(),
    });
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
                    let recipes = build_recipe_list(
                        &library,
                        &wallet,
                        &research_state,
                        &tab_btn.category,
                    );
                    commands.queue(PopulateRecipesDirectCommand {
                        recipes_data: recipes
                            .into_iter()
                            .map(|r| (r.id, r.display_name, r.craft_time, r.cost_str, r.can_afford))
                            .collect(),
                    });
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
    mut last_data: Local<Vec<RecipeDisplayData>>,
) {
    if let Ok(ui_root) = ui_query.single() {
        let recipes = build_recipe_list(&library, &wallet, &research_state, &ui_root.active_tab);
        
        if *last_data == recipes {
            return;
        }
        *last_data = recipes.clone();
        
        commands.queue(PopulateRecipesDirectCommand {
            recipes_data: recipes
                .into_iter()
                .map(|r| (r.id, r.display_name, r.craft_time, r.cost_str, r.can_afford))
                .collect(),
        });
    }
}

// ============================================================================
// Populate Recipes Command
// ============================================================================

/// Command to populate recipes (deferred execution)
struct PopulateRecipesDirectCommand {
    recipes_data: Vec<(String, String, f32, String, bool)>,
}

impl Command for PopulateRecipesDirectCommand {
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
                let card_entity = widgets::spawn_item_card(parent, ());
                parent.commands().entity(card_entity).with_children(|card| {
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
