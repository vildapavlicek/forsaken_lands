use {
    bevy::prelude::*,
    equipment_events::{EquipWeaponRequest, UnequipWeaponRequest},
    hero_components::{AttackRange, AttackSpeed, Damage, Hero, MeleeArc, MeleeWeapon, Weapon},
    shared_components::DisplayName,
    skill_components::EquippedSkills,
    skills_assets::{SkillDefinition, SkillMap},
    states::GameState,
    widgets::{spawn_action_button, spawn_card_title, spawn_item_card, UiTheme},
};

pub struct HeroUiPlugin;

impl Plugin for HeroUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<HeroUiState>()
            .add_observer(on_hero_ui_added)
            .add_observer(on_hero_ui_removed)
            .add_observer(on_hero_ui_refresh)
            .add_systems(
                Update,
                (
                    handle_change_equipment_button,
                    handle_close_equipment_popup,
                    handle_equip_button,
                    handle_unequip_button,
                    handle_hero_tab_interaction,
                    handle_change_skill_button,
                    handle_close_skill_popup,
                    handle_equip_skill_button,
                )
                    .run_if(in_state(HeroUiState::Open).and(in_state(GameState::Running))),
            );
    }
}

// ============================================================================
// State
// ============================================================================

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HeroUiState {
    #[default]
    Closed,
    Open,
}

// ============================================================================
// Components
// ============================================================================

/// Root marker for the hero UI content
#[derive(Component)]
pub struct HeroUiRoot;

/// Tab button for selecting a hero
#[derive(Component)]
pub struct HeroTabButton {
    pub hero_entity: Entity,
}

/// Currently selected hero for display
#[derive(Component)]
pub struct SelectedHero(pub Entity);

/// Button to open equipment change popup
#[derive(Component)]
pub struct ChangeEquipmentButton {
    pub hero_entity: Entity,
}

/// Marker for the equipment popup
#[derive(Component)]
pub struct EquipmentPopup {
    pub hero_entity: Entity,
}

/// Close button for equipment popup
#[derive(Component)]
pub struct CloseEquipmentPopupButton;

/// Button to equip a specific weapon
#[derive(Component)]
pub struct EquipWeaponButton {
    pub hero_entity: Entity,
    pub weapon_entity: Entity,
}

/// Button to unequip current weapon
#[derive(Component)]
pub struct UnequipWeaponButton {
    pub hero_entity: Entity,
}

/// Container for unequipped weapons list
#[derive(Component)]
pub struct UnequippedWeaponsList;

/// Button to open skill change popup
#[derive(Component)]
pub struct ChangeSkillButton {
    pub hero_entity: Entity,
}

/// Marker for the skill popup
#[derive(Component)]
pub struct SkillPopup {
    pub hero_entity: Entity,
}

/// Close button for skill popup
#[derive(Component)]
pub struct CloseSkillPopupButton;

/// Button to equip a specific skill
#[derive(Component)]
pub struct EquipSkillButton {
    pub hero_entity: Entity,
    pub skill_id: String,
}

/// Marker for the hero content container that can be refreshed
/// Marker for the hero content container that can be refreshed
#[derive(Component, Default)]
pub struct HeroContentContainer {
    pub selected_index: usize,
}

// ============================================================================
// Events
// ============================================================================

/// Event to trigger a refresh of the hero UI content
#[derive(Event)]
pub struct RefreshHeroUiEvent;

// ============================================================================
// State Observers
// ============================================================================

fn on_hero_ui_added(
    _trigger: On<Insert, HeroUiRoot>,
    mut next_state: ResMut<NextState<HeroUiState>>,
) {
    debug!("hero ui state switched to OPEN");
    next_state.set(HeroUiState::Open);
}

fn on_hero_ui_removed(
    _trigger: On<Remove, HeroUiRoot>,
    mut next_state: ResMut<NextState<HeroUiState>>,
) {
    debug!("hero ui state switched to CLOSED");
    next_state.set(HeroUiState::Closed);
}

// ============================================================================
// Update Observer
// ============================================================================

