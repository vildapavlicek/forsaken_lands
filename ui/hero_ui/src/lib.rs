use bevy::prelude::*;
use states::GameState;
use widgets::{spawn_card, spawn_effect_display, spawn_primary_button, spawn_stat_display};

pub struct HeroUiPlugin;

impl Plugin for HeroUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Running), setup_hero_ui)
            .add_systems(OnExit(GameState::Running), cleanup_hero_ui);
    }
}

#[derive(Component)]
struct HeroUiRoot;

fn setup_hero_ui(mut commands: Commands) {
    commands
        .spawn((
            HeroUiRoot,
            Node {
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(16, 22, 34)),
        ))
        .with_children(|parent| {
            // Header
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("CYBER-KNIGHT"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Content
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(24.0),
                    width: Val::Percent(100.0),
                    max_width: Val::Px(500.0),
                    ..default()
                })
                .with_children(|content| {
                    spawn_card(
                        content,
                        UiRect::all(Val::Px(12.0)),
                        |card_content| {
                            card_content
                                .spawn(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(16.0),
                                    ..default()
                                })
                                .with_children(|equipped_section| {
                                    equipped_section.spawn((
                                        Node {
                                            width: Val::Px(56.0),
                                            height: Val::Px(56.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                        BorderColor::all(Color::srgba(0.2, 0.5, 1.0, 0.3)),
                                    ));
                                    equipped_section
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            flex_grow: 1.0,
                                            ..default()
                                        })
                                        .with_children(|text_section| {
                                            text_section.spawn((
                                                Text::new("Equipped Weapon"),
                                                TextFont {
                                                    font_size: 12.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb_u8(156, 163, 175)),
                                            ));
                                            text_section.spawn((
                                                Text::new("Pulse Laser Rifle MK-IV"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    equipped_section
                                        .spawn((
                                            Button,
                                            Node {
                                                height: Val::Px(36.0),
                                                padding: UiRect::horizontal(Val::Px(16.0)),
                                                border: UiRect::all(Val::Px(1.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                                        ))
                                        .with_children(|button| {
                                            button.spawn((
                                                Text::new("Change"),
                                                TextFont {
                                                    font_size: 12.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
                        },
                    );

                    content
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|stats_section| {
                            stats_section
                                .spawn(Node {
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    width: Val::Percent(100.0),
                                    ..default()
                                })
                                .with_children(|title_bar| {
                                    title_bar.spawn((
                                        Text::new("WEAPON STATS"),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                                    ));
                                    title_bar.spawn((
                                        Text::new("View Details"),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb_u8(19, 91, 236)),
                                    ));
                                });

                            stats_section
                                .spawn(Node {
                                    display: Display::Grid,
                                    grid_template_columns: RepeatedGridTrack::flex(3, 1.0),
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|grid| {
                                    spawn_stat_display(
                                        grid,
                                        "Damage",
                                        "450",
                                        Color::srgba(0.8, 0.2, 0.2, 0.2),
                                    );
                                    spawn_stat_display(
                                        grid,
                                        "Range",
                                        "50m",
                                        Color::srgba(0.2, 0.8, 0.2, 0.2),
                                    );
                                    spawn_stat_display(
                                        grid,
                                        "Speed",
                                        "1.2s",
                                        Color::srgba(0.8, 0.8, 0.2, 0.2),
                                    );
                                });
                        });

                    content
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|effects_section| {
                            effects_section.spawn((
                                Text::new("PASSIVE EFFECTS"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                            ));

                            spawn_effect_display(
                                effects_section,
                                "Shield Piercer",
                                "Attacks ignore 20% of target's energy shield.",
                                Color::srgb_u8(19, 91, 236),
                            );
                            spawn_effect_display(
                                effects_section,
                                "Static Discharge",
                                "15% chance to stun enemies for 0.5s on hit.",
                                Color::srgb_u8(245, 158, 11),
                            );
                        });

                    content.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });

                    content
                        .spawn(Node {
                            padding: UiRect::top(Val::Px(16.0)),
                            ..default()
                        })
                        .with_children(|button_container| {
                            spawn_primary_button(button_container, "UPGRADE WEAPON");
                        });
                });
        });
}

fn cleanup_hero_ui(mut commands: Commands, query: Query<Entity, With<HeroUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
