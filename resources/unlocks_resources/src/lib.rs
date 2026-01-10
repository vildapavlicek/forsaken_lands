use {
    bevy::{platform::collections::HashMap, prelude::*},
    unlocks_components::{TopicEntity, TopicSubscribers},
};

/// Maps topic keys to their corresponding Topic Entities.
#[derive(Resource, Default)]
pub struct TopicMap {
    pub topics: HashMap<String, Entity>,
}

impl TopicMap {
    /// Returns the Topic Entity for a key, spawning it if it doesn't exist.
    pub fn get_or_create(&mut self, commands: &mut Commands, key: &str) -> Entity {
        if let Some(&entity) = self.topics.get(key) {
            entity
        } else {
            let entity = commands
                .spawn((
                    TopicEntity {
                        key: key.to_string(),
                    },
                    TopicSubscribers::default(),
                ))
                .id();
            self.topics.insert(key.to_string(), entity);
            entity
        }
    }
}

/// Persistent state tracking what has been unlocked.
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct UnlockState {
    /// Set of unlock IDs that have been achieved.
    pub completed: Vec<String>,
}

impl UnlockState {
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.completed.contains(&id.to_string())
    }
}
