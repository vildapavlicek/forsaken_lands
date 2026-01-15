use {
    bevy::prelude::*,
    hero_components::{AttackRange, AttackSpeed, Damage, MeleeArc, MeleeWeapon, Weapon},
    shared_components::DisplayName,
    states::GameState,
    widgets::{spawn_card_title, spawn_item_card, UiTheme},
};

pub struct HeroUiPlugin;

impl Plugin for HeroUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<HeroUiState>()
            .add_observer(on_hero_ui_added)
            .add_observer(on_hero_ui_removed)
            .add_systems(
                Update,
                update_hero_ui.run_if(in_state(HeroUiState::Open).and(in_state(GameState::Running))),
            );
    }
}

// ============================================================================
// State
// ============================================================================

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HeroUiState {
    #[default]
    Closed,
    Open,
}

// ============================================================================
// Components
// ============================================================================

/// Root marker for the hero UI content
#[derive(Component)]
pub struct HeroUiRoot;

/// Tab button for selecting a hero
#[derive(Component)]
pub struct HeroTabButton {
    pub hero_entity: Entity,
}

/// Currently selected hero for display
#[derive(Component)]
pub struct SelectedHero(pub Entity);

// ============================================================================
// State Observers
// ============================================================================

fn on_hero_ui_added(
    _trigger: On<Add, HeroUiRoot>,
    mut next_state: ResMut<NextState<HeroUiState>>,
) {
    next_state.set(HeroUiState::Open);
}

fn on_hero_ui_removed(
    _trigger: On<Remove, HeroUiRoot>,
    mut next_state: ResMut<NextState<HeroUiState>>,
) {
    next_state.set(HeroUiState::Closed);
}

// ============================================================================
// Update System (placeholder for future dynamic updates)
// ============================================================================

fn update_hero_ui() {
    // Placeholder for future dynamic updates (XP bars, level changes, etc.)
}

// ============================================================================
// Display Data
// ============================================================================

/// Data for displaying weapon stats
pub struct WeaponDisplayData {
    pub name: String,
    pub damage: f32,
    pub range: f32,
    pub speed_secs: f32,
    pub melee_arc: Option<f32>, // In degrees, only for melee weapons
}

/// Data for displaying a hero
pub struct HeroDisplayData {
    pub name: String,
    pub weapon: Option<WeaponDisplayData>,
}

// ============================================================================
// UI Spawn Functions
// ============================================================================

/// Spawns the hero content UI.
/// This is called by village_ui when Heroes content is selected.
pub fn spawn_hero_content(
    parent: &mut ChildSpawnerCommands,
    heroes: Vec<(Entity, HeroDisplayData)>,
    selected_index: usize,
) {
    // Hero tabs container
    if heroes.len() > 1 {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            })
            .with_children(|tabs| {
                for (idx, (entity, data)) in heroes.iter().enumerate() {
                    let is_active = idx == selected_index;
                    spawn_hero_tab(tabs, *entity, &data.name, is_active);
                }
            });
    }

    // Display selected hero details
    if let Some((_, hero_data)) = heroes.get(selected_index) {
        spawn_hero_details(parent, hero_data);
    }
}

fn spawn_hero_tab(
    parent: &mut ChildSpawnerCommands,
    hero_entity: Entity,
    name: &str,
    is_active: bool,
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
            HeroTabButton { hero_entity },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}

fn spawn_hero_details(parent: &mut ChildSpawnerCommands, hero: &HeroDisplayData) {
    // Hero name header card
    let name_card = spawn_item_card(parent, ());
    parent.commands().entity(name_card).with_children(|card| {
        spawn_card_title(card, &hero.name);
    });

    // Weapon section
    if let Some(weapon) = &hero.weapon {
        spawn_weapon_section(parent, weapon);
    } else {
        // No weapon equipped message
        parent.spawn((
            Text::new("No weapon equipped"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(UiTheme::TEXT_SECONDARY),
        ));
    }
}

fn spawn_weapon_section(parent: &mut ChildSpawnerCommands, weapon: &WeaponDisplayData) {
    // Weapon header
    parent.spawn((
        Text::new("Weapon"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(UiTheme::TEXT_HEADER),
        Node {
            margin: UiRect::vertical(Val::Px(8.0)),
            ..default()
        },
    ));

    // Weapon card with stats
    let weapon_card = spawn_item_card(parent, ());
    parent
        .commands()
        .entity(weapon_card)
        .with_children(|card| {
            // Weapon name
            spawn_stat_row(card, "Name", &weapon.name);

            // Damage
            spawn_stat_row(card, "Damage", &format!("{:.1}", weapon.damage));

            // Range
            spawn_stat_row(card, "Range", &format!("{:.1}", weapon.range));

            // Attack speed
            spawn_stat_row(card, "Speed", &format!("{:.2}s", weapon.speed_secs));

            // Melee arc (only for melee weapons)
            if let Some(arc_degrees) = weapon.melee_arc {
                spawn_stat_row(card, "Arc", &format!("{:.0}°", arc_degrees));
            }
        });
}

fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(format!("{}:", label)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_SECONDARY),
            ));

            // Value
            row.spawn((
                Text::new(value),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(UiTheme::TEXT_PRIMARY),
            ));
        });
}

// ============================================================================
// Query Helpers
// ============================================================================

/// Builds HeroDisplayData from hero entity and its weapon children.
/// Call this from village_ui when building the heroes content.
pub fn build_hero_display_data(
    hero_entity: Entity,
    children_query: &Query<&Children>,
    weapon_query: &Query<
        (
            Option<&DisplayName>,
            &Damage,
            &AttackRange,
            &AttackSpeed,
            Option<&MeleeArc>,
        ),
        With<Weapon>,
    >,
    is_melee_query: &Query<(), With<MeleeWeapon>>,
) -> HeroDisplayData {
    // Placeholder hero name (heroes don't have names yet)
    let name = "Hero".to_string();

    // Find weapon child
    let weapon = children_query
        .get(hero_entity)
        .ok()
        .and_then(|children| {
            children.iter().find_map(|child| {
                weapon_query.get(child).ok().map(|(display_name, damage, range, speed, melee_arc)| {
                    let weapon_name = display_name
                        .map(|d| d.0.clone())
                        .unwrap_or_else(|| "Unknown Weapon".to_string());

                    // Get speed in seconds from timer duration
                    let speed_secs = speed.timer.duration().as_secs_f32();

                    // Only show arc for melee weapons
                    let arc_degrees = if is_melee_query.get(child).is_ok() {
                        melee_arc.map(|arc| arc.width.to_degrees())
                    } else {
                        None
                    };

                    WeaponDisplayData {
                        name: weapon_name,
                        damage: damage.0,
                        range: range.0,
                        speed_secs,
                        melee_arc: arc_degrees,
                    }
                })
            })
        });

    HeroDisplayData { name, weapon }
}
