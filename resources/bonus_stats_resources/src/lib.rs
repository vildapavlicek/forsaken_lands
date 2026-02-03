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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
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
            multiplicative: 0.0,
        }
    }
}

impl BonusStat {
    pub fn add(&mut self, bonus: &StatBonus) {
        match bonus.mode {
            StatMode::Additive => self.additive += bonus.value,
            StatMode::Percent => self.percent += bonus.value,
            StatMode::Multiplicative => self.multiplicative += bonus.value,
        }
    }

    pub fn remove(&mut self, bonus: &StatBonus) {
        match bonus.mode {
            StatMode::Additive => self.additive -= bonus.value,
            StatMode::Percent => self.percent -= bonus.value,
            StatMode::Multiplicative => {
                if bonus.value != 0.0 {
                    self.multiplicative -= bonus.value;
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

impl std::ops::Add for BonusStat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BonusStat {
            additive: self.additive + rhs.additive,
            percent: self.percent + rhs.percent,
            multiplicative: self.multiplicative + rhs.multiplicative,
        }
    }
}

/// Resource to manage global **bonuses**.
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct BonusStats {
    /// Stores aggregated bonuses for each key.
    /// Outer key: Category (e.g., "damage", "hp")
    /// Inner key: Sub-key (e.g., "melee", "fire", "" for exact matches)
    bonuses: HashMap<String, HashMap<String, BonusStat>>,
}

impl BonusStats {
    /// Clears all bonuses.
    pub fn clear(&mut self) {
        self.bonuses.clear();
    }

    /// Adds a bonus to a specific key.
    pub fn add(&mut self, key: &str, bonus: StatBonus) {
        let (category, subkey) = key.split_once(':').unwrap_or((key, ""));
        self.bonuses
            .entry(category.to_string())
            .or_default()
            .entry(subkey.to_string())
            .or_default()
            .add(&bonus);
    }

    /// Removes a bonus from a specific key.
    pub fn remove(&mut self, key: &str, bonus: StatBonus) {
        let (category, subkey) = key.split_once(':').unwrap_or((key, ""));
        if let Some(cat_map) = self.bonuses.get_mut(category) {
            if let Some(stat) = cat_map.get_mut(subkey) {
                stat.remove(&bonus);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&BonusStat> {
        let (category, subkey) = key.split_once(':').unwrap_or((key, ""));
        self.bonuses.get(category)?.get(subkey)
    }
}

/// Calculates the final damage considering base damage, source tags, target tags, and active bonuses.
///
/// # Arguments
/// * `base_damage` - The base damage of the weapon or source.
/// * `source_tags` - Tags associated with the damage source (e.g., "damage:melee", "bone_sword").
/// * `target_tags` - Tags associated with the target entity (e.g., "siled", "boss").
/// * `bonus_stats` - The global bonus stats resource.
///
/// # Returns
/// The calculated final damage.
pub fn calculate_damage(
    base_damage: f32,
    source_tags: &[String],
    target_tags: &[String],
    bonus_stats: &BonusStats,
) -> f32 {
    let mut total_bonus_stats = BonusStat::default();

    // We only care about the "damage" category
    if let Some(damage_bonuses) = bonus_stats.bonuses.get("damage") {
        // 1. Process Source Tags match against "damage:{tag}"
        // The source tags are expected to be full keys like "damage:melee" or just "melee"?
        // Looking at the original code, it filtered for "damage:..." prefix.
        for tag in source_tags {
            if let Some(("damage", suffix)) = tag.split_once(':') {
                if let Some(bonus) = damage_bonuses.get(suffix) {
                    total_bonus_stats = total_bonus_stats + *bonus;
                }
            }
        }

        // 2. Process Target Tags match against "damage:{tag}"
        // Target tags are "siled" which should match "damage:siled" rule.
        // In our structure, "damage:siled" is stored as category="damage", subkey="siled".
        // So we just look up the tag directly in the damage_bonuses map.
        for tag in target_tags {
            if let Some(bonus) = damage_bonuses.get(tag.as_str()) {
                total_bonus_stats = total_bonus_stats + *bonus;
            }
        }
    }

    // Calculation:
    // (Base + Additive) * (1 + Percent) * Multiplicative
    let final_damage = (base_damage + total_bonus_stats.additive)
        * (1.0 + total_bonus_stats.percent)
        * total_bonus_stats.multiplicative.max(1.0);

    // Ensure damage doesn't go below 0
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
        assert_eq!(raw.multiplicative, 0.0);
        assert_eq!(raw.multiplicative, 0.0);
    }


    #[test]
    fn test_calculate_damage() {
        let mut stats = BonusStats::default();

        // Base cases
        assert_eq!(calculate_damage(10.0, &[], &[], &stats), 10.0);

        // Global damage bonus
        // NOTE: Requires "damage" tag in source_tags to apply!
        // NOTE: Requires "damage" tag in source_tags to apply!
        stats.add(
            "damage:global",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );
        // Without tag -> no bonus
        assert_eq!(calculate_damage(10.0, &[], &[], &stats), 10.0);
        // With tag -> bonus applies
        assert_eq!(
            calculate_damage(10.0, &["damage:global".to_string()], &[], &stats),
            15.0
        );

        // Source specific bonus (tag)
        stats.add(
            "damage:melee",
            StatBonus {
                value: 0.5, // +50%
                mode: StatMode::Percent,
            },
        );
        let tags = vec!["damage:global".to_string(), "damage:melee".to_string()];
        // (10 + 5) * (1 + 0.5) = 15 * 1.5 = 22.5
        assert_eq!(calculate_damage(10.0, &tags, &[], &stats), 22.5);

        // Multiple source tags
        stats.add(
            "damage:fire",
            StatBonus {
                value: 2.0, // x2
                mode: StatMode::Multiplicative,
            },
        );
        let tags_fire = vec![
            "damage:global".to_string(),
            "damage:melee".to_string(),
            "damage:fire".to_string(),
        ];
        // ((10 + 5) * (1 + 0.5)) * 2 = 22.5 * 2 = 45.0
        assert_eq!(calculate_damage(10.0, &tags_fire, &[], &stats), 45.0);

        // Target specific bonus (NEW "damage:<tag>")
        // Target specific bonus (NEW "damage:<tag>")
        stats.add(
            "damage:siled",
            "damage:siled",
            StatBonus {
                value: 0.5, // +50%
                mode: StatMode::Percent,
            },
        );
        let target_tags = vec!["siled".to_string()];
        // Base damage 10, global +5 (if we include damage tag)
        // With "damage" tag: (10 + 5) * (1 + 0.5) = 22.5
        assert_eq!(
            calculate_damage(10.0, &["damage:global".to_string()], &target_tags, &stats),
            22.5
        );

        // Without global damage deduction but with target bonus?
        // Let's test just target bonus on clean stats.
        let mut clean_stats = BonusStats::default();
        clean_stats.add(
            "damage:siled",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );
        // 10 + 5 = 15
        assert_eq!(
            calculate_damage(10.0, &[], &target_tags, &clean_stats),
            15.0
        );

        // Combined source and target bonuses
        // Source: "melee" (+50%), Target: "siled" (+50%), Global: +5
        // (10 + 5) * (1 + 0.5 + 0.5) = 15 * 2.0 = 30.0
        assert_eq!(
            calculate_damage(
                10.0,
                &vec!["damage:global".to_string(), "damage:melee".to_string()],
                &target_tags,
                &stats
            ),
            30.0
        );

        // Name spaced tag test: race:goblin -> damage:race:goblin
        stats.add(
            "damage:race:goblin",
            StatBonus {
                value: 10.0,
                mode: StatMode::Additive,
            },
        );
        let goblin_tags = vec!["race:goblin".to_string()];
        // using just local clean calc for clarity
        // Base 10 + 10 = 20
        assert_eq!(calculate_damage(10.0, &[], &goblin_tags, &stats), 20.0);
    }

    #[test]
    fn calculate_damage_ignore_non_damage_tags() {
        let mut stats = BonusStats::default();

        stats.add(
            "damage:fire",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );

        stats.add(
            "fire",
            StatBonus {
                value: 5.0,
                mode: StatMode::Multiplicative,
            },
        );

        assert_eq!(
            10.0,
            calculate_damage(5.0, &["damage:fire".into()], &[], &stats)
        )
    }
}