fn on_hero_ui_refresh(
    _trigger: On<RefreshHeroUiEvent>,
    mut commands: Commands,
    content_container_query: Query<
        (Entity, Option<&Children>, &HeroContentContainer),
        With<HeroContentContainer>,
    >,
    hero_query: Query<Entity, With<Hero>>,
    children_query: Query<&Children>,
    equipped_skills_query: Query<&EquippedSkills>,
    skill_map: Res<SkillMap>,
    skill_definitions: Res<Assets<SkillDefinition>>,
    weapon_query: Query<
        (
            Entity,
            Option<&DisplayName>,
            &Damage,
            &AttackRange,
            &AttackSpeed,
            Option<&MeleeArc>,
            Option<&hero_components::WeaponTags>,
        ),
        With<Weapon>,
    >,
    melee_query: Query<(), With<MeleeWeapon>>,
    bonus_stats: Res<bonus_stats::BonusStats>,
) {
    // Get the content container
    let Ok((container_entity, container_children, container)) = content_container_query.single()
    else {
        return;
    };

    // Despawn existing content
    if let Some(children) = container_children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // Get hero entities and build display data
    let hero_entities: Vec<Entity> = hero_query.iter().collect();
    let mut heroes_data: Vec<(Entity, HeroDisplayData)> = Vec::new();

    for hero_entity in &hero_entities {
        let data = build_hero_display_data(
            *hero_entity,
            &children_query,
            &weapon_query,
            &melee_query,
            &equipped_skills_query,
            &skill_map,
            &skill_definitions,
            &bonus_stats,
        );
        heroes_data.push((*hero_entity, data));
    }

    // Respawn updated hero content
    commands.entity(container_entity).with_children(|parent| {
        spawn_hero_content(parent, heroes_data, container.selected_index);
    });
}

// ============================================================================
// Display Data
// ============================================================================

/// Data for displaying weapon stats
#[derive(Clone)]
pub struct WeaponDisplayData {
    pub entity: Entity,
    pub name: String,
    pub damage: f32,
    pub effective_damage: f32,
    pub range: f32,
    pub speed_secs: f32,
    pub melee_arc: Option<f32>, // In degrees, only for melee weapons
}

/// Data for displaying skill info
#[derive(Clone)]
pub struct SkillDisplayData {
    pub id: String,
    pub name: String,
}

/// Data for displaying a hero
pub struct HeroDisplayData {
    pub entity: Entity,
    pub name: String,
    pub weapon: Option<WeaponDisplayData>,
    pub equipped_skills: Vec<SkillDisplayData>,
}

// ============================================================================
// UI Spawn Functions
// ============================================================================

/// Spawns the hero content UI.
/// This is called by village_ui when Heroes content is selected.
pub fn spawn_hero_content(
    parent: &mut ChildSpawnerCommands,
    heroes: Vec<(Entity, HeroDisplayData)>,
    selected_index: usize,
) {
    // Hero tabs container
    if heroes.len() > 1 {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            })
            .with_children(|tabs| {
                for (idx, (entity, data)) in heroes.iter().enumerate() {
                    let is_active = idx == selected_index;
                    spawn_hero_tab(tabs, *entity, &data.name, is_active);
                }
            });
    }

    // Display selected hero details
    if let Some((hero_entity, hero_data)) = heroes.get(selected_index) {
        spawn_hero_details(parent, *hero_entity, hero_data);
    }
}

