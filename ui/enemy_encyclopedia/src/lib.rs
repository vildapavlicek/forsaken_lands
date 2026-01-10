use {bevy::prelude::*, states::EnemyEncyclopediaState, village_components::EnemyEncyclopedia};

pub struct EnemyEncyclopediaUiPlugin;

impl Plugin for EnemyEncyclopediaUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EnemyEncyclopediaState>().add_systems(
            Update,
            update_encyclopedia_ui.run_if(in_state(EnemyEncyclopediaState::Open)),
        );
    }
}

#[derive(Component)]
pub struct EncyclopediaListContainer;

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
            list.spawn(Text::new("Enemy Encyclopedia"))
                .insert(TextFont {
                    font_size: 24.0,
                    ..default()
                })
                .insert(TextColor(Color::WHITE));

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
                    row.spawn(Text::new(entry.display_name.clone()))
                        .insert(TextColor(Color::WHITE));

                    row.spawn(Text::new(format!("Kills: {}", entry.kill_count)))
                        .insert(TextColor(Color::srgb(0.7, 0.7, 0.7)));
                });
            }

            if encyclopedia.inner.is_empty() {
                list.spawn(Text::new("No enemies encountered yet."))
                    .insert(TextColor(Color::srgb(0.5, 0.5, 0.5)))
                    .insert(TextFont {
                        font_size: 16.0,
                        ..default()
                    });
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
        list.spawn(Text::new("Enemy Encyclopedia"))
            .insert(TextFont {
                font_size: 24.0,
                ..default()
            })
            .insert(TextColor(Color::WHITE));

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
                row.spawn(Text::new(entry.display_name.clone()))
                    .insert(TextColor(Color::WHITE));

                row.spawn(Text::new(format!("Kills: {}", entry.kill_count)))
                    .insert(TextColor(Color::srgb(0.7, 0.7, 0.7)));
            });
        }

        if encyclopedia.inner.is_empty() {
            list.spawn(Text::new("No enemies encountered yet."))
                .insert(TextColor(Color::srgb(0.5, 0.5, 0.5)))
                .insert(TextFont {
                    font_size: 16.0,
                    ..default()
                });
        }
    });
}
