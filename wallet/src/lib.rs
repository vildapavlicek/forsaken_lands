use {
    bevy::prelude::*,
    enemy_components::ResourceRewards,
    messages::EnemyKilled,
    std::collections::HashMap,
    system_schedule::GameSchedule,
};

#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Default)]
pub struct Wallet {
    pub resources: HashMap<String, u32>,
}

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wallet>()
            .init_resource::<Wallet>()
            .add_systems(
                Update,
                process_enemy_killed_rewards.in_set(GameSchedule::Effect),
            );
    }
}

fn process_enemy_killed_rewards(
    mut wallet: ResMut<Wallet>,
    mut enemy_killed_reader: MessageReader<EnemyKilled>,
    enemies: Query<&ResourceRewards>,
) {
    for event in enemy_killed_reader.read() {
        if let Ok(rewards) = enemies.get(event.entity) {
            for reward in rewards.0.iter() {
                let current = wallet.resources.entry(reward.id.clone()).or_insert(0);
                *current += reward.value;
                info!("Added {} {} to wallet", reward.value, reward.id);
            }
        }
    }
}
