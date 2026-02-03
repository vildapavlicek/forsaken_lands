use {bevy::prelude::*, bonus_stats_resources::StatMode};

#[derive(Event, Debug, Clone)]
pub struct AddStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}

#[derive(Event, Debug, Clone)]
pub struct RemoveStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}

#[derive(Event, Debug, Clone)]
pub struct IncreaseStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}

#[derive(Event, Debug, Clone)]
pub struct DecreaseStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}
