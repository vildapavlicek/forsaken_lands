use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Reflect)]
pub enum StatMode {
    #[default]
    Additive, // +10
    Percent,        // +10% (0.10)
    Multiplicative, // x2.0
}

pub mod events;
pub mod plugin;

pub use events::*;
pub use plugin::BonusStatsPlugin;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct StatBonus {
    pub value: f32,
    pub mode: StatMode,
}

/// Aggregated bonuses for a specific key (e.g., "damage:melee").
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BonusStat {
    pub additive: f32,
    pub percent: f32,        // Sum of percentages (e.g., 0.1 + 0.2 = 0.3)
    pub multiplicative: f32, // Product of multipliers (starts at 1.0)
}

impl Default for BonusStat {
    fn default() -> Self {
        Self {
            additive: 0.0,
            percent: 0.0,
            multiplicative: 1.0,
        }
    }
}

impl BonusStat {
    pub fn add(&mut self, bonus: &StatBonus) {
        match bonus.mode {
            StatMode::Additive => self.additive += bonus.value,
            StatMode::Percent => self.percent += bonus.value,
            StatMode::Multiplicative => self.multiplicative *= bonus.value,
        }
    }

    pub fn remove(&mut self, bonus: &StatBonus) {
        match bonus.mode {
            StatMode::Additive => self.additive -= bonus.value,
            StatMode::Percent => self.percent -= bonus.value,
            StatMode::Multiplicative => {
                if bonus.value != 0.0 {
                    self.multiplicative /= bonus.value;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.additive = 0.0;
        self.percent = 0.0;
        self.multiplicative = 1.0;
    }
}

/// Resource to manage global **bonuses**.
#[derive(Resource, Default, Debug, Reflect)]
pub struct BonusStats {
    /// Stores aggregated bonuses for each key.
    /// Key example: "damage", "damage:melee".
    bonuses: HashMap<String, BonusStat>,
}

impl BonusStats {
    /// Clears all bonuses.
    pub fn clear(&mut self) {
        self.bonuses.clear();
    }

    /// Adds a bonus to a specific key.
    pub fn add(&mut self, key: &str, bonus: StatBonus) {
        self.bonuses.entry(key.to_string()).or_default().add(&bonus);
    }

    /// Removes a bonus from a specific key.
    pub fn remove(&mut self, key: &str, bonus: StatBonus) {
        if let Some(stat) = self.bonuses.get_mut(key) {
            stat.remove(&bonus);
        }
    }

    pub fn get(&self, key: &str) -> Option<&BonusStat> {
        self.bonuses.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove() {
        let mut stats = BonusStats::default();

        let bonus = StatBonus {
            value: 10.0,
            mode: StatMode::Additive,
        };

        // Add
        stats.add("damage", bonus.clone());
        let raw = stats.get("damage").unwrap();
        assert_eq!(raw.additive, 10.0);

        // Remove
        stats.remove("damage", bonus);
        let raw = stats.get("damage").unwrap();
        assert_eq!(raw.additive, 0.0);
    }

    #[test]
    fn test_accumulation() {
        let mut stats = BonusStats::default();

        // Additive
        stats.add(
            "hp",
            StatBonus {
                value: 10.0,
                mode: StatMode::Additive,
            },
        );
        stats.add(
            "hp",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );

        // Percent
        stats.add(
            "hp",
            StatBonus {
                value: 0.1,
                mode: StatMode::Percent,
            },
        );

        let raw = stats.get("hp").unwrap();
        assert_eq!(raw.additive, 15.0);
        assert_eq!(raw.percent, 0.1);
        assert_eq!(raw.multiplicative, 1.0);
    }
}