fn spawn_hero_tab(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    name: &str,
    is_active: bool,
) {
    let bg_color = if is_active {
        UiTheme::TAB_ACTIVE_BG
    } else {
        UiTheme::TAB_INACTIVE_BG
    };

    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::right(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(UiTheme::TAB_BORDER),
            BackgroundColor(bg_color),
            HeroTabButton { hero_entity },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}

fn spawn_hero_details(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    hero: &HeroDisplayData,
) {
    // Hero name header card
    let name_card = spawn_item_card(parent, ());
    parent.commands().entity(name_card).with_children(|card| {
        spawn_card_title(card, &hero.name);
    });

    // Skills section
    spawn_skills_section(parent, hero_entity, &hero.equipped_skills);

    // Weapon section
    if let Some(weapon) = &hero.weapon {
        spawn_weapon_section(parent, hero_entity, weapon);
    } else {
        // No weapon equipped message
        parent.spawn((
            Text::new("No weapon equipped"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(UiTheme::TEXT_SECONDARY),
        ));

        // Change equipment button (to equip from armory)
        spawn_action_button(
            parent,
            "⚔ Equip Weapon",
            UiTheme::TEXT_PRIMARY,
            UiTheme::BORDER_SUCCESS,
            ChangeEquipmentButton { hero_entity },
        );
    }
}

fn spawn_weapon_section(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    weapon: &WeaponDisplayData,
) {
    // Weapon header
    parent.spawn((
        Text::new("Weapon"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(UiTheme::TEXT_HEADER),
        Node {
            margin: UiRect::vertical(Val::Px(8.0)),
            ..default()
        },
    ));

    // Weapon card with stats
    let weapon_card = spawn_item_card(parent, ());
    parent.commands().entity(weapon_card).with_children(|card| {
        // Weapon name
        spawn_stat_row(card, "Name", &weapon.name);

        // Damage
        let damage_text = if (weapon.effective_damage - weapon.damage).abs() > 0.01 {
            let bonus = weapon.effective_damage - weapon.damage;
            format!(
                "{:.2} ({:.2} + {:.2})",
                weapon.effective_damage, weapon.damage, bonus
            )
        } else {
            format!("{:.2}", weapon.damage)
        };
        spawn_stat_row(card, "Damage", &damage_text);

        // Range
        spawn_stat_row(card, "Range", &format!("{:.1}", weapon.range));

        // Attack speed
        spawn_stat_row(card, "Speed", &format!("{:.2}s", weapon.speed_secs));

        // Melee arc (only for melee weapons)
        if let Some(arc_degrees) = weapon.melee_arc {
            spawn_stat_row(card, "Arc", &format!("{:.0}°", arc_degrees));
        }
    });

    // Change equipment button
    spawn_action_button(
        parent,
        "⚔ Change Equipment",
        UiTheme::TEXT_PRIMARY,
        UiTheme::TAB_BORDER,
        ChangeEquipmentButton { hero_entity },
    );
}

fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(format!("{}:", label)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_SECONDARY),
            ));

            // Value
            row.spawn((
                Text::new(value),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}

pub fn spawn_skills_section(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    equipped_skills: &[SkillDisplayData],
) {
    // Skills header
    parent.spawn((
        Text::new("Skills"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(UiTheme::TEXT_HEADER),
        Node {
            margin: UiRect::vertical(Val::Px(8.0)),
            ..default()
        },
    ));

    // Skill slots container
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|container| {
            // For now, let's just show one slot
            let skill = equipped_skills.first().cloned();

            spawn_skill_slot(container, hero_entity, skill);
        });
}

fn spawn_skill_slot(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    skill: Option<SkillDisplayData>,
) {
    let (label, border_color) = if let Some(s) = skill {
        (s.name, UiTheme::TAB_BORDER)
    } else {
        ("[ Empty Slot ]".to_string(), UiTheme::TEXT_SECONDARY)
    };

    spawn_action_button(
        parent,
        &label,
        UiTheme::TEXT_PRIMARY,
        border_color,
        ChangeSkillButton { hero_entity },
    );
}

// ============================================================================
// Skill Popup
// ============================================================================

pub fn spawn_skill_popup(
    commands: &mut Commands,
    hero_entity: Entity,
    available_skills: Vec<(String, String)>, // (id, display_name)
) {
    // Full-screen overlay
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            SkillPopup { hero_entity },
            Interaction::default(),
        ))
        .with_children(|overlay| {
            // Popup panel
            overlay
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        max_height: Val::Vh(70.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(15.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(UiTheme::POPUP_BG),
                    BorderColor::all(UiTheme::POPUP_BORDER),
                ))
                .with_children(|popup| {
                    // Header row
                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        })
                        .with_children(|header| {
                            header.spawn((
                                Text::new("Select Skill"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(UiTheme::TEXT_HEADER),
                            ));

                            // Close button
                            header
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(UiTheme::CLOSE_BUTTON_BG),
                                    CloseSkillPopupButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("X"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // Scrollable container for available skills
                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            max_height: Val::Vh(40.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        })
                        .with_children(|scroll_container| {
                            for (skill_id, display_name) in available_skills {
                                spawn_skill_selection_card(
                                    scroll_container,
                                    hero_entity,
                                    skill_id,
                                    display_name,
                                );
                            }
                        });
                });
        });
}

fn spawn_skill_selection_card(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    skill_id: String,
    display_name: String,
) {
    let card = spawn_item_card(parent, ());
    parent.commands().entity(card).with_children(|card| {
        card.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(&display_name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));

            row.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(UiTheme::BORDER_SUCCESS),
                BackgroundColor(UiTheme::BUTTON_NORMAL),
                EquipSkillButton {
                    hero_entity,
                    skill_id,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Equip"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(UiTheme::TEXT_PRIMARY),
                ));
            });
        });
    });
}

// ============================================================================
// Equipment Popup
// ============================================================================

