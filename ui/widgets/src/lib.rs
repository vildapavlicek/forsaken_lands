use bevy::prelude::*;

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
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
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

pub fn spawn_card<F>(parent: &mut ChildSpawnerCommands, padding: UiRect, children: F)
where
    F: FnOnce(&mut ChildSpawnerCommands),
{
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.02)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
            BorderRadius::all(Val::Px(12.0)),
        ))
        .with_children(children);
}

pub fn spawn_stat_display(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    icon_bg: Color,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.02)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
            BorderRadius::all(Val::Px(12.0)),
        ))
        .with_children(|container| {
            container.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                BackgroundColor(icon_bg),
                BorderRadius::all(Val::Px(16.0)),
            ));

            container.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(156, 163, 175)),
            ));

            container.spawn((
                Text::new(value),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn spawn_effect_display(parent: &mut ChildSpawnerCommands, title: &str, desc: &str, icon_color: Color) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                column_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.02)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
            BorderRadius::all(Val::Px(12.0)),
        ))
        .with_children(|container| {
            container.spawn((
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    margin: UiRect::top(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(icon_color),
                BorderRadius::all(Val::Px(4.0)),
            ));

            container
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|text_container| {
                    text_container.spawn((
                        Text::new(title),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    text_container.spawn((
                        Text::new(desc),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(156, 163, 175)),
                    ));
                });
        });
}

pub fn spawn_primary_button(parent: &mut ChildSpawnerCommands, text: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(19, 91, 236)),
            BorderRadius::all(Val::Px(12.0)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
