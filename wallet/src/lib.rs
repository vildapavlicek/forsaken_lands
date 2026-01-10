use {
    bevy::prelude::*, enemy_components::ResourceRewards, hero_events::EnemyKilled,
    std::collections::HashMap,
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
            .add_observer(process_enemy_killed_rewards);
    }
}

fn process_enemy_killed_rewards(
    trigger: On<EnemyKilled>,
    mut wallet: ResMut<Wallet>,
    enemies: Query<&ResourceRewards>,
) {
    let event = trigger.event();
    if let Ok(rewards) = enemies.get(event.entity) {
        for reward in rewards.0.iter() {
            let current = wallet.resources.entry(reward.id.clone()).or_insert(0);
            *current += reward.value;
            trace!("Added {} {} to wallet", reward.value, reward.id);
        }
    }
}
