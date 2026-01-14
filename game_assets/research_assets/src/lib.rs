use {
    bevy::{platform::collections::HashMap, prelude::*},
    serde::Deserialize,
};

// --- Asset Definition ---

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct ResearchDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: HashMap<String, u32>,
    pub time_required: f32,
    /// Maximum times this research can be completed. Default is 1 (one-time).
    #[serde(default = "default_max_repeats")]
    pub max_repeats: u32,
}

fn default_max_repeats() -> u32 {
    1
}
