use {
    bevy::{picking::events::Click, prelude::*},
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    portal_components::Portal,
    states::GameState,
    wallet::Wallet,
    widgets::spawn_action_button,
};

pub struct PortalUiPlugin;

impl Plugin for PortalUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_portal_click).add_systems(
            Update,
            (
                update_portal_ui,
                handle_level_up_button,
                handle_close_button,
            )
                .run_if(in_state(GameState::Running)),
        );
    }
}

#[derive(Component)]
struct PortalUiRoot {
    portal_entity: Entity,
}

#[derive(Component)]
struct PortalDivinityText;

#[derive(Component)]
struct PortalCostText;

#[derive(Component)]
struct LevelUpButton {
    portal_entity: Entity,
}

#[derive(Component)]
struct PortalCloseButton;

/// Observer triggered when a Portal entity is clicked.
fn on_portal_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    portal_query: Query<(&Divinity, &DivinityStats), With<Portal>>,
    wallet: Res<Wallet>,
    existing_ui: Query<Entity, With<PortalUiRoot>>,
) {
    let portal_entity = trigger.entity;

    // Toggle behavior: if UI exists, despawn and return
    // for ui_entity in existing_ui.iter() {
    //     commands.entity(ui_entity).despawn();
    //     return;
    // }

    let Ok((divinity, stats)) = portal_query.get(portal_entity) else {
        return;
    };

    let cost = (stats.required_xp / 10.0).ceil() as u32;
    let have = wallet.resources.get("xikegos").copied().unwrap_or(0);
    let can_afford = have >= cost;

    // Spawn the UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(10.0),
                width: Val::Px(250.0),
                margin: UiRect::left(Val::Px(-125.0)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.5, 1.0)),
            PortalUiRoot { portal_entity },
            Pickable::default(),
            Interaction::default(),
        ))
        .with_children(|parent| {
            // Header with Close Button
            parent
                .spawn(Node {
                    display: Display::Flex,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("Portal Divinity"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 1.0, 1.0)),
                    ));

                    // Close Button (using raw button for simplicity/different look)
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
                            BackgroundColor(Color::srgba(0.8, 0.2, 0.2, 0.8)),
                            PortalCloseButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            parent.spawn((
                Text::new(format!("Tier {} Level {}", divinity.tier, divinity.level)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PortalDivinityText,
            ));

            parent.spawn((
                Text::new(format!("Cost: {} / {} xikegos", have, cost)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 1.0, 1.0)),
                PortalCostText,
                Node {
                    margin: UiRect {
                        top: Val::Px(5.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
            ));

            let btn_border = if can_afford {
                Color::srgba(0.0, 1.0, 0.0, 1.0)
            } else {
                Color::srgba(1.0, 0.0, 0.0, 1.0)
            };

            spawn_action_button(
                parent,
                "Level Up",
                Color::WHITE,
                btn_border,
                LevelUpButton { portal_entity },
            );
        });
}

fn handle_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PortalCloseButton>)>,
    ui_query: Query<Entity, With<PortalUiRoot>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for ui_entity in ui_query.iter() {
                commands.entity(ui_entity).despawn();
            }
        }
    }
}

fn update_portal_ui(
    wallet: Res<Wallet>,
    portal_query: Query<(&Divinity, &DivinityStats), With<Portal>>,
    ui_query: Query<(Entity, &PortalUiRoot)>,
    mut text_query: Query<(
        &mut Text,
        Option<&PortalDivinityText>,
        Option<&PortalCostText>,
    )>,
    mut btn_border_query: Query<&mut BorderColor, With<LevelUpButton>>,
    children_query: Query<&Children>,
) {
    for (ui_entity, ui_root) in ui_query.iter() {
        let Ok((divinity, stats)) = portal_query.get(ui_root.portal_entity) else {
            continue;
        };

        let cost = (stats.required_xp / 10.0).ceil() as u32;
        let have = wallet.resources.get("xikegos").copied().unwrap_or(0);
        let can_afford = have >= cost;

        // Recursive text update
        fn update_children_recursive(
            entity: Entity,
            divinity: &Divinity,
            have: u32,
            cost: u32,
            can_afford: bool,
            children_query: &Query<&Children>,
            text_query: &mut Query<(
                &mut Text,
                Option<&PortalDivinityText>,
                Option<&PortalCostText>,
            )>,
            btn_border_query: &mut Query<&mut BorderColor, With<LevelUpButton>>,
        ) {
            if let Ok((mut text, is_div, is_cost)) = text_query.get_mut(entity) {
                if is_div.is_some() {
                    text.0 = format!("Tier {} Level {}", divinity.tier, divinity.level);
                } else if is_cost.is_some() {
                    text.0 = format!("Cost: {} / {} xikegos", have, cost);
                }
            }

            if let Ok(mut border) = btn_border_query.get_mut(entity) {
                *border = BorderColor::all(if can_afford {
                    Color::srgba(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::srgba(1.0, 0.0, 0.0, 1.0)
                });
            }

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    update_children_recursive(
                        child,
                        divinity,
                        have,
                        cost,
                        can_afford,
                        children_query,
                        text_query,
                        btn_border_query,
                    );
                }
            }
        }

        update_children_recursive(
            ui_entity,
            divinity,
            have,
            cost,
            can_afford,
            &children_query,
            &mut text_query,
            &mut btn_border_query,
        );
    }
}

fn handle_level_up_button(
    mut commands: Commands,
    mut wallet: ResMut<Wallet>,
    portal_query: Query<&DivinityStats, With<Portal>>,
    interaction_query: Query<(&Interaction, &LevelUpButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed && btn.portal_entity != Entity::PLACEHOLDER {
            if let Ok(stats) = portal_query.get(btn.portal_entity) {
                let cost = (stats.required_xp / 10.0).ceil() as u32;
                let have = wallet.resources.get("xikegos").copied().unwrap_or(0);

                if have >= cost {
                    // Deduct xikegos
                    if let Some(res) = wallet.resources.get_mut("xikegos") {
                        *res -= cost;
                    }

                    // Add XP
                    commands.trigger(IncreaseDivinity {
                        entity: btn.portal_entity,
                        xp_amount: stats.required_xp, // Exactly enough to level up
                    });

                    info!("Portal Leveled Up! Spent {} xikegos", cost);
                }
            }
        }
    }
}
