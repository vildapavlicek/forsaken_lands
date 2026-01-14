//! Universal notification UI system.
//! Shows toast-style notifications at the top of the screen with stacking support.
//! Handles unlock achievements, research completions, and other notification events.

use {
    bevy::prelude::*,
    research::ResearchCompleted,
    states::GameState,
    unlocks::UnlockAchieved,
    widgets::UiTheme,
};

/// Duration in seconds to show each notification
const NOTIFICATION_DURATION: f32 = 5.0;
/// Height of each notification in pixels
const NOTIFICATION_HEIGHT: f32 = 50.0;
/// Gap between notifications
const NOTIFICATION_GAP: f32 = 8.0;
/// Top offset for the first notification
const NOTIFICATION_TOP_OFFSET: f32 = 10.0;
/// Maximum number of concurrent notifications
const MAX_NOTIFICATIONS: usize = 5;

pub struct NotificationUiPlugin;

impl Plugin for NotificationUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationQueue>()
            .add_observer(on_unlock_achieved)
            .add_observer(on_research_completed)
            .add_systems(OnExit(GameState::Loading), cleanup_loading_notifications)
            .add_systems(
                Update,
                (
                    spawn_pending_notifications,
                    update_notification_positions,
                    despawn_expired_notifications,
                )
                    .chain()
                    .run_if(in_state(GameState::Running)),
            );
    }
}

// ============================================================================
// Types
// ============================================================================

/// Data for a pending notification
#[derive(Clone)]
pub struct NotificationData {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
}

/// Type of notification affects styling
#[derive(Clone, Copy, Default, Debug)]
pub enum NotificationType {
    #[default]
    Info,
    Unlock,
    Research,
}

impl NotificationType {
    /// Get the background color for this notification type
    fn background_color(&self) -> Color {
        match self {
            NotificationType::Info => UiTheme::PANEL_BG,
            NotificationType::Unlock => Color::srgba(0.1, 0.15, 0.1, 0.9),
            NotificationType::Research => Color::srgba(0.1, 0.1, 0.2, 0.9),
        }
    }

    /// Get the border color for this notification type
    fn border_color(&self) -> Color {
        match self {
            NotificationType::Info => UiTheme::CARD_BORDER,
            NotificationType::Unlock => Color::srgba(0.3, 0.7, 0.3, 1.0),
            NotificationType::Research => Color::srgba(0.4, 0.4, 0.8, 1.0),
        }
    }
}

// ============================================================================
// Resources and Components
// ============================================================================

/// Resource managing active and pending notifications
#[derive(Resource, Default)]
pub struct NotificationQueue {
    /// Entities of currently displayed notifications (in order, oldest first)
    active: Vec<Entity>,
    /// Pending notifications to spawn (when space is available)
    pending: Vec<NotificationData>,
}

impl NotificationQueue {
    /// Queue a new notification to be displayed
    pub fn push(&mut self, data: NotificationData) {
        self.pending.push(data);
    }
}

/// Marker component for individual notifications
#[derive(Component)]
struct Notification {
    timer: Timer,
}

// ============================================================================
// Event Observers
// ============================================================================

/// Responds to UnlockAchieved events by queueing a notification
fn on_unlock_achieved(trigger: On<UnlockAchieved>, mut queue: ResMut<NotificationQueue>) {
    let event = trigger.event();
    let message = event
        .display_name
        .clone()
        .unwrap_or_else(|| event.unlock_id.clone());

    queue.push(NotificationData {
        title: "Unlocked".to_string(),
        message,
        notification_type: NotificationType::Unlock,
    });
}

/// Responds to ResearchCompleted events by queueing a notification
fn on_research_completed(trigger: On<ResearchCompleted>, mut queue: ResMut<NotificationQueue>) {
    queue.push(NotificationData {
        title: "Research Completed".to_string(),
        message: trigger.event().research_id.clone(),
        notification_type: NotificationType::Research,
    });
}

// ============================================================================
// Systems
// ============================================================================

/// Cleans up any notifications spawned during loading
fn cleanup_loading_notifications(
    mut commands: Commands,
    query: Query<Entity, With<Notification>>,
    mut queue: ResMut<NotificationQueue>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    queue.active.clear();
    queue.pending.clear();
}

/// Spawns pending notifications if we have room
fn spawn_pending_notifications(mut commands: Commands, mut queue: ResMut<NotificationQueue>) {
    while !queue.pending.is_empty() && queue.active.len() < MAX_NOTIFICATIONS {
        let notification = queue.pending.remove(0);
        let entity = spawn_notification(&mut commands, &notification, queue.active.len());
        queue.active.push(entity);
    }
}

/// Updates positions of all active notifications based on their index
fn update_notification_positions(
    queue: Res<NotificationQueue>,
    mut query: Query<&mut Node, With<Notification>>,
) {
    for (index, &entity) in queue.active.iter().enumerate() {
        if let Ok(mut node) = query.get_mut(entity) {
            node.top = Val::Px(calculate_top_position(index));
        }
    }
}

/// Despawns notifications after their timer expires and reindexes the queue
fn despawn_expired_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Notification)>,
    mut queue: ResMut<NotificationQueue>,
) {
    // Collect expired entities
    let mut expired = Vec::new();
    for (entity, mut notification) in query.iter_mut() {
        notification.timer.tick(time.delta());
        if notification.timer.is_finished() {
            expired.push(entity);
        }
    }

    // Remove expired from active list and despawn
    for entity in expired {
        queue.active.retain(|&e| e != entity);
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates the top position for a notification based on its index
fn calculate_top_position(index: usize) -> f32 {
    NOTIFICATION_TOP_OFFSET + (index as f32) * (NOTIFICATION_HEIGHT + NOTIFICATION_GAP)
}

/// Spawns a notification entity
fn spawn_notification(
    commands: &mut Commands,
    notification: &NotificationData,
    index: usize,
) -> Entity {
    let display_text = format!("{}: {}", notification.title, notification.message);

    commands
        .spawn((
            Text::new(display_text),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(calculate_top_position(index)),
                left: Val::Percent(20.0),
                right: Val::Percent(20.0),
                height: Val::Px(NOTIFICATION_HEIGHT),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            TextColor(UiTheme::TEXT_PRIMARY),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            BackgroundColor(notification.notification_type.background_color()),
            BorderColor::all(notification.notification_type.border_color()),
            BorderRadius::all(Val::Px(8.0)),
            Notification {
                timer: Timer::from_seconds(NOTIFICATION_DURATION, TimerMode::Once),
            },
        ))
        .id()
}