/// Spawns the equipment popup showing available weapons
pub fn spawn_equipment_popup(
    commands: &mut Commands,
    hero_entity: Entity,
    equipped_weapon: Option<&WeaponDisplayData>,
    unequipped_weapons: Vec<WeaponDisplayData>,
) {
    // Full-screen overlay
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            EquipmentPopup { hero_entity },
            Interaction::default(),
        ))
        .with_children(|overlay| {
            // Popup panel
            overlay
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        max_height: Val::Vh(70.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(15.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(UiTheme::POPUP_BG),
                    BorderColor::all(UiTheme::POPUP_BORDER),
                ))
                .with_children(|popup| {
                    // Header row
                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        })
                        .with_children(|header| {
                            header.spawn((
                                Text::new("Equipment"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(UiTheme::TEXT_HEADER),
                            ));

                            // Close button
                            header
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(UiTheme::CLOSE_BUTTON_BG),
                                    CloseEquipmentPopupButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("X"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // Currently equipped section
                    popup.spawn((
                        Text::new("Equipped"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(UiTheme::TEXT_HEADER),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            ..default()
                        },
                    ));

                    if let Some(weapon) = equipped_weapon {
                        spawn_popup_weapon_card(popup, hero_entity, weapon, true);
                    } else {
                        popup.spawn((
                            Text::new("No weapon equipped"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(UiTheme::TEXT_SECONDARY),
                            Node {
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    }

                    // Available weapons section
                    popup.spawn((
                        Text::new("Available Weapons"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(UiTheme::TEXT_HEADER),
                        Node {
                            margin: UiRect::vertical(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    if unequipped_weapons.is_empty() {
                        popup.spawn((
                            Text::new("No weapons available. Craft weapons to add them here."),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(UiTheme::TEXT_SECONDARY),
                        ));
                    } else {
                        // Scrollable container for available weapons
                        popup
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    max_height: Val::Vh(40.0),
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                                UnequippedWeaponsList,
                            ))
                            .with_children(|scroll_container| {
                                // Spawn weapons directly here
                                for weapon in &unequipped_weapons {
                                    spawn_popup_weapon_card(
                                        scroll_container,
                                        hero_entity,
                                        weapon,
                                        false,
                                    );
                                }
                            });
                    }
                });
        });
}

fn spawn_popup_weapon_card(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    weapon: &WeaponDisplayData,
    is_equipped: bool,
) {
    let weapon_card = spawn_item_card(parent, ());
    let weapon_entity = weapon.entity;
    let weapon_name = weapon.name.clone();

    parent.commands().entity(weapon_card).with_children(|card| {
        // Weapon info row
        card.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|row| {
            // Weapon info
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|info| {
                info.spawn((
                    Text::new(&weapon_name),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(UiTheme::TEXT_PRIMARY),
                ));

                info.spawn((
                    Text::new(format!(
                        "DMG: {:.2} | RNG: {:.1}",
                        weapon.effective_damage, weapon.range
                    )),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(UiTheme::TEXT_SECONDARY),
                ));
            });

            // Action button
            if is_equipped {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(UiTheme::BORDER_ERROR),
                    BackgroundColor(UiTheme::BUTTON_NORMAL),
                    UnequipWeaponButton { hero_entity },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Unequip"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(UiTheme::TEXT_PRIMARY),
                    ));
                });
            } else {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(UiTheme::BORDER_SUCCESS),
                    BackgroundColor(UiTheme::BUTTON_NORMAL),
                    EquipWeaponButton {
                        hero_entity,
                        weapon_entity,
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Equip"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(UiTheme::TEXT_PRIMARY),
                    ));
                });
            }
        });
    });
}

// ============================================================================
// Button Handlers
// ============================================================================

