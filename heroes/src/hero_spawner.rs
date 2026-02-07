use {
    bevy::prelude::*,
    hero_components::{EquippedWeaponId, Hero},
    unlocks_events::UnlockAchieved,
    village_components::Village,
};

pub fn hero_spawn_on_unlock(
    trigger: On<UnlockAchieved>,
    mut commands: Commands,
    hero_query: Query<&Name, With<Hero>>,
    village_query: Query<Entity, With<Village>>,
) {
    let event = trigger.event();

    debug!(%event.reward_id, "hero spawner reacting to event");

    let Some(hero_name) = event.reward_id.strip_prefix("hero:") else {
        return;
    };

    debug!("Hero unlock triggered: {}", hero_name);

    if hero_query.iter().any(|name| name.as_str() == hero_name) {
        debug!("Hero '{}' already exists, skipping spawn", hero_name);
        return;
    };

    let Ok(village) = village_query.single() else {
        error!("multiple or no village found!");
        return;
    };

    info!("Spawning hero: {}", hero_name);
    commands.spawn((
        Name::new(hero_name.to_owned()),
        Hero,
        EquippedWeaponId(Some("melee_rock".to_string())),
        ChildOf(village),
    ));
}
