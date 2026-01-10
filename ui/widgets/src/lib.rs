use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};

/// Line height for scroll calculations (pixels per line)
const SCROLL_LINE_HEIGHT: f32 = 21.0;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (button_interaction_system, send_scroll_events))
            .add_observer(on_scroll_handler);
    }
}

// ============================================================================
// Scroll Handling
// ============================================================================

/// UI scrolling event that propagates through the hierarchy.
#[derive(EntityEvent, Debug, Clone)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    /// The target entity for this scroll event.
    #[event_target]
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

/// Converts mouse wheel input into scroll events on hovered UI entities.
fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        // Convert line units to pixels
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= SCROLL_LINE_HEIGHT;
        }

        // Swap axes if Ctrl is held (horizontal scroll)
        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        // Trigger scroll event on all hovered entities
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// Handles scroll events by updating ScrollPosition on scrollable containers.
fn on_scroll_handler(
    scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = scroll.delta;

    // Handle horizontal scrolling
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        let at_max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !at_max {
            scroll_position.x = (scroll_position.x + delta.x).clamp(0.0, max_offset.x.max(0.0));
        }
    }

    // Handle vertical scrolling
    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        let at_max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !at_max {
            scroll_position.y = (scroll_position.y + delta.y).clamp(0.0, max_offset.y.max(0.0));
        }
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
// Panel Configuration
// ============================================================================

/// Configuration for panel position and size.
/// Supports flexible positioning using Bevy's `Val` type for viewport-relative
/// or fixed pixel dimensions.
pub struct PanelConfig {
    /// Width of the panel (supports Val::Px, Val::Percent, Val::Vw, etc.)
    pub width: Val,
    /// Height of the panel
    pub height: Val,
    /// Left position (use Val::Auto for right-aligned or centered panels)
    pub left: Val,
    /// Right position (use Val::Auto for left-aligned or centered panels)
    pub right: Val,
    /// Top position
    pub top: Val,
    /// Bottom position (use Val::Auto for top-positioned panels)
    pub bottom: Val,
    /// Background color for the panel
    pub background: Color,
}

impl PanelConfig {
    /// Centered panel with percentage-based viewport dimensions.
    /// Uses viewport width (vw) and viewport height (vh) for responsive sizing.
    pub fn centered(width_percent: f32, height_percent: f32) -> Self {
        Self {
            width: Val::Vw(width_percent),
            height: Val::Vh(height_percent),
            left: Val::Percent(50.0),
            right: Val::Auto,
            top: Val::Percent(50.0),
            bottom: Val::Auto,
            background: UiTheme::PANEL_BG,
        }
    }

    /// Popup-style centered panel with fixed pixel dimensions.
    /// Positioned with a fixed offset from the top.
    pub fn popup(width: f32, height: f32) -> Self {
        Self {
            width: Val::Px(width),
            height: Val::Px(height),
            left: Val::Percent(50.0),
            right: Val::Auto,
            top: Val::Px(50.0),
            bottom: Val::Auto,
            background: UiTheme::POPUP_BG,
        }
    }

    /// Left-aligned panel with fixed width and height stretching.
    pub fn left_panel(offset: f32, width: f32) -> Self {
        Self {
            width: Val::Px(width),
            height: Val::Auto,
            left: Val::Px(offset),
            right: Val::Auto,
            top: Val::Px(10.0),
            bottom: Val::Px(10.0),
            background: UiTheme::PANEL_BG,
        }
    }

    /// Right-aligned panel with fixed width and height stretching.
    pub fn right_panel(offset: f32, width: f32) -> Self {
        Self {
            width: Val::Px(width),
            height: Val::Auto,
            left: Val::Auto,
            right: Val::Px(offset),
            top: Val::Px(10.0),
            bottom: Val::Px(10.0),
            background: UiTheme::PANEL_BG,
        }
    }
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
        (
            &Interaction,
            &mut BackgroundColor,
            &AnimatedButton,
            Option<&mut Transform>,
        ),
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

/// Marker for the wrapper container used for centering panels.
/// This is used internally and should not be added manually.
#[derive(Component)]
pub struct UiPanelWrapper;

/// Reference to the wrapper entity that contains this panel.
/// Used for cleanup when the panel is despawned.
#[derive(Component)]
pub struct PanelWrapperRef(pub Entity);

/// Spawns a styled UI panel/window with consistent styling.
/// Returns the Entity so callers can add children via `commands.entity(id).with_children(...)`.
///
/// For centered panels (using `Val::Percent(50.0)` for left/top), this spawns a full-screen
/// wrapper that uses flexbox to center the panel properly. The wrapper is automatically
/// despawned when the panel entity is despawned (if using `.despawn()` with recursive despawn).
///
/// **Important**: When despawning a centered panel, use `despawn()` rather than
/// `despawn_descendants()` to ensure the wrapper is cleaned up. Alternatively,
/// query for `Parent` and despawn that if you need finer control.
pub fn spawn_ui_panel<M: Component>(
    commands: &mut Commands,
    config: PanelConfig,
    root_marker: M,
) -> Entity {
    // Determine if we need flexbox centering (when using 50% positioning)
    let needs_center_x = matches!(config.left, Val::Percent(50.0));
    let needs_center_y = matches!(config.top, Val::Percent(50.0));

    if needs_center_x || needs_center_y {
        // For centered panels, spawn a full-screen wrapper with flexbox centering
        let wrapper = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: if needs_center_x {
                        JustifyContent::Center
                    } else {
                        JustifyContent::FlexStart
                    },
                    align_items: if needs_center_y {
                        AlignItems::Center
                    } else {
                        AlignItems::FlexStart
                    },
                    ..default()
                },
                UiPanelWrapper,
                // Make wrapper non-pickable so clicks pass through to game world
                Pickable::IGNORE,
            ))
            .id();

        // Spawn the actual panel as a child of the wrapper
        let panel = commands
            .spawn((
                Node {
                    width: config.width,
                    height: config.height,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(config.background),
                root_marker,
                PanelWrapperRef(wrapper),
                Pickable::default(),
                Interaction::default(),
            ))
            .id();

        commands.entity(wrapper).add_child(panel);
        panel
    } else {
        // For non-centered panels, use direct absolute positioning
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: config.left,
                    right: config.right,
                    top: config.top,
                    bottom: config.bottom,
                    width: config.width,
                    height: config.height,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(config.background),
                root_marker,
                Pickable::default(),
                Interaction::default(),
            ))
            .id()
    }
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
            overflow: Overflow::scroll_y(),
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ScrollPosition::default(),
        Interaction::default(),
        marker,
    ));
}

// ============================================================================
// Item Card Widget
// ============================================================================

/// Spawns a styled item card. Returns Entity for adding children via with_children.
pub fn spawn_item_card<M: Bundle>(parent: &mut ChildSpawnerCommands, marker: M) -> Entity {
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
            Pickable::IGNORE,
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
pub fn spawn_menu_button<M: Component>(parent: &mut ChildSpawnerCommands, text: &str, marker: M) {
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
