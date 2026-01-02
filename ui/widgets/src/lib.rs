use bevy::prelude::*;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_interaction_system);
    }
}

#[derive(Component)]
pub struct AnimatedButton {
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
}

fn button_interaction_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &AnimatedButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color, anim) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(anim.pressed_color);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(anim.hover_color);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(anim.normal_color);
            }
        }
    }
}

/// Spawns a cost text display showing resource requirements.
/// Text is colored green if affordable, red if not.
pub fn spawn_cost_text(parent: &mut ChildSpawnerCommands, cost_str: &str, can_afford: bool) {
    parent.spawn((
        Text::new(cost_str),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(if can_afford {
            Color::srgba(0.7, 1.0, 0.7, 1.0)
        } else {
            Color::srgba(1.0, 0.7, 0.7, 1.0)
        }),
    ));
}

/// Spawns a timer text display showing duration in seconds.
pub fn spawn_timer_text(parent: &mut ChildSpawnerCommands, seconds: f32) {
    parent.spawn((
        Text::new(&format!("Time: {}s", seconds)),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgba(0.7, 0.7, 1.0, 1.0)),
    ));
}

/// Spawns an action button with customizable text, colors, and a marker component.
pub fn spawn_action_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    text_color: Color,
    border_color: Color,
    marker: M,
) {
    let normal_color = Color::srgba(0.2, 0.2, 0.2, 1.0);
    let hover_color = Color::srgba(0.3, 0.3, 0.3, 1.0);
    let pressed_color = Color::srgba(0.1, 0.1, 0.1, 1.0);

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(30.0),
                margin: UiRect::top(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(border_color),
            BackgroundColor(normal_color),
            AnimatedButton {
                normal_color,
                hover_color,
                pressed_color,
            },
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}
