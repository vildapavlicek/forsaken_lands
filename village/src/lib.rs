use {
    bevy::prelude::*,
    enemy_components::MonsterId,
    hero_events::EnemyKilled,
    village_components::{EncyclopediaEntry, EnemyEncyclopedia, Village},
};

pub struct VillagePlugin;

impl Plugin for VillagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Village>();
        app.register_type::<EnemyEncyclopedia>();
        app.register_type::<EncyclopediaEntry>();

        app.add_observer(update_encyclopedia);
    }
}

fn update_encyclopedia(
    trigger: On<EnemyKilled>,
    mut village_query: Query<&mut EnemyEncyclopedia, With<Village>>,
    monster_query: Query<&MonsterId>,
) {
    let Ok(monster_id) = monster_query.get(trigger.event().entity) else {
        return;
    };

    for mut encyclopedia in &mut village_query {
        encyclopedia.increment_kill_count(monster_id.0.clone());
        info!(
            monster_id = %monster_id.0,
            kill_count = %encyclopedia.inner.get(&monster_id.0).unwrap().kill_count,
            "updated encyclopedia",
        );
    }
}
