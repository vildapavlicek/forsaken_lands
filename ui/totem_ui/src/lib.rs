use {
    bevy::{picking::events::Click, prelude::*},
    states::GameState,
    totem::Totem,
    widgets::{PanelWrapperRef, UiTheme, spawn_menu_panel, spawn_panel_header_with_close},
};

pub struct TotemUiPlugin;

impl Plugin for TotemUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_totem_click).add_systems(
            Update,
            (handle_close_button).run_if(in_state(GameState::Running)),
        );
    }
}

#[derive(Component)]
struct TotemUiRoot {
    totem_entity: Entity,
}

#[derive(Component)]
struct TotemCloseButton;

fn on_totem_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    totem_query: Query<(), With<Totem>>,
    existing_ui: Query<(Entity, Option<&PanelWrapperRef>), With<TotemUiRoot>>,
) {
    let totem_entity = trigger.entity;

    if totem_query.get(totem_entity).is_err() {
        return;
    }

    if let Ok((ui_entity, wrapper_ref)) = existing_ui.single() {
        if let Some(wrapper) = wrapper_ref {
            commands.entity(wrapper.0).despawn();
        } else {
            commands.entity(ui_entity).despawn();
        }
        return;
    }

    spawn_totem_ui(&mut commands, totem_entity);
}

fn spawn_totem_ui(commands: &mut Commands, totem_entity: Entity) {
    let panel_entity = spawn_menu_panel(commands, TotemUiRoot { totem_entity });

    commands.entity(panel_entity).with_children(|parent| {
        spawn_panel_header_with_close(parent, "Totem Menu", TotemCloseButton);

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("The Totem radiates dark energy, continually damaging enemies in the area.\n\nActive Skill: Totem Aura\nUpgrades: Coming Soon"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(UiTheme::TEXT_INFO),
                ));
            });
    });
}

fn handle_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<TotemCloseButton>)>,
    ui_query: Query<(Entity, Option<&PanelWrapperRef>), With<TotemUiRoot>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for (ui_entity, wrapper_ref) in ui_query.iter() {
                if let Some(wrapper) = wrapper_ref {
                    commands.entity(wrapper.0).despawn();
                } else {
                    commands.entity(ui_entity).despawn();
                }
            }
        }
    }
}
