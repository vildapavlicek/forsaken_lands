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
#[reflect(Resource)]
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

/// Calculates the final damage considering base damage, weapon tags, and active bonuses.
///
/// # Arguments
/// * `base_damage` - The base damage of the weapon or source.
/// * `tags` - Tags associated with the damage source (e.g., "melee", "bone_sword").
/// * `bonus_stats` - The global bonus stats resource.
///
/// # Returns
/// The calculated final damage.
pub fn calculate_damage(base_damage: f32, tags: &[String], bonus_stats: &BonusStats) -> f32 {
    let mut total_additive = 0.0;
    let mut total_percent = 0.0;
    let mut total_multiplicative = 1.0;

    // Helper to accumulate bonuses for a specific key
    let mut accumulate_for_key = |key: &str| {
        if let Some(stat) = bonus_stats.get(key) {
            total_additive += stat.additive;
            total_percent += stat.percent;
            total_multiplicative *= stat.multiplicative;
        }
    };

    // 1. Base "damage" key
    accumulate_for_key("damage");

    // 2. Tag-specific keys: "damage:{tag}"
    for tag in tags {
        accumulate_for_key(&format!("damage:{}", tag));
    }

    // Calculation:
    // (Base + Additive) * (1 + Percent) * Multiplicative
    let final_damage = (base_damage + total_additive) * (1.0 + total_percent) * total_multiplicative;

    // Ensure damage doesn't go below 0 (unless we want healing attacks?)
    final_damage.max(0.0)
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
    #[test]
    fn test_calculate_damage() {
        let mut stats = BonusStats::default();

        // Base cases
        assert_eq!(calculate_damage(10.0, &[], &stats), 10.0);

        // Global damage bonus
        stats.add(
            "damage",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );
        assert_eq!(calculate_damage(10.0, &[], &stats), 15.0); // 10 + 5

        // Tag specific bonus
        stats.add(
            "damage:melee",
            StatBonus {
                value: 0.5, // +50%
                mode: StatMode::Percent,
            },
        );
        let tags = vec!["melee".to_string()];
        // (10 + 5) * (1 + 0.5) = 15 * 1.5 = 22.5
        assert_eq!(calculate_damage(10.0, &tags, &stats), 22.5);

        // Multiple tags
        stats.add(
            "damage:fire",
            StatBonus {
                value: 2.0, // x2
                mode: StatMode::Multiplicative,
            },
        );
        let tags_fire = vec!["melee".to_string(), "fire".to_string()];
        // ((10 + 5) * (1 + 0.5)) * 2 = 22.5 * 2 = 45.0
        assert_eq!(calculate_damage(10.0, &tags_fire, &stats), 45.0);
    }
}
