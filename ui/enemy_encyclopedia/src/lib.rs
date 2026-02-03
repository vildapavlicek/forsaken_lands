use {
    bevy::prelude::*,
    bonus_stats_resources::{BonusStat, BonusStats},
    enemy_resources::EnemyDetailsCache,
    states::{GameState, VillageView},
    village_components::EnemyEncyclopedia,
    wallet::Wallet,
    widgets::{ContentContainer, spawn_menu_button},
};

pub struct EnemyEncyclopediaUiPlugin;

impl Plugin for EnemyEncyclopediaUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(VillageView::Encyclopedia), spawn_encyclopedia_ui)
            .add_systems(
                Update,
                handle_back_button.run_if(in_state(GameState::Running)),
            )
            .add_systems(
                Update,
                update_encyclopedia_ui.run_if(in_state(VillageView::Encyclopedia)),
            );
    }
}

/// Back button to return to menu - we need this marker to match village_ui's behavior
#[derive(Component)]
struct VillageBackButton;

#[derive(Component)]
pub struct EncyclopediaListContainer;

fn spawn_encyclopedia_ui(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&Children>), With<ContentContainer>>,
    encyclopedia_query: Query<&EnemyEncyclopedia>,
    details_cache: Res<EnemyDetailsCache>,
    wallet: Res<Wallet>,
    bonus_stats: Res<BonusStats>,
) {
    let Some((container, children)) = query.iter_mut().next() else {
        return;
    };

    // Despawn existing children
    let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
    for child in to_despawn {
        commands.entity(child).despawn();
    }

    let Some(encyclopedia) = encyclopedia_query.iter().next() else {
        return;
    };

    // Spawn back button and encyclopedia content
    commands.entity(container).with_children(|parent| {
        // Back button
        spawn_menu_button(parent, "← Back", VillageBackButton, true);

        // Spawn encyclopedia content
        spawn_enemy_encyclopedia_content(
            parent,
            encyclopedia,
            &details_cache,
            &wallet,
            &bonus_stats,
        );
    });
}

// Back button handler (needed since we spawn it)
fn handle_back_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<VillageBackButton>)>,
    mut next_state: ResMut<NextState<VillageView>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(VillageView::Menu);
        }
    }
}

pub fn spawn_enemy_encyclopedia_content(
    parent: &mut ChildSpawnerCommands,
    encyclopedia: &EnemyEncyclopedia,
    details_cache: &EnemyDetailsCache,
    wallet: &Wallet,
    bonus_stats: &BonusStats,
) {
    // Collect and sort entries
    let mut entries: Vec<(&String, &village_components::EncyclopediaEntry)> =
        encyclopedia.inner.iter().collect();
    entries.sort_by_key(|(_, entry)| entry.encounter_order);

    // Use widgets scrollable container
    widgets::spawn_scrollable_container(parent, EncyclopediaListContainer, |scroll_content| {
        populate_encyclopedia_list(scroll_content, &entries, details_cache, wallet, bonus_stats);
    });
}

