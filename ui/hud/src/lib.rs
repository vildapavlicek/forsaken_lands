use {
    bevy::prelude::*,
    components::{EnemyStatusRoot, HpBarFill, TimeBarFill},
    enemy_components::{Dead, Enemy, Health, Lifetime},
    states::GameState,
};

pub mod components;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_enemy_status_bars.run_if(in_state(GameState::Running)),
        )
        .add_observer(spawn_enemy_status_bars);
    }
}

const BAR_WIDTH: f32 = 40.0;
const BAR_HEIGHT: f32 = 4.0;
const HP_BAR_COLOR: Color = Color::srgb(0.0, 1.0, 0.0); // Green
const TIME_BAR_COLOR: Color = Color::srgb(0.0, 0.5, 0.8); // Oceanic Blue
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);

fn spawn_enemy_status_bars(
    trigger: On<Add, Sprite>,
    mut commands: Commands,
    sprite: Query<&Sprite, (With<Enemy>, Without<Dead>)>,
) {
    let parent = trigger.entity;

    let Ok(sprite) = sprite.get(parent) else {
        return;
    };

    let Some(half_size_y) = sprite.custom_size.map(|size| size.y / 2.0) else {
        debug!(
            "Enemy sprite {:?} missing custom_size, skipping status bars",
            parent
        );
        return;
    };

    let time_bar_offset = half_size_y + BAR_HEIGHT + 2.0;
    let hp_bar_y_offset = time_bar_offset + BAR_HEIGHT + 2.0;

    commands.entity(parent).with_children(|parent_cmd| {
        // Root
        parent_cmd
            .spawn((
                EnemyStatusRoot,
                Transform::from_xyz(0.0, 0.0, 10.0), // Ensure it's above the sprite
                Visibility::Inherited,
                Name::new("EnemyStatusRoot"),
            ))
            .with_children(|root| {
                // HP Bar Background
                root.spawn((
                    Sprite {
                        color: BG_COLOR,
                        custom_size: Some(Vec2::new(BAR_WIDTH + 2.0, BAR_HEIGHT + 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, hp_bar_y_offset, 0.0),
                ));

                // HP Bar Fill
                root.spawn((
                    HpBarFill,
                    Sprite {
                        color: HP_BAR_COLOR,
                        custom_size: Some(Vec2::new(BAR_WIDTH, BAR_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, hp_bar_y_offset, 0.1),
                ));

                // Time Bar Background
                root.spawn((
                    Sprite {
                        color: BG_COLOR,
                        custom_size: Some(Vec2::new(BAR_WIDTH + 2.0, BAR_HEIGHT + 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, time_bar_offset, 0.0),
                ));

                // Time Bar Fill
                root.spawn((
                    TimeBarFill,
                    Sprite {
                        color: TIME_BAR_COLOR,
                        custom_size: Some(Vec2::new(BAR_WIDTH, BAR_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, time_bar_offset, 0.1),
                ));
            });
    });
}

fn update_enemy_status_bars(
    mut hp_query: Query<&mut Transform, (With<HpBarFill>, Without<TimeBarFill>)>,
    mut time_query: Query<&mut Transform, (With<TimeBarFill>, Without<HpBarFill>)>,
    root_query: Query<(&ChildOf, &Children), With<EnemyStatusRoot>>,
    enemy_query: Query<(Option<&Health>, Option<&Lifetime>), With<Enemy>>,
) {
    for (child_of, children) in root_query.iter() {
        if let Ok((health, lifetime)) = enemy_query.get(child_of.parent()) {
            // Update HP Bar
            if let Some(health) = health {
                let percent = (health.current / health.max).clamp(0.0, 1.0);
                for child in children.iter() {
                    if let Ok(mut transform) = hp_query.get_mut(child) {
                        transform.scale.x = percent;
                        // Left align center scaling correction (if pivot is center)
                        // Bevy sprites are centered by default. To scale from left, we need to shift x.
                        // Wait, scaling a centered sprite shrinks it towards center.
                        // To look like a bar draining to the left, we need to adjust translation.x
                        // Default width is BAR_WIDTH. New width is BAR_WIDTH * percent.
                        // Difference is BAR_WIDTH * (1.0 - percent).
                        // Shift left by half that difference to keep left edge fixed.
                        transform.translation.x = -(BAR_WIDTH * (1.0 - percent)) / 2.0;
                    }
                }
            } else {
                // No health component, maybe hide bar?
                for child in children.iter() {
                    if let Ok(mut transform) = hp_query.get_mut(child) {
                        transform.scale.x = 0.0;
                    }
                }
            }

            // Update Time Bar
            if let Some(lifetime) = lifetime {
                let percent = lifetime.0.fraction_remaining();
                for child in children.iter() {
                    if let Ok(mut transform) = time_query.get_mut(child) {
                        transform.scale.x = percent;
                        transform.translation.x = -(BAR_WIDTH * (1.0 - percent)) / 2.0;
                    }
                }
            } else {
                // No lifetime, hide bar
                for child in children.iter() {
                    if let Ok(mut transform) = time_query.get_mut(child) {
                        transform.scale.x = 0.0;
                    }
                }
            }
        }
    }
}
