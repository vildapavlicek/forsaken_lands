use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Debug, Clone)]
pub struct EnemyStatBlock {
    pub health: f32,
    pub speed: f32,
    pub drops: Vec<String>,
}

#[derive(Resource, Default, Debug)]
pub struct EnemyDetailsCache {
    pub details: HashMap<String, EnemyStatBlock>,
}
