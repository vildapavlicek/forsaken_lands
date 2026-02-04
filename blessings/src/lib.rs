use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    serde::Deserialize,
    std::collections::{HashMap, HashSet},
    unlocks_events::UnlockAchieved,
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
    blessing_definitions: Res<Assets<BlessingDefinition>>,
) {
    let event = trigger.event();

    if let Ok(mut blessings) = blessings_query.single_mut() {
        if let Some((_, def)) = blessing_definitions
            .iter()
            .find(|(_, d)| d.id == event.blessing_id)
        {
            let current_level = *blessings.unlocked.get(&event.blessing_id).unwrap_or(&0);

            // Check limits
            match def.limit {
                BlessingLimit::MaxLevel(max) if current_level >= max => {
                    info!(
                        "Blessing {} is already at max level {}",
                        event.blessing_id, max
                    );
                    return;
                }
                _ => {}
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
                unlock_id: format!("blessing:{}", event.blessing_id),
                display_name: Some(def.name.clone()),
                reward_id: def.reward_id.clone(),
            });
        }
    }
}

/// Observes UnlockAchieved to detect when a blessing becomes available
fn handle_unlock_achieved(trigger: On<UnlockAchieved>, mut blessing_state: ResMut<BlessingState>) {
    let event = trigger.event();

    // Check if this unlock rewards a blessing availability
    // Format: "blessing:{blessing_id}"
    if let Some(blessing_id) = event.reward_id.strip_prefix("blessing:") {
        if !blessing_state.available.contains(blessing_id) {
            info!("Blessing unlocked: {}", blessing_id);
            blessing_state.available.insert(blessing_id.to_string());
        }
    }
}

#[derive(Debug, Clone, Deserialize, Reflect, PartialEq)]
pub enum BlessingLimit {
    Unlimited,
    MaxLevel(u32),
}

impl Default for BlessingLimit {
    fn default() -> Self {
        Self::Unlimited
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
    #[serde(default)]
    pub limit: BlessingLimit,
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
