use {
    bevy::prelude::*,
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    enemy_components::MonsterId,
    hero_events::EnemyKilled,
    shared_components::DisplayName,
    village_components::{EncyclopediaEntry, EnemyEncyclopedia, Village},
};

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>();
        app.register_type::<EnemyEncyclopedia>();
        app.register_type::<EncyclopediaEntry>();

        app.add_observer(update_encyclopedia);
        app.add_observer(handle_divinity_increase);
    }
}

fn update_encyclopedia(
    trigger: On<EnemyKilled>,
    mut village_query: Query<&mut EnemyEncyclopedia, With<Village>>,
    monster_query: Query<(&MonsterId, Option<&DisplayName>)>,
) {
    let Ok((monster_id, display_name)) = monster_query.get(trigger.event().entity) else {
        return;
    };

    let display_name = display_name
        .map(|d| d.0.clone())
        .unwrap_or_else(|| monster_id.0.clone());

    for mut encyclopedia in &mut village_query {
        encyclopedia.increment_kill_count(monster_id.0.clone(), display_name.clone());
        trace!(
            monster_id = %monster_id.0,
            kill_count = %encyclopedia.inner.get(&monster_id.0).unwrap().kill_count,
            "updated encyclopedia",
        );
    }
}

fn handle_divinity_increase(
    trigger: On<IncreaseDivinity>,
    mut query: Query<(&mut Divinity, &mut DivinityStats), With<Village>>,
) {
    let event = trigger.event();
    if let Ok((mut divinity, mut stats)) = query.get_mut(event.entity) {
        info!(
            xp_added = event.xp_amount,
            "increased Village's divinity XP"
        );
        if stats.add_xp(event.xp_amount, &mut divinity) {
            info!(
                tier = divinity.tier,
                level = divinity.level,
                "Village leveled up"
            );
        }
    }
}
