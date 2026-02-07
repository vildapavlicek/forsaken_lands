use {
    bevy::{picking::events::Click, prelude::*},
    divinity_components::{CurrentDivinity, Divinity},
    portal_components::Portal,
    states::GameState,
    unlocks_assets::{ConditionNode, UnlockDefinition},
    village_components::Village,
    widgets::{PanelWrapperRef, UiTheme, spawn_menu_panel, spawn_panel_header_with_close},
};

pub struct PortalUiPlugin;

impl Plugin for PortalUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_portal_click).add_systems(
            Update,
            (
                update_portal_ui,
                handle_tier_navigation,
                handle_close_button,
            )
                .run_if(in_state(GameState::Running)),
        );
    }
}

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
struct PortalUiRoot {
    portal_entity: Entity,
}

#[derive(Component)]
struct CurrentDivinityText;

#[derive(Component)]
struct MaxDivinityText;

#[derive(Component)]
struct UnlockConditionText;

#[derive(Component)]
struct DecreaseTierButton {
    portal_entity: Entity,
}

#[derive(Component)]
struct IncreaseTierButton {
    portal_entity: Entity,
}

#[derive(Component)]
struct PortalCloseButton;

// ============================================================================
// Portal Click Observer
// ============================================================================

/// Observer triggered when a Portal entity is clicked.
fn on_portal_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    portal_query: Query<(), With<Portal>>,
    existing_ui: Query<(Entity, Option<&PanelWrapperRef>), With<PortalUiRoot>>,
) {
    let portal_entity = trigger.entity;

    // Verify this is a portal entity
    if portal_query.get(portal_entity).is_err() {
        return;
    }

    // Toggle: if UI exists, close it; otherwise open
    if let Ok((ui_entity, wrapper_ref)) = existing_ui.single() {
        // Despawn wrapper if it exists, otherwise just despawn the panel
        if let Some(wrapper) = wrapper_ref {
            commands.entity(wrapper.0).despawn();
        } else {
            commands.entity(ui_entity).despawn();
        }
        return;
    }

    spawn_portal_ui(&mut commands, portal_entity);
}

// ============================================================================
// Spawn Portal UI
// ============================================================================

fn spawn_portal_ui(commands: &mut Commands, portal_entity: Entity) {
    let panel_entity = spawn_menu_panel(commands, PortalUiRoot { portal_entity });

    commands.entity(panel_entity).with_children(|parent| {
        // Header with close button
        spawn_panel_header_with_close(parent, "Portal Menu", PortalCloseButton);

        // Tier navigation row: [<] Tier X - Level Y [>]
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(15.0)),
                column_gap: Val::Px(15.0),
                ..default()
            })
            .with_children(|row| {
                // Decrease button [<]
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(UiTheme::CARD_BORDER),
                    BackgroundColor(UiTheme::BUTTON_NORMAL),
                    DecreaseTierButton { portal_entity },
                ))
                .with_child((
                    Text::new("<"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Current tier/level text
                row.spawn((
                    Text::new("Tier 1 - Level 1"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    CurrentDivinityText,
                ));

                // Increase button [>]
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(UiTheme::CARD_BORDER),
                    BackgroundColor(UiTheme::BUTTON_NORMAL),
                    IncreaseTierButton { portal_entity },
                ))
                .with_child((
                    Text::new(">"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

        // Max tier available section
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(10.0)),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("Max tier available:"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(UiTheme::TEXT_INFO),
                ));

                col.spawn((
                    Text::new("Tier 1 - Level 1"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    MaxDivinityText,
                    Node {
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    },
                ));
            });

        // Unlock Condition Text
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(10.0)),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(UiTheme::TEXT_INFO),
                    UnlockConditionText,
                    Node {
                        max_width: Val::Px(300.0),
                        ..default()
                    },
                ));
            });
    });
}

// ============================================================================
// Systems
// ============================================================================