#[allow(clippy::type_complexity)]
fn handle_change_equipment_button(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &ChangeEquipmentButton),
        (Changed<Interaction>, With<Button>),
    >,
    existing_popup: Query<Entity, With<EquipmentPopup>>,
    // Queries for building weapon data
    hero_children_query: Query<&Children, With<hero_components::Hero>>,
    unequipped_weapons_query: Query<
        (
            Entity,
            Option<&DisplayName>,
            &Damage,
            &AttackRange,
            &AttackSpeed,
            Option<&MeleeArc>,
            Option<&hero_components::WeaponTags>,
        ),
        (With<Weapon>, Without<ChildOf>),
    >,
    equipped_weapon_query: Query<
        (
            Entity,
            Option<&DisplayName>,
            &Damage,
            &AttackRange,
            &AttackSpeed,
            Option<&MeleeArc>,
            Option<&hero_components::WeaponTags>,
        ),
        With<Weapon>,
    >,
    melee_query: Query<(), With<MeleeWeapon>>,
    bonus_stats: Res<bonus_stats::BonusStats>,
) {
    // Log all button interactions for debugging
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Close existing popup if any

            for entity in existing_popup.iter() {
                commands.entity(entity).despawn();
            }

            let hero_entity = btn.hero_entity;

            // Build equipped weapon data
            let equipped_weapon = hero_children_query
                .get(hero_entity)
                .ok()
                .and_then(|children| {
                    children.iter().find_map(|child| {
                        equipped_weapon_query.get(child).ok().map(
                            |(entity, display_name, damage, range, speed, melee_arc, tags)| {
                                let name = display_name
                                    .map(|d| d.0.clone())
                                    .unwrap_or_else(|| "Unknown Weapon".to_string());
                                let speed_secs = speed.timer.duration().as_secs_f32();
                                let arc = if melee_query.get(child).is_ok() {
                                    melee_arc.map(|a| a.width.to_degrees())
                                } else {
                                    None
                                };
                                let raw_tags = tags.map(|t| t.0.clone()).unwrap_or_default();
                                let effective_damage = bonus_stats::calculate_damage(
                                    damage.0,
                                    &raw_tags,
                                    &[],
                                    &bonus_stats,
                                );
                                WeaponDisplayData {
                                    entity,
                                    name,
                                    damage: damage.0,
                                    effective_damage,
                                    range: range.0,
                                    speed_secs,
                                    melee_arc: arc,
                                }
                            },
                        )
                    })
                });

            // Build unequipped weapons list

            let unequipped_weapons: Vec<WeaponDisplayData> = unequipped_weapons_query
                .iter()
                .map(
                    |(entity, display_name, damage, range, speed, melee_arc, tags)| {
                        let name = display_name
                            .map(|d| d.0.clone())
                            .unwrap_or_else(|| "Unknown Weapon".to_string());
                        let speed_secs = speed.timer.duration().as_secs_f32();
                        let arc = if melee_query.get(entity).is_ok() {
                            melee_arc.map(|a| a.width.to_degrees())
                        } else {
                            None
                        };
                        let raw_tags = tags.map(|t| t.0.clone()).unwrap_or_default();
                        let effective_damage =
                            bonus_stats::calculate_damage(damage.0, &raw_tags, &[], &bonus_stats);
                        WeaponDisplayData {
                            entity,
                            name,
                            damage: damage.0,
                            effective_damage,
                            range: range.0,
                            speed_secs,
                            melee_arc: arc,
                        }
                    },
                )
                .collect();

            // Spawn popup (weapons are spawned directly inside the popup)
            spawn_equipment_popup(
                &mut commands,
                hero_entity,
                equipped_weapon.as_ref(),
                unequipped_weapons,
            );
        }
    }
}

fn handle_close_equipment_popup(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseEquipmentPopupButton>)>,
    popup_query: Query<Entity, With<EquipmentPopup>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in popup_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn handle_equip_button(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &EquipWeaponButton),
        (Changed<Interaction>, With<Button>),
    >,
    popup_query: Query<Entity, With<EquipmentPopup>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Trigger equip event
            commands.trigger(EquipWeaponRequest {
                hero: btn.hero_entity,
                weapon: btn.weapon_entity,
            });

            // Close popup and trigger UI refresh
            for entity in popup_query.iter() {
                commands.entity(entity).despawn();
            }
            commands.trigger(RefreshHeroUiEvent);
        }
    }
}

fn handle_unequip_button(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &UnequipWeaponButton),
        (Changed<Interaction>, With<Button>),
    >,
    popup_query: Query<Entity, With<EquipmentPopup>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Trigger unequip event
            commands.trigger(UnequipWeaponRequest {
                hero: btn.hero_entity,
            });

            // Close popup and trigger UI refresh
            for entity in popup_query.iter() {
                commands.entity(entity).despawn();
            }
            commands.trigger(RefreshHeroUiEvent);
        }
    }
}

fn handle_hero_tab_interaction(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &HeroTabButton), (Changed<Interaction>, With<Button>)>,
    mut container_query: Query<&mut HeroContentContainer>,
    hero_query: Query<Entity, With<Hero>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut container) = container_query.single_mut() {
                // Find index of this hero
                let heroes: Vec<Entity> = hero_query.iter().collect();
                if let Some(index) = heroes.iter().position(|e| *e == btn.hero_entity) {
                    container.selected_index = index;
                    commands.trigger(RefreshHeroUiEvent);
                }
            }
        }
    }
}

