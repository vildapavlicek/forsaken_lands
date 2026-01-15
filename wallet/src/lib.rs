use {
    bevy::prelude::*,
    enemy_components::ResourceRewards,
    hero_events::EnemyKilled,
    std::collections::{HashMap, HashSet},
    unlocks_events::UnlockAchieved,
};

/// Central storage for all collected player resources (the game's economy state).
///
/// This **Resource** acts as the single source of truth for resource quantities.
/// It is:
/// - **Updated by:** `process_enemy_killed_rewards` (Observer) when enemies die.
/// - **Queried by:** UI systems (display), Crafting/Research systems (affordability checks).
#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Default)]
pub struct Wallet {
    /// Maps resource IDs (e.g., "wood", "iron") to their current quantity.
    pub resources: HashMap<String, u32>,
    /// Set of resource IDs that have been unlocked and are available for use.
    pub unlocked_resources: HashSet<String>,
}

/// Stores drop rate modifiers for resources.
///
/// Values represent multipliers: 1.0 = base rate, 1.2 = +20%, etc.
/// This is separate from `Wallet` because rates are configurable for balancing
/// and should not be persisted with save data.
#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Default)]
pub struct ResourceRates {
    /// Maps resource IDs to their rate multipliers.
    /// Missing entries default to 1.0 (no modifier).
    pub rates: HashMap<String, f32>,
}

impl ResourceRates {
    /// Gets the rate multiplier for a resource. Returns 1.0 if not set.
    pub fn get_rate(&self, resource_id: &str) -> f32 {
        *self.rates.get(resource_id).unwrap_or(&1.0)
    }

    /// Sets the rate multiplier for a resource.
    pub fn set_rate(&mut self, resource_id: impl Into<String>, rate: f32) {
        self.rates.insert(resource_id.into(), rate);
    }

    /// Adds a bonus to the current rate (additive).
    /// E.g., `add_bonus("bones", 0.2)` increases bones rate by 20%.
    pub fn add_bonus(&mut self, resource_id: &str, bonus: f32) {
        let current = self.get_rate(resource_id);
        self.rates.insert(resource_id.to_string(), current + bonus);
    }

    /// Removes a bonus from the current rate (additive).
    /// E.g., `remove_bonus("bones", 0.2)` decreases bones rate by 20%.
    pub fn remove_bonus(&mut self, resource_id: &str, bonus: f32) {
        let current = self.get_rate(resource_id);
        self.rates.insert(resource_id.to_string(), current - bonus);
    }

    /// Resets the rate for a resource back to the default (1.0).
    pub fn reset_rate(&mut self, resource_id: &str) {
        self.rates.remove(resource_id);
    }
}

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wallet>()
            .register_type::<ResourceRates>()
            .init_resource::<Wallet>()
            .init_resource::<ResourceRates>()
            .add_observer(process_enemy_killed_rewards)
            .add_observer(on_resource_unlock_achieved);
    }
}

fn process_enemy_killed_rewards(
    trigger: On<EnemyKilled>,
    mut wallet: ResMut<Wallet>,
    rates: Res<ResourceRates>,
    enemies: Query<&ResourceRewards>,
) {
    let event = trigger.event();
    if let Ok(rewards) = enemies.get(event.entity) {
        for reward in rewards.0.iter() {
            let rate = rates.get_rate(&reward.id);
            let modified_value = (reward.value as f32 * rate).round() as u32;
            let current = wallet.resources.entry(reward.id.clone()).or_insert(0);
            *current += modified_value;
            trace!(
                "Added {} {} to wallet (base: {}, rate: {})",
                modified_value,
                reward.id,
                reward.value,
                rate
            );
        }
    }
}

/// Observer for UnlockAchieved events with `resource_` prefix.
/// Adds the resource ID to the wallet's unlocked_resources set.
fn on_resource_unlock_achieved(trigger: On<UnlockAchieved>, mut wallet: ResMut<Wallet>) {
    let event = trigger.event();
    const PREFIX: &str = "resource_";

    if event.reward_id.starts_with(PREFIX) {
        let resource_id = &event.reward_id[PREFIX.len()..];
        wallet.unlocked_resources.insert(resource_id.to_string());
        info!("Resource '{}' is now unlocked", resource_id);
    }
}
