use {
    bevy::prelude::*,
    blessings::{BlessingDefinition, BlessingLimit, BlessingState, Blessings, BuyBlessing},
    growth::GrowthStrategy,
    states::{GameState, VillageView},
    wallet::Wallet,
    widgets::{
        ContentContainer, UiTheme, spawn_action_button, spawn_card_title, spawn_description_text,
        spawn_menu_button, spawn_scrollable_container,
    },
};

pub struct BlessingsUiPlugin;

impl Plugin for BlessingsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(VillageView::Blessings), spawn_blessings_ui)
            .add_systems(
                Update,
                handle_back_button.run_if(in_state(GameState::Running)),
            )
            .add_systems(
                Update,
                (handle_blessing_button, update_blessings_ui)
                    .run_if(in_state(VillageView::Blessings)),
            );
    }
}

/// Back button to return to menu - we need this marker to match village_ui's behavior
#[derive(Component)]
struct VillageBackButton;

#[derive(Component)]
pub struct BlessingsUiRoot;

#[derive(Component)]
pub struct BlessingsItemsContainer;

#[derive(Component)]
pub struct BlessingButton {
    pub id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub struct BlessingDisplayData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub current_level: u32,
    pub cost: u32,
    pub can_afford: bool,
    pub is_locked: bool,
    pub limit: BlessingLimit, 
}

fn spawn_blessings_ui(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&Children>), With<ContentContainer>>,
) {
    let Some((container, children)) = query.iter_mut().next() else {
        return;
    };

    // Despawn existing children
    let to_despawn: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
    for child in to_despawn {
        commands.entity(child).despawn();
    }

    // Spawn back button and blessings content
    commands.entity(container).with_children(|parent| {
        // Back button
        spawn_menu_button(parent, "← Back", VillageBackButton, true);

        // Spawn blessings root and scroll container
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BlessingsUiRoot,
            ))
            .with_children(|container| {
                spawn_scrollable_container(container, BlessingsItemsContainer, |_| {});
            });
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

struct PopulateBlessingsCommand {
    data: Vec<BlessingDisplayData>,
}

impl Command for PopulateBlessingsCommand {
    fn apply(self, world: &mut World) {
        let mut container_query =
            world.query_filtered::<(Entity, Option<&Children>), With<BlessingsItemsContainer>>();

        let Some((container_entity, children)) = container_query.iter(world).next() else {
            return;
        };

        if let Some(children) = children {
            let children_vec: Vec<_> = children.to_vec();
            for child in children_vec {
                world.commands().entity(child).despawn();
            }
        }

        world
            .commands()
            .entity(container_entity)
            .with_children(|parent| {
                for item in self.data {
                    let card = widgets::spawn_item_card(parent, ());
                    parent.commands().entity(card).with_children(|c| {
                        let title = if item.is_locked {
                            format!("{} (LOCKED)", item.name)
                        } else {
                            format!("{} (Lvl {})", item.name, item.current_level)
                        };
                        spawn_card_title(c, &title);
                        spawn_description_text(c, &item.description);

                        if item.is_locked {
                            c.spawn((
                                Text::new("LOCKED"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(UiTheme::NOT_AFFORDABLE),
                            ));
                        } else {
                            c.spawn((
                                Text::new(format!("Cost: {} Entropy", item.cost)),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(if item.can_afford {
                                    UiTheme::AFFORDABLE
                                } else {
                                    UiTheme::NOT_AFFORDABLE
                                }),
                            ));

                            let is_maxed = match item.limit {
                                BlessingLimit::MaxLevel(max) => item.current_level >= max,
                                BlessingLimit::Unlimited => false,
                            };

                            if is_maxed {
                                c.spawn((
                                    Text::new("MAX LEVEL"),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(UiTheme::TEXT_DISABLED),
                                ));
                            } else {
                                spawn_action_button(
                                    c,
                                    "Upgrade",
                                    if item.can_afford {
                                        UiTheme::AFFORDABLE
                                    } else {
                                        UiTheme::BORDER_DISABLED
                                    },
                                    if item.can_afford {
                                        UiTheme::BORDER_SUCCESS
                                    } else {
                                        UiTheme::BORDER_DISABLED
                                    },
                                    BlessingButton { id: item.id },
                                );
                            }
                        }
                    });
                }
            });
    }
}

fn handle_blessing_button(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &BlessingButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            commands.trigger(BuyBlessing {
                blessing_id: btn.id.clone(),
            });
        }
    }
}

fn update_blessings_ui(
    mut commands: Commands,
    assets: Res<Assets<BlessingDefinition>>,
    wallet: Res<Wallet>,
    blessings_query: Query<&Blessings>,
    ui_query: Query<Entity, With<BlessingsUiRoot>>,
    container_query: Query<Option<&Children>, With<BlessingsItemsContainer>>,
    mut last_data: Local<Option<Vec<BlessingDisplayData>>>,
    blessing_state: Res<BlessingState>,
) {
    if ui_query.is_empty() {
        return;
    }

    let Ok(blessings) = blessings_query.single() else {
        return;
    };

    let mut data = Vec::new();
    let current_entropy = wallet.resources.get("entropy").copied().unwrap_or(0);

    for (id, def) in assets.iter() {
        let id_str = def.id.clone();
        let current_level = blessings.unlocked.get(&id_str).copied().unwrap_or(0);
        let cost = def.cost.calculate(current_level) as u32;
        let is_locked = !blessing_state.available.contains(&id_str);

        let can_afford = current_entropy >= cost && !is_locked;

        // Ensure we are using valid asset handle logic if we need to filter?
        // Currently iterating all assets seems fine as there are no complicated conditions yet.
        let _ = id;

        data.push(BlessingDisplayData {
            id: id_str,
            name: def.name.clone(),
            description: def.description.clone(),
            current_level,
            cost,
            can_afford,
            is_locked,
            limit: def.limit.clone(),
        });
    }

    data.sort_by(|a, b| a.name.cmp(&b.name));

    let container_empty = container_query
        .iter()
        .next()
        .map(|c| c.map(|children| children.is_empty()).unwrap_or(true))
        .unwrap_or(true);

    if !data.is_empty() && container_empty {
        // Force update if we have data but UI is empty
    } else if let Some(last) = last_data.as_ref() {
        if *last == data {
            return;
        }
    }
    *last_data = Some(data.clone());

    commands.queue(PopulateBlessingsCommand { data });
}
