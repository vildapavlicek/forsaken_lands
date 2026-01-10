use {
    bevy::{picking::events::Click, prelude::*},
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    portal_components::Portal,
    states::GameState,
    wallet::Wallet,
    widgets::{
        PanelWrapperRef, UiTheme, spawn_action_button, spawn_item_card,
        spawn_menu_panel, spawn_panel_header_with_close,
    },
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

// ============================================================================
// Components
// ============================================================================

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

#[derive(Component)]
struct DivinityCard;

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
        spawn_panel_header_with_close(parent, "Portal", PortalCloseButton);

        // Divinity Card
        let card = spawn_item_card(parent, DivinityCard);
        parent.commands().entity(card).with_children(|card_parent| {
            card_parent.spawn((
                Text::new("Divinity"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_HEADER),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Divinity level text
            card_parent.spawn((
                Text::new("Tier - Level -"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PortalDivinityText,
            ));

            // Cost text
            card_parent.spawn((
                Text::new("Cost: - / - xikegos"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_INFO),
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

            spawn_action_button(
                card_parent,
                "Level Up",
                Color::WHITE,
                UiTheme::BORDER_DISABLED,
                LevelUpButton { portal_entity },
            );
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

/// Context for recursive UI update
struct UpdateContext {
    divinity: Divinity,
    have: u32,
    cost: u32,
    can_afford: bool,
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
    mut color_query: Query<&mut TextColor>,
    mut border_query: Query<&mut BorderColor, With<LevelUpButton>>,
    children_query: Query<&Children>,
) {
    for (ui_entity, ui_root) in ui_query.iter() {
        let Ok((divinity, stats)) = portal_query.get(ui_root.portal_entity) else {
            continue;
        };

        let cost = (stats.required_xp / 10.0).ceil() as u32;
        let have = wallet.resources.get("xikegos").copied().unwrap_or(0);
        let can_afford = have >= cost;

        let ctx = UpdateContext {
            divinity: divinity.clone(),
            have,
            cost,
            can_afford,
        };

        update_children_recursive(
            ui_entity,
            &ctx,
            &children_query,
            &mut text_query,
            &mut color_query,
            &mut border_query,
        );
    }
}

#[allow(clippy::type_complexity)]
fn update_children_recursive(
    entity: Entity,
    ctx: &UpdateContext,
    children_query: &Query<&Children>,
    text_query: &mut Query<(
        &mut Text,
        Option<&PortalDivinityText>,
        Option<&PortalCostText>,
    )>,
    color_query: &mut Query<&mut TextColor>,
    border_query: &mut Query<&mut BorderColor, With<LevelUpButton>>,
) {
    if let Ok((mut text, is_div, is_cost)) = text_query.get_mut(entity) {
        if is_div.is_some() {
            text.0 = format!("Tier {} Level {}", ctx.divinity.tier, ctx.divinity.level);
        } else if is_cost.is_some() {
            text.0 = format!("Cost: {} / {} xikegos", ctx.have, ctx.cost);
            if let Ok(mut color) = color_query.get_mut(entity) {
                color.0 = if ctx.can_afford {
                    UiTheme::AFFORDABLE
                } else {
                    UiTheme::NOT_AFFORDABLE
                };
            }
        }
    }

    if let Ok(mut border) = border_query.get_mut(entity) {
        *border = BorderColor::all(if ctx.can_afford {
            UiTheme::BORDER_SUCCESS
        } else {
            UiTheme::BORDER_ERROR
        });

        if let Ok(mut text_color) = color_query.get_mut(entity) {
            text_color.0 = if ctx.can_afford {
                Color::WHITE
            } else {
                UiTheme::BORDER_DISABLED
            };
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_children_recursive(
                child,
                ctx,
                children_query,
                text_query,
                color_query,
                border_query,
            );
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_level_up_button(
    mut commands: Commands,
    mut wallet: ResMut<Wallet>,
    portal_query: Query<&DivinityStats, With<Portal>>,
    interaction_query: Query<(&Interaction, &LevelUpButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && btn.portal_entity != Entity::PLACEHOLDER
            && let Ok(stats) = portal_query.get(btn.portal_entity)
        {
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
