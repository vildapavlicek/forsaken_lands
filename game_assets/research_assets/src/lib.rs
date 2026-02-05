use {
    bevy::{platform::collections::HashMap, prelude::*},
    serde::Deserialize,
    unlocks_assets::UnlockDefinition,
};

// --- Asset Definition ---

#[derive(Asset, TypePath, Debug, Clone, Deserialize, serde::Serialize)]
pub struct ResearchDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: HashMap<String, u32>,
    pub time_required: f32,
    /// Maximum times this research can be completed. Default is 1 (one-time).
    #[serde(default = "default_max_repeats")]
    pub max_repeats: u32,

    /// Optional inline unlock definition for when this research becomes available
    #[serde(default)]
    pub unlock: Option<UnlockDefinition>,
}

fn default_max_repeats() -> u32 {
    1
}
