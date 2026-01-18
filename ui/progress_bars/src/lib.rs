use {
    bevy::prelude::*,
    crafting::CraftingInProgress,
    research::{InProgress, ResearchNode},
    research_assets::ResearchDefinition,
    states::GameState,
    village_components::Village,
};

// ============================================================================
// Plugin
// ============================================================================

pub struct ProgressBarsPlugin;

impl Plugin for ProgressBarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_progress_bars.run_if(in_state(GameState::Running)),
        )
        .add_observer(on_research_started)
        .add_observer(on_research_ended)
        .add_observer(on_crafting_started)
        .add_systems(OnExit(GameState::Running), clean_up_progress_bars);
    }
}

// ============================================================================
// Components
// ============================================================================

/// Root container for progress bars, spawned as child of Village
#[derive(Component)]
pub struct ProgressBarsRoot;

/// Individual progress bar entry for research
#[derive(Component)]
pub struct ResearchProgressBar {
    pub research_entity: Entity,
}

/// Individual progress bar entry for crafting
#[derive(Component)]
pub struct CraftingProgressBar {
    pub crafting_entity: Entity,
}

/// The fill portion of a progress bar
#[derive(Component)]
struct ProgressBarFill;

/// Text showing time remaining
#[derive(Component)]
struct ProgressBarText;

// ============================================================================
// Constants
// ============================================================================

const BAR_WIDTH: f32 = 80.0;
const BAR_HEIGHT: f32 = 8.0;
const BAR_Y_OFFSET: f32 = -28.0;
const BAR_SPACING: f32 = 14.0;
const BAR_BG_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
const BAR_FILL_RESEARCH: Color = Color::srgba(0.2, 0.6, 1.0, 1.0);
const BAR_FILL_CRAFTING: Color = Color::srgba(1.0, 0.6, 0.2, 1.0);

// ============================================================================
// Systems - Spawn/Despawn
// ============================================================================

/// Observer: When InProgress is added to a research entity, spawn a progress bar
fn on_research_started(
    trigger: On<Add, InProgress>,
    mut commands: Commands,
    village_query: Query<Entity, With<Village>>,
    root_query: Query<Entity, With<ProgressBarsRoot>>,
    research_query: Query<&ResearchNode>,
    research_bars: Query<&ResearchProgressBar>,
    crafting_bars: Query<&CraftingProgressBar>,
    assets: Res<Assets<ResearchDefinition>>,
) {
    let research_entity = trigger.event().entity;

    // Get research name
    let Ok(node) = research_query.get(research_entity) else {
        return;
    };
    let name = assets
        .get(&node.handle)
        .map(|d| d.name.clone())
        .unwrap_or_else(|| node.id.clone());

    // Get or spawn root
    let root_entity = if let Ok(root) = root_query.single() {
        root
    } else {
        let Ok(village) = village_query.single() else {
            return;
        };
        spawn_progress_root(&mut commands, village)
    };

    // Count existing bars for vertical offset
    let bar_index = research_bars.iter().count() + crafting_bars.iter().count();

    spawn_progress_bar(
        &mut commands,
        root_entity,
        &name,
        bar_index,
        BAR_FILL_RESEARCH,
        ResearchProgressBar { research_entity },
    );
}

/// Observer: When InProgress is removed from a research entity, despawn its bar
fn on_research_ended(
    trigger: On<Remove, InProgress>,
    mut commands: Commands,
    bar_query: Query<(Entity, &ResearchProgressBar)>,
) {
    let research_entity = trigger.event().entity;

    // Find and despawn the matching bar
    for (bar_entity, bar) in bar_query.iter() {
        if bar.research_entity == research_entity {
            commands.entity(bar_entity).despawn();
            break;
        }
    }

    // Schedule cleanup check
    commands.queue(CleanupEmptyRootCommand);
}

/// Observer: When CraftingInProgress is spawned, add a progress bar
fn on_crafting_started(
    trigger: On<Add, CraftingInProgress>,
    mut commands: Commands,
    village_query: Query<Entity, With<Village>>,
    root_query: Query<Entity, With<ProgressBarsRoot>>,
    crafting_query: Query<&CraftingInProgress>,
    research_bars: Query<&ResearchProgressBar>,
    crafting_bars: Query<&CraftingProgressBar>,
) {
    let crafting_entity = trigger.event().entity;

    // Get recipe name
    let Ok(crafting) = crafting_query.get(crafting_entity) else {
        return;
    };
    let name = format!("Crafting: {}", crafting.recipe_id);

    // Get or spawn root
    let root_entity = if let Ok(root) = root_query.single() {
        root
    } else {
        let Ok(village) = village_query.single() else {
            return;
        };
        spawn_progress_root(&mut commands, village)
    };

    // Count existing bars for vertical offset
    let bar_index = research_bars.iter().count() + crafting_bars.iter().count();

    spawn_progress_bar(
        &mut commands,
        root_entity,
        &name,
        bar_index,
        BAR_FILL_CRAFTING,
        CraftingProgressBar { crafting_entity },
    );
}

