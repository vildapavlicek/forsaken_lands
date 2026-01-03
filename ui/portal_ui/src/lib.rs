use {
    bevy::{picking::events::Click, prelude::*},
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    portal_components::Portal,
    states::GameState,
    wallet::Wallet,
    widgets::{
        spawn_action_button, spawn_panel_header_with_close, spawn_ui_panel, PanelPosition,
        UiTheme,
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


    let Ok((divinity, stats)) = portal_query.get(portal_entity) else {
        return;
    };

    let cost = (stats.required_xp / 10.0).ceil() as u32;
    let have = wallet.resources.get("xikegos").copied().unwrap_or(0);
    let can_afford = have >= cost;

    // Spawn the panel using widget
    let panel_entity = spawn_ui_panel(
        &mut commands,
        PanelPosition::CenterPopup { top: 10.0 },
        250.0,
        Val::Auto,
        PortalUiRoot { portal_entity },
    );

    commands.entity(panel_entity).with_children(|parent| {
        // Header with close button
        spawn_panel_header_with_close(parent, "Portal Divinity", PortalCloseButton);

        // Divinity level text
        parent.spawn((
            Text::new(format!("Tier {} Level {}", divinity.tier, divinity.level)),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            PortalDivinityText,
        ));

        // Cost text
        parent.spawn((
            Text::new(format!("Cost: {} / {} xikegos", have, cost)),
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

        // Level up button
        let btn_border = if can_afford {
            UiTheme::BORDER_SUCCESS
        } else {
            UiTheme::BORDER_ERROR
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

/// Context for recursive UI update
struct UpdateContext<'a> {
    divinity: &'a Divinity,
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

        let ctx = UpdateContext {
            divinity,
            have,
            cost,
            can_afford,
        };

        update_children_recursive(
            ui_entity,
            &ctx,
            &children_query,
            &mut text_query,
            &mut btn_border_query,
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
    btn_border_query: &mut Query<&mut BorderColor, With<LevelUpButton>>,
) {
    if let Ok((mut text, is_div, is_cost)) = text_query.get_mut(entity) {
        if is_div.is_some() {
            text.0 = format!("Tier {} Level {}", ctx.divinity.tier, ctx.divinity.level);
        } else if is_cost.is_some() {
            text.0 = format!("Cost: {} / {} xikegos", ctx.have, ctx.cost);
        }
    }

    if let Ok(mut border) = btn_border_query.get_mut(entity) {
        *border = BorderColor::all(if ctx.can_afford {
            UiTheme::BORDER_SUCCESS
        } else {
            UiTheme::BORDER_ERROR
        });
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_children_recursive(child, ctx, children_query, text_query, btn_border_query);
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
