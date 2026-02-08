use {
    bevy::{picking::prelude::*, prelude::*},
    buildings_components::TheMaw,
    hero_components::{AttackRange, AttackSpeed, Damage, Hero, MeleeArc, MeleeWeapon, Weapon},
    hero_ui::{HeroContentContainer, HeroUiRoot, spawn_hero_content},
    research::ResearchState,
    shared_components::DisplayName,
    skill_components::EquippedSkills,
    skills_assets::{SkillDefinition, SkillMap},
    states::{GameState, VillageView},
    village_components::Village,
    widgets::{
        ContentContainer, PanelWrapperRef, spawn_menu_button, spawn_menu_panel,
        spawn_panel_header_with_close,
    },
};

pub struct VillageUiPlugin;

impl Plugin for VillageUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<VillageView>()
            .add_observer(on_village_clicked)
            .add_systems(
                Update,
                (handle_menu_button, handle_back_button, handle_close_button)
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(OnEnter(VillageView::Menu), show_menu_content)
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
    Blessings,
}

/// Root of the village UI
#[derive(Component)]
pub struct VillageUiRoot {
    pub content: VillageContent,
}

/// Close button marker
#[derive(Component)]
struct VillageCloseButton;

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
    mut next_village_state: ResMut<NextState<VillageView>>,
) {
    // Verify this is a village entity
    let clicked_entity = trigger.entity;
    if village_query.get(clicked_entity).is_err() {
        return;
    }

    // Toggle: if UI exists, close it; otherwise open
    if let Ok((ui_entity, wrapper_ref)) = existing_ui.single() {
        next_village_state.set(VillageView::Closed);
        // Despawn wrapper if it exists, otherwise just despawn the panel
        if let Some(wrapper) = wrapper_ref {
            commands.entity(wrapper.0).despawn();
        } else {
            commands.entity(ui_entity).despawn();
        }
        return;
    }

    next_village_state.set(VillageView::Menu);
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
    // commands.queue(SpawnMenuContentCommand); // Handled by OnEnter(VillageView::Menu)
}

fn show_menu_content(mut commands: Commands) {
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

        // Check if The Maw exists to enable Blessings
        let maw_exists = world.query::<&TheMaw>().iter(world).next().is_some();

        // Check if simple crafting is researched
        let research_state = world.resource::<ResearchState>();
        let crafting_researched = research_state
            .completion_counts
            .get("simple_crafting")
            .is_some();

        // Spawn menu buttons
        world.commands().entity(container).with_children(|parent| {
            spawn_menu_button(
                parent,
                "🔬 Research",
                VillageMenuButton {
                    target: VillageContent::Research,
                },
                true,
            );
            spawn_menu_button(
                parent,
                if crafting_researched {
                    "⚒ Crafting"
                } else {
                    "⚒ Crafting (Locked)"
                },
                VillageMenuButton {
                    target: VillageContent::Crafting,
                },
                crafting_researched,
            );
            spawn_menu_button(
                parent,
                "📖 Encyclopedia",
                VillageMenuButton {
                    target: VillageContent::Encyclopedia,
                },
                true,
            );
            spawn_menu_button(
                parent,
                "🦸 Heroes",
                VillageMenuButton {
                    target: VillageContent::Heroes,
                },
                true,
            );
            spawn_menu_button(
                parent,
                if maw_exists {
                    "✨ Blessings"
                } else {
                    "✨ Blessings (Locked)"
                },
                VillageMenuButton {
                    target: VillageContent::Blessings,
                },
                maw_exists,
            );
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
                    let raw_tags = world
                        .get::<hero_components::WeaponTags>(child)
                        .map(|t| t.0.clone())
                        .unwrap_or_default();

                    let effective_damage = if let Some(bonus_stats) =
                        world.get_resource::<bonus_stats::BonusStats>()
                    {
                        bonus_stats::calculate_damage(damage_val, &raw_tags, &[], bonus_stats)
                    } else {
                        damage_val
                    };

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
                        effective_damage,
                        range: range_val,
                        speed_secs,
                        melee_arc: arc_degrees,
                    });
                    break;
                }
            }

            let skill_map = world.resource::<SkillMap>();
            let skill_definitions = world.resource::<Assets<SkillDefinition>>();

            let equipped_skills = world
                .get::<EquippedSkills>(*hero_entity)
                .map(|s| {
                    s.0.iter()
                        .map(|id| {
                            let name = skill_map
                                .handles
                                .get(id)
                                .and_then(|h| skill_definitions.get(h))
                                .map(|def| def.display_name.clone())
                                .unwrap_or_else(|| id.clone());
                            hero_ui::SkillDisplayData {
                                id: id.clone(),
                                name,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            heroes_data.push((
                *hero_entity,
                hero_ui::HeroDisplayData {
                    entity: *hero_entity,
                    name,
                    weapon: weapon_data,
                    equipped_skills,
                },
            ));
        }

        // Spawn back button and heroes content
        world.commands().entity(container).with_children(|parent| {
            // Back button
            spawn_menu_button(parent, "← Back", VillageBackButton, true);

            // Add HeroUiRoot marker for state tracking
            parent.spawn(HeroUiRoot);

            // Hero content container (refreshable)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    HeroContentContainer::default(),
                ))
                .with_children(|content| {
                    spawn_hero_content(content, heroes_data, 0);
                });
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
    mut next_village_state: ResMut<NextState<VillageView>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Ok(mut ui_root) = ui_query.single_mut()
        {
            ui_root.content = btn.target;

            match btn.target {
                VillageContent::Crafting => next_village_state.set(VillageView::Crafting),
                VillageContent::Research => next_village_state.set(VillageView::Research),
                VillageContent::Encyclopedia => next_village_state.set(VillageView::Encyclopedia),
                VillageContent::Heroes => next_village_state.set(VillageView::Heroes),
                VillageContent::Blessings => next_village_state.set(VillageView::Blessings),
                VillageContent::Menu => next_village_state.set(VillageView::Menu),
            }

            match btn.target {
                VillageContent::Heroes => {
                    commands.queue(SpawnHeroesContentCommand);
                }
                _ => {} // Other views handle their own content via state monitoring
            }
        }
    }
}

fn handle_back_button(
    _commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<VillageBackButton>)>,
    mut ui_query: Query<&mut VillageUiRoot>,
    mut next_village_state: ResMut<NextState<VillageView>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Ok(mut ui_root) = ui_query.single_mut()
        {
            ui_root.content = VillageContent::Menu;
            next_village_state.set(VillageView::Menu);
        }
    }
}

fn handle_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<VillageCloseButton>)>,
    ui_query: Query<(Entity, Option<&PanelWrapperRef>), With<VillageUiRoot>>,
    mut next_village_state: ResMut<NextState<VillageView>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for (ui_entity, wrapper_ref) in ui_query.iter() {
                next_village_state.set(VillageView::Closed);
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