fn populate_encyclopedia_list(
    parent: &mut ChildSpawnerCommands,
    entries: &[(&String, &village_components::EncyclopediaEntry)],
    details_cache: &EnemyDetailsCache,
    wallet: &Wallet,
    bonus_stats: &BonusStats,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },))
        .with_children(|list| {
            // Title
            list.spawn((
                Text::new("Enemy Encyclopedia"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Grid Container for cards
            list.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(10.0),
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|grid| {
                // List of enemies
                for (enemy_id, entry) in entries {
                    spawn_enemy_card(grid, entry, enemy_id, details_cache, wallet, bonus_stats);
                }

                if entries.is_empty() {
                    grid.spawn((
                        Text::new("No enemies encountered yet."),
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                    ));
                }
            });
        });
}

fn spawn_enemy_card(
    parent: &mut ChildSpawnerCommands,
    entry: &village_components::EncyclopediaEntry,
    enemy_id: &str,
    details_cache: &EnemyDetailsCache,
    wallet: &Wallet,
    bonus_stats: &BonusStats,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(250.0),
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        })
        .insert(BorderColor::all(Color::srgb(0.3, 0.3, 0.3)))
        .insert(BackgroundColor(Color::srgb(0.15, 0.15, 0.15)))
        .with_children(|card| {
            // Name
            card.spawn((
                Text::new(entry.display_name.clone()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Basic Stats (Kills/Escapes)
            card.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            })
            .with_children(|stats| {
                stats.spawn((
                    Text::new(format!("Kills: {}", entry.kill_count)),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                ));
                stats.spawn((
                    Text::new(format!("Escapes: {}", entry.escape_count)),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                ));
            });

            // Advanced Stats (from cache)
            if let Some(details) = details_cache.details.get(enemy_id) {
                card.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|details_node| {
                    details_node.spawn((
                        Text::new(format!("♥ Max Health: {:.1}", details.health)),
                        TextColor(Color::srgb(0.4, 1.0, 0.4)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                    ));
                    details_node.spawn((
                        Text::new(format!("⏩ Speed: {:.1}", details.speed)),
                        TextColor(Color::srgb(0.4, 0.8, 1.0)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                    ));

                    if !details.drops.is_empty() {
                        details_node.spawn((
                            Text::new("Drops:"),
                            TextColor(Color::srgb(1.0, 0.84, 0.0)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            Node {
                                margin: UiRect::top(Val::Px(2.0)),
                                ..default()
                            },
                        ));
                        for drop in &details.drops {
                            let drop_text = if wallet.unlocked_resources.contains(drop) {
                                format!(" • {}", drop)
                            } else {
                                format!(" • Unidentified")
                            };
                            details_node.spawn((
                                Text::new(drop_text),
                                TextColor(Color::srgb(1.0, 1.0, 0.8)),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                            ));
                        }
                    }

                    // Bonus Stats
                    let mut total = BonusStat::default();
                    for tag in &details.tags {
                        let key = format!("damage:{}", tag);
                        if let Some(stat) = bonus_stats.get(&key) {
                            total = total + *stat;
                        }
                    }

                    // Only display if there's any bonus
                    if total.additive != 0.0 || total.percent != 0.0 || total.multiplicative > 0.0 {
                        let mult_val = total.multiplicative.max(1.0);
                        let text = format!(
                            "Bonus: +{}/{:.0}%/*{}",
                            total.additive,
                            total.percent * 100.0,
                            mult_val
                        );

                        details_node.spawn((
                            Text::new(text),
                            TextColor(Color::srgb(1.0, 0.5, 0.5)), // Red-ish for damage?
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            Node {
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                        ));
                    }
                });
            } else {
                // Locked info
                card.spawn((
                    Text::new("Stats: ???\n(Research required)"),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }
        });
}

fn update_encyclopedia_ui(
    mut commands: Commands,
    encyclopedia_query: Query<&EnemyEncyclopedia, Changed<EnemyEncyclopedia>>,
    details_cache: Res<EnemyDetailsCache>,
    wallet: Res<Wallet>,
    bonus_stats: Res<BonusStats>,
    container_query: Query<(Entity, &Children), With<EncyclopediaListContainer>>,
) {
    let Some(encyclopedia) = encyclopedia_query.iter().next() else {
        return;
    };

    let Some((container, children)) = container_query.iter().next() else {
        return;
    };

    // Despawn old content
    for &child in children {
        commands.entity(child).despawn();
    }

    // Collect and sort entries
    let mut entries: Vec<(&String, &village_components::EncyclopediaEntry)> =
        encyclopedia.inner.iter().collect();
    entries.sort_by_key(|(_, entry)| entry.encounter_order);

    // Repopulate
    commands.entity(container).with_children(|scroll_content| {
        populate_encyclopedia_list(
            scroll_content,
            &entries,
            &details_cache,
            &wallet,
            &bonus_stats,
        );
    });
}
