use bevy::prelude::*;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_interaction_system);
    }
}

// ============================================================================
// Theme / Colors
// ============================================================================

/// Centralized UI color palette for consistent styling
pub struct UiTheme;

impl UiTheme {
    pub const PANEL_BG: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
    pub const POPUP_BG: Color = Color::srgba(0.1, 0.1, 0.2, 0.9);
    pub const CARD_BG: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);
    pub const CARD_BORDER: Color = Color::srgba(0.3, 0.3, 0.3, 1.0);
    pub const POPUP_BORDER: Color = Color::srgba(0.3, 0.3, 0.5, 1.0);

    pub const TEXT_PRIMARY: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::srgba(0.8, 0.8, 0.8, 1.0);
    pub const TEXT_HEADER: Color = Color::srgba(0.8, 0.8, 1.0, 1.0);
    pub const TEXT_INFO: Color = Color::srgba(0.7, 0.7, 1.0, 1.0);

    pub const BUTTON_NORMAL: Color = Color::srgba(0.2, 0.2, 0.2, 1.0);
    pub const BUTTON_HOVER: Color = Color::srgba(0.3, 0.3, 0.3, 1.0);
    pub const BUTTON_PRESSED: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);

    pub const CLOSE_BUTTON_BG: Color = Color::srgba(0.8, 0.2, 0.2, 0.8);

    pub const AFFORDABLE: Color = Color::srgba(0.7, 1.0, 0.7, 1.0);
    pub const NOT_AFFORDABLE: Color = Color::srgba(1.0, 0.7, 0.7, 1.0);

    pub const BORDER_SUCCESS: Color = Color::srgba(0.0, 1.0, 0.0, 1.0);
    pub const BORDER_ERROR: Color = Color::srgba(1.0, 0.0, 0.0, 1.0);
    pub const BORDER_DISABLED: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);

    pub const TAB_ACTIVE_BG: Color = Color::srgba(0.3, 0.3, 0.4, 1.0);
    pub const TAB_INACTIVE_BG: Color = Color::srgba(0.15, 0.15, 0.2, 1.0);
    pub const TAB_BORDER: Color = Color::srgba(0.4, 0.4, 0.5, 1.0);
}

// ============================================================================
// Panel Position
// ============================================================================

/// Defines panel positioning on screen
pub enum PanelPosition {
    /// Fixed distance from left edge
    Left(f32),
    /// Fixed distance from right edge
    Right(f32),
    /// Centered popup (horizontally centered with top offset)
    CenterPopup { top: f32 },
}

// ============================================================================
// Animated Button
// ============================================================================

#[derive(Component)]
pub struct AnimatedButton {
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
}

#[allow(clippy::type_complexity)]
fn button_interaction_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &AnimatedButton, Option<&mut Transform>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color, anim, mut transform) in query.iter_mut() {
        let (color, scale) = match *interaction {
            Interaction::Pressed => (anim.pressed_color, 0.98),
            Interaction::Hovered => (anim.hover_color, 1.05),
            Interaction::None => (anim.normal_color, 1.0),
        };

        *bg_color = BackgroundColor(color);

        if let Some(tf) = transform.as_mut() {
            tf.scale = Vec3::splat(scale);
        }
    }
}

// ============================================================================
// Panel Widget
// ============================================================================

/// Spawns a styled UI panel/window with consistent styling.
/// Returns the Entity so callers can add children via `commands.entity(id).with_children(...)`.
pub fn spawn_ui_panel<M: Component>(
    commands: &mut Commands,
    position: PanelPosition,
    width: f32,
    height: Val,
    root_marker: M,
) -> Entity {
    let (left, right, margin_left, bg_color) = match position {
        PanelPosition::Left(offset) => (
            Val::Px(offset),
            Val::Auto,
            Val::Px(0.0),
            UiTheme::PANEL_BG,
        ),
        PanelPosition::Right(offset) => (
            Val::Auto,
            Val::Px(offset),
            Val::Px(0.0),
            UiTheme::PANEL_BG,
        ),
        PanelPosition::CenterPopup { top: _ } => (
            Val::Percent(50.0),
            Val::Auto,
            Val::Px(-width / 2.0), // Center by offsetting half width
            UiTheme::POPUP_BG,
        ),
    };

    let top = match position {
        PanelPosition::CenterPopup { top } => Val::Px(top),
        _ => Val::Px(10.0),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left,
                right,
                top,
                width: Val::Px(width),
                height,
                margin: UiRect::left(margin_left),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            root_marker,
            Pickable::default(),
            Interaction::default(),
        ))
        .id()
}

// ============================================================================
// Panel Header Widget
// ============================================================================