fn handle_change_skill_button(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &ChangeSkillButton),
        (Changed<Interaction>, With<Button>),
    >,
    skill_map: Res<SkillMap>,
    skill_definitions: Res<Assets<SkillDefinition>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let hero_entity = btn.hero_entity;

            // Collect available skills
            let mut available_skills = Vec::new();
            for (id, handle) in skill_map.handles.iter() {
                if let Some(def) = skill_definitions.get(handle) {
                    available_skills.push((id.clone(), def.display_name.clone()));
                }
            }

            spawn_skill_popup(&mut commands, hero_entity, available_skills);
        }
    }
}

fn handle_close_skill_popup(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseSkillPopupButton>)>,
    popup_query: Query<Entity, With<SkillPopup>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in popup_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn handle_equip_skill_button(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &EquipSkillButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut hero_query: Query<&mut EquippedSkills>,
    popup_query: Query<Entity, With<SkillPopup>>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Skill {} equipped to hero {:?}", btn.skill_id, btn.hero_entity);

            if let Ok(mut equipped) = hero_query.get_mut(btn.hero_entity) {
                // For now, let's just replace the first skill or add if empty
                if equipped.0.is_empty() {
                    equipped.0.push(btn.skill_id.clone());
                } else {
                    equipped.0[0] = btn.skill_id.clone();
                }
            } else {
                // If the hero doesn't have the component, something is wrong, but we can add it
                commands.entity(btn.hero_entity).insert(EquippedSkills(vec![btn.skill_id.clone()]));
            }

            // Close popup and refresh UI
            for entity in popup_query.iter() {
                commands.entity(entity).despawn();
            }
            commands.trigger(RefreshHeroUiEvent);
        }
    }
}

// ============================================================================
// Query Helpers
// ============================================================================

/// Builds HeroDisplayData from hero entity and its weapon children.
/// Call this from village_ui when building the heroes content.
pub fn build_hero_display_data(
    hero_entity: Entity,
    children_query: &Query<&Children>,
    weapon_query: &Query<
        (
            Entity,
            Option<&DisplayName>,
            &Damage,
            &AttackRange,
            &AttackSpeed,
            Option<&MeleeArc>,
            Option<&hero_components::WeaponTags>,
        ),
        With<Weapon>,
    >,
    is_melee_query: &Query<(), With<MeleeWeapon>>,
    equipped_skills_query: &Query<&EquippedSkills>,
    skill_map: &SkillMap,
    skill_definitions: &Assets<SkillDefinition>,
    bonus_stats: &bonus_stats::BonusStats,
) -> HeroDisplayData {
    // Placeholder hero name (heroes don't have names yet)
    let name = "Hero".to_string();

    // Fetch equipped skills
    let equipped_skills = equipped_skills_query
        .get(hero_entity)
        .map(|s| {
            s.0.iter()
                .map(|id| {
                    let name = skill_map
                        .handles
                        .get(id)
                        .and_then(|h| skill_definitions.get(h))
                        .map(|def| def.display_name.clone())
                        .unwrap_or_else(|| id.clone());
                    SkillDisplayData {
                        id: id.clone(),
                        name,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Find weapon child
    let weapon = children_query.get(hero_entity).ok().and_then(|children| {
        children.iter().find_map(|child| {
            weapon_query.get(child).ok().map(
                |(entity, display_name, damage, range, speed, melee_arc, tags)| {
                    let weapon_name = display_name
                        .map(|d| d.0.clone())
                        .unwrap_or_else(|| "Unknown Weapon".to_string());

                    // Get speed in seconds from timer duration
                    let speed_secs = speed.timer.duration().as_secs_f32();

                    // Only show arc for melee weapons
                    let arc_degrees = if is_melee_query.get(child).is_ok() {
                        melee_arc.map(|arc| arc.width.to_degrees())
                    } else {
                        None
                    };

                    // Calculate effective damage
                    let raw_tags = tags.map(|t| t.0.clone()).unwrap_or_default();
                    let effective_damage =
                        bonus_stats::calculate_damage(damage.0, &raw_tags, &[], bonus_stats);

                    WeaponDisplayData {
                        entity,
                        name: weapon_name,
                        damage: damage.0,
                        effective_damage,
                        range: range.0,
                        speed_secs,
                        melee_arc: arc_degrees,
                    }
                },
            )
        })
    });

    HeroDisplayData {
        entity: hero_entity,
        name,
        weapon,
        equipped_skills,
    }
}
