use {
    bevy::{picking::prelude::*, prelude::*},
    crafting_resources::RecipesLibrary,
    research::{ResearchLibrary, ResearchState},
    states::{EnemyEncyclopediaState, GameState},
    village_components::{EnemyEncyclopedia, Village},
    wallet::Wallet,
    widgets::{
        spawn_menu_button, spawn_panel_header_with_close, spawn_ui_panel, PanelPosition,
    },
};

pub struct VillageUiPlugin;

impl Plugin for VillageUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_village_clicked)
            .add_systems(
                Update,
                (
                    handle_menu_button,
                    handle_back_button,
                    handle_close_button,
                )
                    .run_if(in_state(GameState::Running)),
            );
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
    existing_ui: Query<Entity, With<VillageUiRoot>>,
) {
    // Verify this is a village entity
    let clicked_entity = trigger.entity;
    if village_query.get(clicked_entity).is_err() {
        return;
    }

    // Toggle: if UI exists, close it; otherwise open
    if let Ok(ui_entity) = existing_ui.single() {
        commands.entity(ui_entity).despawn();
        return;
    }

    spawn_village_ui(&mut commands);
}

// ============================================================================
// Spawn Village UI
// ============================================================================

fn spawn_village_ui(commands: &mut Commands) {
    let panel = spawn_ui_panel(
        commands,
        PanelPosition::CenterPopup { top: 50.0 },
        350.0,
        Val::Px(400.0),
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
        world
            .commands()
            .entity(container)
            .with_children(|parent| {
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

        // Get resources needed for crafting content
        let library = world.resource::<RecipesLibrary>();
        let wallet = world.resource::<Wallet>();
        let research_state = world.resource::<ResearchState>();

        // Build crafting data
        let crafting_data = crafting_ui::build_crafting_data(library, wallet, research_state);

        // Spawn back button and crafting content
        world
            .commands()
            .entity(container)
            .with_children(|parent| {
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

        // Get resources needed for research content
        let library = world.resource::<ResearchLibrary>();
        let state = world.resource::<ResearchState>();
        let wallet = world.resource::<Wallet>();

        // Build research data
        let research_data = research_ui::build_research_data(library, state, wallet);

        // Spawn back button and research content
        world
            .commands()
            .entity(container)
            .with_children(|parent| {
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
        world
            .commands()
            .entity(container)
            .with_children(|parent| {
                // Back button
                spawn_menu_button(parent, "← Back", VillageBackButton);

                // Spawn encyclopedia content
                enemy_encyclopedia::spawn_enemy_encyclopedia_content(parent, &encyclopedia);
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
    ui_query: Query<Entity, With<VillageUiRoot>>,
    mut next_state: ResMut<NextState<EnemyEncyclopediaState>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for ui_entity in ui_query.iter() {
                next_state.set(EnemyEncyclopediaState::Closed);
                commands.entity(ui_entity).despawn();
            }
        }
    }
}
