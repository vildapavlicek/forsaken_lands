use {
    bevy::prelude::*,
    bonus_stats_resources::StatBonus,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    unlocks_assets::UnlockDefinition,
};

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct StatBonusDefinition {
    /// The trigger topic this bonus listens for (e.g., "research:steel_sword", "quest:intro")
    pub id: String,
    /// Map of stat keys to list of bonuses (e.g. "damage" -> [+10])
    pub bonuses: HashMap<String, Vec<StatBonus>>,
    /// Optional inline unlock definition for when this bonus should be granted
    #[serde(default)]
    pub unlock: Option<UnlockDefinition>,
}