fn handle_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PortalCloseButton>)>,
    ui_query: Query<(Entity, Option<&PanelWrapperRef>), With<PortalUiRoot>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for (ui_entity, wrapper_ref) in ui_query.iter() {
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

fn update_portal_ui(
    portal_query: Query<&CurrentDivinity, With<Portal>>,
    village_query: Query<&Divinity, With<Village>>,
    ui_query: Query<&PortalUiRoot>,
    mut current_text_query: Query<&mut Text, (With<CurrentDivinityText>, Without<MaxDivinityText>)>,
    mut max_text_query: Query<&mut Text, (With<MaxDivinityText>, Without<CurrentDivinityText>)>,
    mut condition_text_query: Query<
        &mut Text,
        (
            With<UnlockConditionText>,
            Without<CurrentDivinityText>,
            Without<MaxDivinityText>,
        ),
    >,
    unlock_definitions: Res<Assets<UnlockDefinition>>,
) {
    let Some(max_divinity) = village_query.iter().next() else {
        return;
    };

    for ui_root in ui_query.iter() {
        let Ok(divinity) = portal_query.get(ui_root.portal_entity) else {
            continue;
        };

        // Update current divinity text
        for mut text in current_text_query.iter_mut() {
            text.0 = format!("Tier {} - Level {}", divinity.tier, divinity.level);
        }

        // Update max divinity text
        for mut text in max_text_query.iter_mut() {
            text.0 = format!("Tier {} - Level {}", max_divinity.tier, max_divinity.level);
        }

        // Update unlock condition text
        // Determine what the NEXT unlock is
        let (target_tier, target_level) = if max_divinity.level < divinity_components::MAX_LEVEL {
            (max_divinity.tier, max_divinity.level + 1)
        } else {
            (max_divinity.tier + 1, 1)
        };
        let target_reward_id = format!("divinity:{}-{}", target_tier, target_level);

        let mut condition_text: String;

        // Find the unlock definition for this reward
        if let Some((_, def)) = unlock_definitions
            .iter()
            .find(|(_, d)| d.reward_id == target_reward_id)
        {
            condition_text = format!("To unlock Tier {} Level {}:\n", target_tier, target_level);
            match &def.condition {
                ConditionNode::Completed { topic } => {
                    // Try to fix up the topic for display if it's a known format
                    if let Some(research_name) = topic.strip_prefix("research:") {
                        // Basic capitalization or formatting could go here
                        condition_text.push_str(&format!("Research: {}", research_name));
                    } else {
                        condition_text.push_str(&format!("Complete: {}", topic));
                    }
                }
                ConditionNode::Value { topic, target, .. } => {
                    condition_text.push_str(&format!("{} >= {}", topic, target));
                }
                ConditionNode::And(nodes) => {
                    condition_text.push_str("Complete ALL:\n");
                    for node in nodes {
                        match node {
                            ConditionNode::Completed { topic } => {
                                if let Some(research_name) = topic.strip_prefix("research:") {
                                    condition_text
                                        .push_str(&format!("- Research: {}\n", research_name));
                                } else {
                                    condition_text.push_str(&format!("- Complete: {}\n", topic));
                                }
                            }
                            ConditionNode::Value { topic, target, .. } => {
                                condition_text.push_str(&format!("- {} >= {}\n", topic, target));
                            }
                            _ => condition_text.push_str("- ...\n"),
                        }
                    }
                }
                _ => condition_text.push_str("Unknown condition"),
            }
        } else {
            // Check if we are at absolute max (no more definitions found)
            condition_text = "Max Level Reached".to_string();
        }

        for mut text in condition_text_query.iter_mut() {
            text.0 = condition_text.clone();
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_tier_navigation(
    mut portal_query: Query<&mut CurrentDivinity, With<Portal>>,
    village_query: Query<&Divinity, With<Village>>,
    decrease_query: Query<
        (&Interaction, &DecreaseTierButton),
        (Changed<Interaction>, With<Button>),
    >,
    increase_query: Query<
        (&Interaction, &IncreaseTierButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let Some(max_divinity) = village_query.iter().next() else {
        return;
    };
    // Handle decrease button
    for (interaction, btn) in decrease_query.iter() {
        if *interaction == Interaction::Pressed
            && let Ok(mut divinity) = portal_query.get_mut(btn.portal_entity) {
                // Decrease level, wrapping to previous tier if needed
                if divinity.level > 1 {
                    divinity.level -= 1;
                } else if divinity.tier > 1 {
                    divinity.tier -= 1;
                    divinity.level = divinity_components::MAX_LEVEL;
                }
                // If already at tier 1 level 1, do nothing
            }
    }

    // Handle increase button
    for (interaction, btn) in increase_query.iter() {
        if *interaction == Interaction::Pressed
            && let Ok(mut divinity) = portal_query.get_mut(btn.portal_entity) {
                // Only allow increase up to max unlocked divinity
                let current = **divinity;
                let max = *max_divinity;

                if current < max {
                    if divinity.level < divinity_components::MAX_LEVEL {
                        divinity.level += 1;
                    } else {
                        divinity.tier += 1;
                        divinity.level = 1;
                    }

                    // Clamp to max
                    if divinity.0 > max {
                        divinity.0 = max;
                    }
                }
            }
    }
}
