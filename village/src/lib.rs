use {
    bevy::prelude::*,
    divinity_components::Divinity,
    enemy_components::MonsterId,
    enemy_events::EnemyEscaped,
    hero_events::EnemyKilled,
    shared_components::DisplayName,
    unlocks_events::UnlockAchieved,
    village_components::{EncyclopediaEntry, EnemyEncyclopedia, Village},
    village_resources::{DivinityUnlockState, VillageResourcesPlugin},
};

pub mod equipment;

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VillageResourcesPlugin)
            .register_type::<Village>()
            .register_type::<EnemyEncyclopedia>()
            .register_type::<EncyclopediaEntry>();

        app.add_observer(update_encyclopedia);
        app.add_observer(update_encyclopedia_on_escape);

        app.add_observer(divinity_increase_unlock);
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
        warn!(entity = ?trigger.event().entity, "failed to update encyclopedia: enemy data not found");
        return;
    };

    let display_name = display_name
        .map(|d| d.0.clone())
        .unwrap_or_else(|| monster_id.0.clone());

    for mut encyclopedia in &mut village_query {
        encyclopedia.increment_kill_count(&monster_id.0, &display_name);
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

fn update_encyclopedia_on_escape(
    trigger: On<EnemyEscaped>,
    mut village_query: Query<&mut EnemyEncyclopedia, With<Village>>,
    monster_query: Query<(&MonsterId, Option<&DisplayName>)>,
    mut commands: Commands,
) {
    let Ok((monster_id, display_name)) = monster_query.get(trigger.event().entity) else {
        warn!(entity = ?trigger.event().entity, "failed to update encyclopedia (escape): enemy data not found");
        return;
    };

    let display_name = display_name
        .map(|d| d.0.clone())
        .unwrap_or_else(|| monster_id.0.clone());

    for mut encyclopedia in &mut village_query {
        encyclopedia.increment_escape_count(&monster_id.0, &display_name);
        let entry = encyclopedia.inner.get(&monster_id.0).unwrap();
        let escape_count = entry.escape_count;

        trace!(
            monster_id = %monster_id.0,
            escape_count = %escape_count,
            "updated encyclopedia (escape)",
        );

        // Notify unlock system about escape count change
        commands.trigger(unlocks_events::ValueChanged {
            topic: format!("escapes:{}", monster_id.0),
            value: escape_count as f32,
        });
    }
}

fn divinity_increase_unlock(
    event: On<UnlockAchieved>,
    mut cmd: Commands,
    mut divinity: Query<&mut Divinity, With<Village>>,
    mut claimed_state: ResMut<DivinityUnlockState>,
) {
    let event = event.event();

    let Some(divinity_event) = event.reward_id.strip_prefix("divinity:") else {
        return;
    };

    let Ok(new_divinity) = Divinity::from_dashed_str(divinity_event) else {
        error!(%divinity_event, "Divinity in invalid format");
        return;
    };

    // Check if this unlock has already granted its divinity level-up
    if claimed_state.claimed.contains(&event.unlock_id) {
        debug!(
            unlock_id = %event.unlock_id,
            "Divinity level-up already claimed, skipping"
        );
        return;
    }

    match divinity.single_mut() {
        Ok(mut divinity) => {
            *divinity = new_divinity;
            claimed_state.claimed.insert(event.unlock_id.clone());
            info!(
                unlock_id = %event.unlock_id,
                tier = divinity.tier,
                level = divinity.level,
                "Divinity level-up granted"
            );

            cmd.trigger(unlocks_events::ValueChanged {
                topic: "divinity".into(),
                value: (divinity.tier * 100 + divinity.level) as f32,
            });
        }
        Err(err) => error!(%err, "failed to query village's divinity"),
    }
}

pub fn clean_up_village(mut commands: Commands, villages: Query<Entity, With<Village>>) {
    debug!("Cleaning up village");
    for entity in villages.iter() {
        commands.entity(entity).despawn();
    }
}