// ============================================================================
// Systems - Update
// ============================================================================

/// Update progress bar fills and text based on timer progress
#[allow(clippy::type_complexity)]
fn update_progress_bars(
    mut commands: Commands,
    research_query: Query<&InProgress>,
    crafting_query: Query<&CraftingInProgress>,
    research_bars: Query<(Entity, &ResearchProgressBar, &Children)>,
    crafting_bars: Query<(Entity, &CraftingProgressBar, &Children)>,
    mut fill_query: Query<&mut Transform, With<ProgressBarFill>>,
    mut text_query: Query<&mut Text2d, With<ProgressBarText>>,
    root_query: Query<Entity, With<ProgressBarsRoot>>,
) {
    // Update research bars
    for (bar_entity, bar, children) in research_bars.iter() {
        if let Ok(progress) = research_query.get(bar.research_entity) {
            let fraction = progress.timer.fraction();
            let remaining = progress.timer.remaining_secs();
            update_bar_visuals(
                children,
                fraction,
                remaining,
                &mut fill_query,
                &mut text_query,
            );
        } else {
            // Research no longer in progress, despawn bar
            commands.entity(bar_entity).despawn();
        }
    }

    // Update crafting bars
    for (bar_entity, bar, children) in crafting_bars.iter() {
        if let Ok(crafting) = crafting_query.get(bar.crafting_entity) {
            let fraction = crafting.timer.fraction();
            let remaining = crafting.timer.remaining_secs();
            update_bar_visuals(
                children,
                fraction,
                remaining,
                &mut fill_query,
                &mut text_query,
            );
        } else {
            // Crafting entity despawned, remove bar
            commands.entity(bar_entity).despawn();
        }
    }

    // Cleanup empty root
    if research_bars.is_empty() && crafting_bars.is_empty() {
        if let Ok(root) = root_query.single() {
            commands.entity(root).despawn();
        }
    }
}

fn update_bar_visuals(
    children: &Children,
    fraction: f32,
    remaining: f32,
    fill_query: &mut Query<&mut Transform, With<ProgressBarFill>>,
    text_query: &mut Query<&mut Text2d, With<ProgressBarText>>,
) {
    for child in children.iter() {
        if let Ok(mut transform) = fill_query.get_mut(child) {
            // Scale the fill bar based on progress
            transform.scale.x = fraction;
            // Adjust position to keep it left-aligned
            transform.translation.x = (fraction - 1.0) * BAR_WIDTH / 2.0;
        }
        if let Ok(mut text) = text_query.get_mut(child) {
            text.0 = format!("{:.1}s", remaining);
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn spawn_progress_root(commands: &mut Commands, village: Entity) -> Entity {
    let root = commands
        .spawn((
            ProgressBarsRoot,
            Name::new("ProgressBarsRoot"),
            Transform::from_translation(Vec3::new(0.0, BAR_Y_OFFSET, 10.0)),
            Visibility::Inherited,
        ))
        .id();

    commands.entity(village).add_child(root);
    root
}

fn spawn_progress_bar<M: Component>(
    commands: &mut Commands,
    root: Entity,
    name: &str,
    index: usize,
    fill_color: Color,
    marker: M,
) {
    let y_offset = -(index as f32) * BAR_SPACING;

    let bar = commands
        .spawn((
            marker,
            // Name for filtering in save/load (prevents zombie bars)
            Name::new("ProgressBar"),
            Transform::from_translation(Vec3::new(0.0, y_offset, 0.0)),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            // Background
            parent.spawn((
                Sprite {
                    color: BAR_BG_COLOR,
                    custom_size: Some(Vec2::new(BAR_WIDTH, BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));

            // Fill
            parent.spawn((
                ProgressBarFill,
                Sprite {
                    color: fill_color,
                    custom_size: Some(Vec2::new(BAR_WIDTH, BAR_HEIGHT - 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
            ));

            // Name text (above bar)
            parent.spawn((
                Text2d::new(name),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, BAR_HEIGHT + 2.0, 0.1)),
            ));

            // Time text (on bar)
            parent.spawn((
                ProgressBarText,
                Text2d::new(""),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
            ));
        })
        .id();

    commands.entity(root).add_child(bar);
}

/// Command to cleanup empty root on next frame
struct CleanupEmptyRootCommand;

impl Command for CleanupEmptyRootCommand {
    fn apply(self, world: &mut World) {
        let mut root_query =
            world.query_filtered::<(Entity, Option<&Children>), With<ProgressBarsRoot>>();

        if let Some((root, children)) = root_query.iter(world).next() {
            let is_empty = children.is_none_or(|c| c.is_empty());
            if is_empty {
                world.commands().entity(root).despawn();
            }
        }
    }
}
pub fn clean_up_progress_bars(mut commands: Commands, query: Query<Entity, With<ProgressBarsRoot>>) {
    debug!("Cleaning up progress bars");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
