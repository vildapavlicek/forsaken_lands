use bevy::prelude::*;
use hero_components::{AttackRange, AttackSpeed, Damage, Hero, Weapon};
use states::GameState;
use village_components::Village;
use widgets::{spawn_card, spawn_effect_display, spawn_primary_button, spawn_stat_display};

pub struct HeroUiPlugin;

impl Plugin for HeroUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_village_click);
        app.add_systems(Update, handle_close_button.run_if(in_state(GameState::Running)));
    }
}

#[derive(Component)]
struct HeroUiRoot;

fn on_village_click(
    trigger: On<Pointer<Click>>,
    village_query: Query<Entity, With<Village>>,
    hero_query: Query<&Children, With<Hero>>,
    weapon_query: Query<(&Damage, &AttackRange, &AttackSpeed), With<Weapon>>,
    ui_root_query: Query<Entity, With<HeroUiRoot>>,
    mut commands: Commands,
) {
    // Check if the clicked entity is a village
    if village_query.get(trigger.entity).is_err() {
        return;
    }

    // Singleton check
    if !ui_root_query.is_empty() {
        return;
    }

    // Get hero and weapon data
    let Ok(hero_children) = hero_query.single() else {
        error!("Expected a single hero entity");
        return;
    };

    let mut weapon_stats = None;
    for &child in hero_children {
        if let Ok(stats) = weapon_query.get(child) {
            weapon_stats = Some(stats);
            break;
        }
    }

    let Some((damage, range, speed)) = weapon_stats else {
        error!("no weapon for a hero");
        return;
    };

    // Spawn UI
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
            ZIndex(100), // Ensure it's on top
        ))
        .with_children(|parent| {
            // Header
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                ))
                .with_children(|header| {
                    // Empty node for spacing
                    header.spawn(Node::default());

                    header.spawn((
                        Text::new("CYBER-KNIGHT"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    header
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            CloseButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("X"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
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
                    spawn_card(content, UiRect::all(Val::Px(12.0)), |card_content| {
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
                                    BorderRadius::all(Val::Px(8.0)),
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
                                        BorderRadius::all(Val::Px(8.0)),
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
                    });

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
                                        &damage.0.to_string(),
                                        Color::srgba(0.8, 0.2, 0.2, 0.2),
                                    );
                                    spawn_stat_display(
                                        grid,
                                        "Range",
                                        &format!("{:.0}m", range.0),
                                        Color::srgba(0.2, 0.8, 0.2, 0.2),
                                    );
                                    spawn_stat_display(
                                        grid,
                                        "Speed",
                                        &format!("{:.1}s", speed.timer.duration().as_secs_f32()),
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

#[derive(Component)]
struct CloseButton;

fn handle_close_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
    ui_root_query: Query<Entity, With<HeroUiRoot>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in ui_root_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}
