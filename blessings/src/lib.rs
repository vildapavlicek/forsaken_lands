use bevy::prelude::*;
use std::collections::HashMap;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};
use growth::GrowthStrategy;
use wallet::Wallet;

pub struct BlessingsPlugin;

impl Plugin for BlessingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<BlessingDefinition>::new(&["blessing.ron"]));
        app.register_type::<Blessings>();
        app.init_resource::<BlessingState>();
        app.add_observer(purchase_blessing);
    }
}

/// Event to trigger buying a blessing
#[derive(Debug, Clone, Event)]
pub struct BuyBlessing {
    pub blessing_id: String,
}

/// Event when a blessing is successfully purchased
#[derive(Debug, Clone, Event)]
pub struct BlessingPurchased {
    pub blessing_id: String,
    pub new_level: u32,
    pub buyer: Entity,
}

fn purchase_blessing(
    trigger: On<BuyBlessing>,
    mut commands: Commands,
    mut blessings_query: Query<(Entity, &mut Blessings)>,
    mut wallet: ResMut<Wallet>,
    blessing_state: Res<BlessingState>,
    blessing_definitions: Res<Assets<BlessingDefinition>>,
) {
    let event = trigger.event();
    if let Ok((entity, mut blessings)) = blessings_query.single_mut() {
        if let Some(handle) = blessing_state.blessings.get(&event.blessing_id) {
            if let Some(def) = blessing_definitions.get(handle) {
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
                     blessings.unlocked.insert(event.blessing_id.clone(), new_level);
                     
                     info!("Purchased blessing {}. New Level: {}", event.blessing_id, new_level);
                     
                     commands.trigger(BlessingPurchased {
                         blessing_id: event.blessing_id.clone(),
                         new_level,
                         buyer: entity,
                     });
                } else {
                    warn!("Not enough Entropy. Cost: {}, Current: {}", cost, current_entropy);
                }
            }
        }
    }
}

/// Asset definition for a Blessing.
#[derive(Debug, Clone, Deserialize, TypePath, Asset)]
pub struct BlessingDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The specific effect this blessing applies
    pub effect: BlessingEffect,
    /// Cost calculation strategy
    pub cost: growth::Growth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
pub enum BlessingEffect {
    DecreaseSpawnTimer,
    IncreaseMonsterLifetime,
}

/// Component attached to "The Maw" to track unlocked blessings.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Blessings {
    /// Map of Blessing ID to current level
    pub unlocked: HashMap<String, u32>,
}

/// Resource to map Blessing IDs to Asset Handles for easy lookup
#[derive(Resource, Default)]
pub struct BlessingState {
    pub blessings: HashMap<String, Handle<BlessingDefinition>>,
}
