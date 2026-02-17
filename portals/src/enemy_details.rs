use {
    bevy::prelude::*,
    enemy_components::{Drops, Health, MonsterTags, MovementSpeed},
    enemy_resources::{EnemyDetailsCache, EnemyStatBlock},
    loading::GameAssets,
    unlocks::UnlockAchieved,
};

/// An Observer that listens for `UnlockAchieved` events to populate the `EnemyDetailsCache`.
///
/// This system bridges the gap between the Unlock System (logic) and the UI System (display).
/// When an enemy-related unlock occurs (identified by the `encyclopedia_data:` prefix),
/// it parses the enemy's prefab (DynamicScene) to extract static stats like health, speed, and drops,
/// caching them for efficient UI access without entity instantiation.
///
/// # Trigger
/// - `UnlockAchieved`: Triggered by the Unlock System when all conditions for an unlock are met.
///
/// # Outcome
/// - Populates `EnemyDetailsCache` with an `EnemyStatBlock` for the unlocked monster ID.
pub fn cache_details_on_unlock(
    trigger: On<UnlockAchieved>,
    game_assets: Res<GameAssets>,
    scenes: Res<Assets<DynamicScene>>,
    mut cache: ResMut<EnemyDetailsCache>,
) {
    let reward_id = &trigger.event().reward_id;
    if !reward_id.starts_with("encyclopedia_data:") {
        return;
    }

    let monster_id = reward_id.replace("encyclopedia_data:", "");
    debug!("Unlocking enemy details for: {}", monster_id);

    if let Some(prefab_handle) = game_assets.enemies.get(&monster_id) {
        if let Some(details) = extract_enemy_details(&scenes, prefab_handle) {
            cache.details.insert(monster_id, details);
        } else {
            error!("Failed to extract enemy details for: {}", monster_id);
        }
    } else {
        error!("Enemy prefab not found for monster_id: {}", monster_id);
    }
}

fn extract_enemy_details(
    scenes: &Assets<DynamicScene>,
    handle: &Handle<DynamicScene>,
) -> Option<EnemyStatBlock> {
    let scene = scenes.get(handle)?;

    let mut health_val = 0.0;
    let mut speed_val = 0.0;
    let mut drops_vec = Vec::new();
    let mut tags_vec = Vec::new();

    for entity in &scene.entities {
        for component in &entity.components {
            if let Some(h) = component.try_downcast_ref::<Health>() {
                health_val = h.max;
                continue;
            };

            if let Some(s) = component.try_downcast_ref::<MovementSpeed>() {
                speed_val = s.0;
                continue;
            };

            if let Some(d) = component.try_downcast_ref::<Drops>() {
                drops_vec = d.0.iter().map(|drop| drop.id.clone()).collect();
                continue;
            };

            if let Some(t) = component.try_downcast_ref::<MonsterTags>() {
                tags_vec = t.0.clone();
                continue;
            };
        }
    }

    Some(EnemyStatBlock {
        health: health_val,
        speed: speed_val,
        drops: drops_vec,
        tags: tags_vec,
    })
}
