use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bonus_stats_assets::StatBonusDefinition;
use bonus_stats_events::*;
use bonus_stats_resources::{BonusStats, StatBonus};
use unlocks_events::StatusCompleted;

pub struct BonusStatsPlugin;

impl Plugin for BonusStatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BonusStats>()
            .register_type::<BonusStats>()
            .init_resource::<BonusTriggerMap>()
            .add_plugins(RonAssetPlugin::<StatBonusDefinition>::new(&[
                "stats.ron",
            ]))
            .add_observer(on_add_stat_bonus)
            .add_observer(on_remove_stat_bonus)
            .add_observer(on_increase_stat_bonus)
            .add_observer(on_decrease_stat_bonus)
            // Integration Support
            .add_observer(on_status_completed)
            .add_systems(
                Update,
                update_bonus_trigger_map.run_if(resource_changed::<Assets<StatBonusDefinition>>),
            )
            .add_systems(OnEnter(states::GameState::Loading), clear_bonus_stats);
    }
}

fn clear_bonus_stats(mut stats: ResMut<BonusStats>) {
    stats.clear();
    debug!("Cleared BonusStats resource");
}

/// Lookup map to quickly find bonuses for a given trigger (topic)
#[derive(Resource, Default)]
struct BonusTriggerMap {
    /// Mapping from Trigger ID (e.g. "research:steel_sword") to map of key -> bonuses
    triggers: std::collections::HashMap<String, std::collections::HashMap<String, Vec<StatBonus>>>,
}

/// Updates the trigger map when definition assets are loaded/modified\
/// Rebuilds the map from scratch to ensure consistency.
fn update_bonus_trigger_map(
    assets: Res<Assets<StatBonusDefinition>>,
    mut map: ResMut<BonusTriggerMap>,
) {
    if assets.is_empty() {
        return;
    }
    
    // We ideally want to incremental update, but rebuilding is safer/simpler without events.
    // If performance becomes an issue, we can revisit AssetEvents.
    map.triggers.clear();
    
    for (_, def) in assets.iter() {
        map.triggers.insert(def.id.clone(), def.bonuses.clone());
    }
    debug!("Rebuilt BonusTriggerMap with {} entries", map.triggers.len());
}

/// Listens for StatusCompleted events (from Research, Quests, etc.)
/// and applies corresponding bonuses.
fn on_status_completed(trigger: On<StatusCompleted>, mut stats: ResMut<BonusStats>, map: Res<BonusTriggerMap>) {
    let event = trigger.event();
    if let Some(bonuses_map) = map.triggers.get(&event.topic) {
        info!("Applying bonuses for completed topic: {}", event.topic);
        for (stat_key, bonus_list) in bonuses_map {
            for bonus in bonus_list {
                stats.add(stat_key, bonus.clone());
            }
        }
    }
}




fn on_add_stat_bonus(trigger: On<AddStatBonus>, mut stats: ResMut<BonusStats>) {
    let event = trigger.event();
    stats.add(
        &event.key,
        StatBonus {
            value: event.value,
            mode: event.mode,
        },
    );
    debug!(
        "Added stat bonus: {} {:?} ({})",
        event.key, event.value, event.mode as u8
    );
}

fn on_remove_stat_bonus(trigger: On<RemoveStatBonus>, mut stats: ResMut<BonusStats>) {
    let event = trigger.event();
    stats.remove(
        &event.key,
        StatBonus {
            value: event.value,
            mode: event.mode,
        },
    );
    debug!(
        "Removed stat bonus: {} {:?} ({})",
        event.key, event.value, event.mode as u8
    );
}

fn on_increase_stat_bonus(trigger: On<IncreaseStatBonus>, mut stats: ResMut<BonusStats>) {
    let event = trigger.event();
    stats.add(
        &event.key,
        StatBonus {
            value: event.value,
            mode: event.mode,
        },
    );
    debug!(
        "Increased stat: {} by {:?} ({})",
        event.key, event.value, event.mode as u8
    );
}

fn on_decrease_stat_bonus(trigger: On<DecreaseStatBonus>, mut stats: ResMut<BonusStats>) {
    let event = trigger.event();
    stats.remove(
        &event.key,
        StatBonus {
            value: event.value,
            mode: event.mode,
        },
    );
    debug!(
        "Decreased stat: {} by {:?} ({})",
        event.key, event.value, event.mode as u8
    );
}
