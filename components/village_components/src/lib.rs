use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Reflect, Default, Debug, Clone)]
pub struct EncyclopediaEntry {
    pub kill_count: u64,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct EnemyEncyclopedia {
    pub inner: HashMap<String, EncyclopediaEntry>,
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
#[require(EnemyEncyclopedia)]
pub struct Village;
