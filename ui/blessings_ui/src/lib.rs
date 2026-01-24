use {
    bevy::prelude::*,
    blessings::{BlessingDefinition, Blessings, BuyBlessing},
    growth::GrowthStrategy,
    wallet::Wallet,
    widgets::{
        UiTheme, spawn_action_button, spawn_card_title, spawn_description_text,
        spawn_scrollable_container,
    },
};

pub struct BlessingsUiPlugin;

impl Plugin for BlessingsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_blessing_button, update_blessings_ui));
    }
}

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
}

pub fn spawn_blessings_content(parent: &mut ChildSpawnerCommands, data: Vec<BlessingDisplayData>) {
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
            spawn_scrollable_container(container, BlessingsItemsContainer);
        });

    parent.commands().queue(PopulateBlessingsCommand { data });
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
                        spawn_card_title(c, &format!("{} (Lvl {})", item.name, item.current_level));
                        spawn_description_text(c, &item.description);

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
    mut last_data: Local<Option<Vec<BlessingDisplayData>>>,
) {
    if ui_query.is_empty() {
        return;
    }

    let Ok(blessings) = blessings_query.single() else {
        return;
    };

    let mut data = Vec::new();
    let current_entropy = wallet.resources.get("entropy").copied().unwrap_or(0);

    for (_id, def) in assets.iter() {
        let id_str = def.id.clone();
        let current_level = blessings.unlocked.get(&id_str).copied().unwrap_or(0);
        let cost = def.cost.calculate(current_level) as u32;

        let can_afford = current_entropy >= cost;

        data.push(BlessingDisplayData {
            id: id_str,
            name: def.name.clone(),
            description: def.description.clone(),
            current_level,
            cost,
            can_afford,
        });
    }

    data.sort_by(|a, b| a.name.cmp(&b.name));

    if let Some(last) = last_data.as_ref() {
        if *last == data {
            return;
        }
    }
    *last_data = Some(data.clone());

    commands.queue(PopulateBlessingsCommand { data });
}
