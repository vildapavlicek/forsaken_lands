//! Simple unlock notification UI.
//! Shows a toast-style notification at the top center of the screen for 5 seconds when an unlock happens.

use bevy::prelude::*;
use states::GameState;
use unlocks::UnlockCompletedEvent;
use widgets::UiTheme;

/// Duration in seconds to show the notification
const NOTIFICATION_DURATION: f32 = 5.0;

pub struct UnlockNotificationUiPlugin;

impl Plugin for UnlockNotificationUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_unlock_completed)
            .add_systems(
                Update,
                despawn_expired_notifications.run_if(in_state(GameState::Running)),
            );
    }
}

/// Marker component for the unlock notification UI
#[derive(Component)]
struct UnlockNotification {
    timer: Timer,
}

/// Spawns a notification when an unlock is completed
fn on_unlock_completed(trigger: On<UnlockCompletedEvent>, mut commands: Commands) {
    let event = trigger.event();
    let unlock_id = &event.unlock_id;

    commands.spawn((
        Text::new(format!("Unlocked: {}", unlock_id)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Percent(50.0),
            // Center the text horizontally
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        TextColor(UiTheme::TEXT_PRIMARY),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        BackgroundColor(UiTheme::PANEL_BG),
        BorderRadius::all(Val::Px(8.0)),
        UnlockNotification {
            timer: Timer::from_seconds(NOTIFICATION_DURATION, TimerMode::Once),
        },
    ));
}

/// Despawns notifications after their timer expires
fn despawn_expired_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut UnlockNotification)>,
) {
    for (entity, mut notification) in query.iter_mut() {
        notification.timer.tick(time.delta());
        if notification.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