/// Spawns a header row with title. Use with_children to add close button if needed.
pub fn spawn_panel_header(parent: &mut ChildSpawnerCommands, title: &str) -> Entity {
    parent
        .spawn(Node {
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|header| {
            header.spawn((
                Text::new(title),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_HEADER),
            ));
        })
        .id()
}

/// Spawns a header row with title and close button in one call.
pub fn spawn_panel_header_with_close<M: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    close_marker: M,
) {
    parent
        .spawn(Node {
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|header| {
            header.spawn((
                Text::new(title),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_HEADER),
            ));

            spawn_close_button(header, close_marker);
        });
}

// ============================================================================
// Close Button Widget
// ============================================================================

/// Spawns a styled close button (X button)
pub fn spawn_close_button<M: Component>(parent: &mut ChildSpawnerCommands, marker: M) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(UiTheme::CLOSE_BUTTON_BG),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("X"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

// ============================================================================
// Scrollable Container Widget
// ============================================================================

/// Spawns a scrollable flex column container for list content.
pub fn spawn_scrollable_container<M: Component>(parent: &mut ChildSpawnerCommands, marker: M) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip(),
            flex_grow: 1.0,
            ..default()
        },
        marker,
    ));
}

// ============================================================================
// Item Card Widget
// ============================================================================

/// Spawns a styled item card. Returns Entity for adding children via with_children.
pub fn spawn_item_card<M: Component>(parent: &mut ChildSpawnerCommands, marker: M) -> Entity {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::bottom(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(UiTheme::CARD_BORDER),
            BackgroundColor(UiTheme::CARD_BG),
            marker,
        ))
        .id()
}

// ============================================================================
// Text Widgets
// ============================================================================

/// Spawns a title text for cards
pub fn spawn_card_title(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(UiTheme::TEXT_PRIMARY),
    ));
}

/// Spawns a description text
pub fn spawn_description_text(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(UiTheme::TEXT_SECONDARY),
    ));
}

/// Spawns a cost text display showing resource requirements.
/// Text is colored green if affordable, red if not.
pub fn spawn_cost_text(parent: &mut ChildSpawnerCommands, cost_str: &str, can_afford: bool) {
    parent.spawn((
        Text::new(cost_str),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(if can_afford {
            UiTheme::AFFORDABLE
        } else {
            UiTheme::NOT_AFFORDABLE
        }),
    ));
}

/// Spawns a timer text display showing duration in seconds.
pub fn spawn_timer_text(parent: &mut ChildSpawnerCommands, seconds: f32) {
    parent.spawn((
        Text::new(format!("Time: {}s", seconds)),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(UiTheme::TEXT_INFO),
    ));
}

// ============================================================================
// Action Button Widget
// ============================================================================

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
            BackgroundColor(UiTheme::BUTTON_NORMAL),
            AnimatedButton {
                normal_color: UiTheme::BUTTON_NORMAL,
                hover_color: UiTheme::BUTTON_HOVER,
                pressed_color: UiTheme::BUTTON_PRESSED,
            },
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

// ============================================================================
// Tab Bar Widget
// ============================================================================

/// Spawns a horizontal tab bar container. Returns Entity for adding tab buttons.
pub fn spawn_tab_bar(parent: &mut ChildSpawnerCommands) -> Entity {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .id()
}

/// Spawns a tab button with active/inactive styling.
pub fn spawn_tab_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    is_active: bool,
    marker: M,
) {
    let bg_color = if is_active {
        UiTheme::TAB_ACTIVE_BG
    } else {
        UiTheme::TAB_INACTIVE_BG
    };

    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::right(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(UiTheme::TAB_BORDER),
            BackgroundColor(bg_color),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}

// ============================================================================
// Icon Button Widget
// ============================================================================

/// Spawns a small icon-style button (e.g., for opening panels)
pub fn spawn_icon_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    icon_text: &str,
    marker: M,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                margin: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(UiTheme::CARD_BORDER),
            BackgroundColor(UiTheme::BUTTON_NORMAL),
            AnimatedButton {
                normal_color: UiTheme::BUTTON_NORMAL,
                hover_color: UiTheme::BUTTON_HOVER,
                pressed_color: UiTheme::BUTTON_PRESSED,
            },
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(icon_text),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}

// ============================================================================
// Menu Button Widget
// ============================================================================

/// Spawns a large menu button for navigation (e.g., in village menu)
pub fn spawn_menu_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    marker: M,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(UiTheme::TAB_BORDER),
            BackgroundColor(UiTheme::BUTTON_NORMAL),
            AnimatedButton {
                normal_color: UiTheme::BUTTON_NORMAL,
                hover_color: UiTheme::BUTTON_HOVER,
                pressed_color: UiTheme::BUTTON_PRESSED,
            },
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}
