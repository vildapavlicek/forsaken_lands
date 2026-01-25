use {
    bevy::prelude::*,
    states::{GameState, VillageView},
    village_components::EnemyEncyclopedia,
    widgets::{spawn_menu_button, ContentContainer},
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
        spawn_enemy_encyclopedia_content(parent, encyclopedia);
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
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            EncyclopediaListContainer,
        ))
        .with_children(|list| {
            // Title
            list.spawn((
                Text::new("Enemy Encyclopedia"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // List of enemies
            for (_enemy_id, entry) in &encyclopedia.inner {
                list.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    padding: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((Text::new(entry.display_name.clone()), TextColor(Color::WHITE)));

                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(15.0),
                        ..default()
                    })
                    .with_children(|stats| {
                        stats.spawn((
                            Text::new(format!("Kills: {}", entry.kill_count)),
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));

                        stats.spawn((
                            Text::new(format!("Escapes: {}", entry.escape_count)),
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                    });
                });
            }

            if encyclopedia.inner.is_empty() {
                list.spawn((
                    Text::new("No enemies encountered yet."),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                ));
            }
        });
}

fn update_encyclopedia_ui(
    mut commands: Commands,
    encyclopedia_query: Query<&EnemyEncyclopedia, Changed<EnemyEncyclopedia>>,
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

    // Repopulate
    commands.entity(container).with_children(|list| {
        // Title
        list.spawn((
            Text::new("Enemy Encyclopedia"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        // List of enemies
        for (_enemy_id, entry) in &encyclopedia.inner {
            list.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(5.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((Text::new(entry.display_name.clone()), TextColor(Color::WHITE)));

                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|stats| {
                    stats.spawn((
                        Text::new(format!("Kills: {}", entry.kill_count)),
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));

                    stats.spawn((
                        Text::new(format!("Escapes: {}", entry.escape_count)),
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
            });
        }

        if encyclopedia.inner.is_empty() {
            list.spawn((
                Text::new("No enemies encountered yet."),
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
            ));
        }
    });
}
