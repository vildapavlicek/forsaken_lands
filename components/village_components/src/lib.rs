use {
    bevy::{picking::prelude::Pickable, prelude::*},
    std::collections::HashMap,
};

#[derive(Reflect, Default, Debug, Clone)]
pub struct EncyclopediaEntry {
    pub display_name: String,
    pub kill_count: u64,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct EnemyEncyclopedia {
    pub inner: HashMap<String, EncyclopediaEntry>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[require(EnemyEncyclopedia, Pickable)]
pub struct Village;

impl EnemyEncyclopedia {
    pub fn increment_kill_count(&mut self, enemy_id: String, display_name: String) {
        self.inner
            .entry(enemy_id)
            .and_modify(|e| e.kill_count += 1)
            .or_insert(EncyclopediaEntry {
                display_name,
                kill_count: 1,
            });
    }
}
