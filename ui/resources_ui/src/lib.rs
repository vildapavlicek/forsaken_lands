use {bevy::prelude::*, states::GameState, wallet::Wallet, widgets::UiTheme};

pub struct ResourcesUiPlugin;

impl Plugin for ResourcesUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Running), setup_resources_ui)
            .add_systems(
                Update,
                update_resources_ui
                    .run_if(in_state(GameState::Running).and(resource_changed::<Wallet>)),
            );
    }
}

#[derive(Component)]
struct ResourceText;

fn setup_resources_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Resources: "),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TextColor(UiTheme::TEXT_PRIMARY),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        ResourceText,
    ));
}

fn update_resources_ui(wallet: Res<Wallet>, mut query: Query<&mut Text, With<ResourceText>>) {
    for mut text in query.iter_mut() {
        let mut resources_str = String::from("Resources: ");
        let mut sorted_resources: Vec<_> = wallet.resources.iter().collect();
        sorted_resources.sort_by_key(|(id, _)| *id);

        for (id, value) in sorted_resources {
            resources_str.push_str(&format!("{}: {}  ", id, value));
        }
        text.0 = resources_str;
    }
}
