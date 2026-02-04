use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    growth::GrowthStrategy,
    serde::Deserialize,
    std::collections::{HashMap, HashSet},
    unlocks_events::UnlockAchieved,
    wallet::Wallet,
};

pub struct BlessingsPlugin;

impl Plugin for BlessingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<BlessingDefinition>::new(&["blessing.ron"]));
        app.register_type::<Blessings>();
        app.init_resource::<BlessingState>();
        app.add_observer(purchase_blessing);
        app.add_observer(handle_unlock_achieved);
    }
}

/// Event to trigger buying a blessing
#[derive(Debug, Clone, Event)]
pub struct BuyBlessing {
    pub blessing_id: String,
}

fn purchase_blessing(
    trigger: On<BuyBlessing>,
    mut commands: Commands,
    mut blessings_query: Query<&mut Blessings>,
    mut wallet: ResMut<Wallet>,
    blessing_state: Res<BlessingState>,
    blessing_definitions: Res<Assets<BlessingDefinition>>,
) {
    let event = trigger.event();

    // Check if blessing is unlocked/available
    if !blessing_state.available.contains(&event.blessing_id) {
        warn!("Blessing {} is locked or not available", event.blessing_id);
        return;
    }

    if let Ok(mut blessings) = blessings_query.single_mut() {
        if let Some((_, def)) = blessing_definitions
            .iter()
            .find(|(_, d)| d.id == event.blessing_id)
        {
            let current_level = *blessings.unlocked.get(&event.blessing_id).unwrap_or(&0);
            let cost = def.cost.calculate(current_level);

            // Check affordability
            let entropy_key = "entropy".to_string();
            let current_entropy = *wallet.resources.get(&entropy_key).unwrap_or(&0);

            if current_entropy >= cost as u32 {
                // Deduct cost
                if let Some(val) = wallet.resources.get_mut(&entropy_key) {
                    *val -= cost as u32;
                }

                // Increment level
                let new_level = current_level + 1;
                blessings
                    .unlocked
                    .insert(event.blessing_id.clone(), new_level);

                info!(
                    "Purchased blessing {}. New Level: {}",
                    event.blessing_id, new_level
                );

                // Trigger generic unlock event for downstream systems
                commands.trigger(UnlockAchieved {
                    unlock_id: format!("blessing:{}:{}", event.blessing_id, new_level),
                    display_name: Some(def.name.clone()),
                    reward_id: def.reward_id.clone(),
                });
            } else {
                warn!(
                    "Not enough Entropy. Cost: {}, Current: {}",
                    cost, current_entropy
                );
            }
        }
    }
}

/// Observes UnlockAchieved to detect when a blessing becomes available
fn handle_unlock_achieved(trigger: On<UnlockAchieved>, mut blessing_state: ResMut<BlessingState>) {
    let event = trigger.event();

    // Check if this unlock rewards a blessing availability
    // Format: "blessing_available:{blessing_id}"
    if let Some(blessing_id) = event.reward_id.strip_prefix("blessing_available:") {
        if !blessing_state.available.contains(blessing_id) {
            info!("Blessing unlocked: {}", blessing_id);
            blessing_state.available.insert(blessing_id.to_string());
        }
    }
}

/// Asset definition for a Blessing.
#[derive(Debug, Clone, Deserialize, TypePath, Asset)]
pub struct BlessingDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The reward ID triggers when this blessing is purchased/upgraded
    pub reward_id: String,
    /// Cost calculation strategy
    pub cost: growth::Growth,
}

/// Component attached to "The Maw" to track unlocked blessings.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Blessings {
    /// Map of Blessing ID to current level
    pub unlocked: HashMap<String, u32>,
}

/// Resource to map Blessing IDs to Asset Handles and track availability
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlessingState {
    pub blessings: HashMap<String, Handle<BlessingDefinition>>,
    /// Set of blessing IDs that are currently available to buy
    pub available: HashSet<String>,
}
