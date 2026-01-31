use crate::{events::*, BonusStats, StatBonus};
use bevy::prelude::*;

pub struct BonusStatsPlugin;

impl Plugin for BonusStatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BonusStats>()
            .register_type::<BonusStats>()
            .add_observer(on_add_stat_bonus)
            .add_observer(on_remove_stat_bonus)
            .add_observer(on_increase_stat_bonus)
            .add_observer(on_decrease_stat_bonus);
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
