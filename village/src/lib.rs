use {
    bevy::prelude::*,
    divinity_components::{Divinity, DivinityStats},
    divinity_events::IncreaseDivinity,
    enemy_components::MonsterId,
    hero_events::EnemyKilled,
    shared_components::DisplayName,
    village_components::{EncyclopediaEntry, EnemyEncyclopedia, Village},
};

pub mod equipment;

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>();
        app.register_type::<EnemyEncyclopedia>();
        app.register_type::<EncyclopediaEntry>();

        app.add_observer(update_encyclopedia);
        app.add_observer(handle_divinity_increase);
        app.add_observer(equipment::handle_equip_weapon);
        app.add_observer(equipment::handle_unequip_weapon);
        app.add_observer(equipment::handle_unequip_weapon);
        app.add_systems(OnExit(states::GameState::Running), clean_up_village);
    }
}

fn update_encyclopedia(
    trigger: On<EnemyKilled>,
    mut village_query: Query<&mut EnemyEncyclopedia, With<Village>>,
    monster_query: Query<(&MonsterId, Option<&DisplayName>)>,
    mut commands: Commands,
) {
    let Ok((monster_id, display_name)) = monster_query.get(trigger.event().entity) else {
        return;
    };

    let display_name = display_name
        .map(|d| d.0.clone())
        .unwrap_or_else(|| monster_id.0.clone());

    for mut encyclopedia in &mut village_query {
        encyclopedia.increment_kill_count(monster_id.0.clone(), display_name.clone());
        let kill_count = encyclopedia.inner.get(&monster_id.0).unwrap().kill_count;
        trace!(
            monster_id = %monster_id.0,
            kill_count = %kill_count,
            "updated encyclopedia",
        );

        // Notify unlock system about kill count change
        commands.trigger(unlocks_events::ValueChanged {
            topic: format!("kills:{}", monster_id.0),
            value: kill_count as f32,
        });
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

pub fn clean_up_village(mut commands: Commands, villages: Query<Entity, With<Village>>) {
    debug!("Cleaning up village");
    for entity in villages.iter() {
        commands.entity(entity).despawn();
    }
}
