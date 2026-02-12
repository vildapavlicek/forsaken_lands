use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};


/// Defines how a [`StatBonus`] value interacts with the base statistic.
///
/// The bonus system aggregates values in three stages:
/// 1. **Additive**: Bonuses are summed (e.g., +10).
/// 2. **Percent**: Bonuses are summed and applied as a multiplier to the base (e.g., +10% and +20% = +30% -> x1.3).
/// 3. **Multiplicative**: Bonuses are summed and applied as a *final* multiplier to the result (e.g., x2 and x2 = x4).
///
/// The calculation formula is:
/// `(Base + AdditiveSum) * (1.0 + PercentSum) * max(1.0, MultiplicativeSum)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Reflect)]
pub enum StatMode {
    /// Adds directly to the base value.
    ///
    /// Use this for flat increases like "+10 Damage" or "+50 Health".
    #[default]
    Additive,
    /// Adds to a percentage accumulator.
    ///
    /// A value of `0.1` represents +10%. Multiple percent bonuses are additive with each other
    /// (e.g., +10% and +20% results in +30%, not +32%).
    /// This multiplier is applied after flat additives.
    Percent,
    /// Adds to a final multiplier accumulator.
    ///
    /// Use this for powerful scaling effects like "Double Damage" (value 2.0).
    /// Note that multiple multiplicative bonuses are **additive with each other**:
    /// two x1.5 bonuses result in x3.0 (1.5 + 1.5), not x2.25 (1.5 * 1.5).
    /// The final multiplier is clamped to a minimum of 1.0.
    Multiplicative,
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

    pub fn get_with_prefix(&self, category: &str, key: &str) -> Option<&BonusStat> {
        self.bonuses.get(category)?.get(key)
    }

    /// Calculates a final value for a specific stat category and sub-stat.
    ///
    /// # Arguments
    /// * `category` - The main stat category (e.g., "damage", "research").
    /// * `base_value` - The starting value.
    /// * `tags` - Tags to match for bonuses (e.g., "melee", "weapon:bone_sword").
    ///            It will look for bonuses at `category:{tag}`.
    pub fn calculate_stat(&self, category: &str, base_value: f32, tags: &[String]) -> f32 {
        let mut total_bonus = BonusStat::default();

        if let Some(category_bonuses) = self.bonuses.get(category) {
            for tag in tags {
                // We support both "category:tag" and just "tag" check if we split it
                // But the current convention for source tags seems to be "damage:melee"
                // and they expect to match against the suffix.
                let suffix = if let Some((cat, suf)) = tag.split_once(':') {
                    if cat == category {
                        suf
                    } else {
                        tag.as_str()
                    }
                } else {
                    tag.as_str()
                };

                if let Some(bonus) = category_bonuses.get(suffix) {
                    total_bonus = total_bonus + *bonus;
                }
            }
        }

        // Calculation: (Base + Additive) * (1 + Percent) * Multiplicative
        let final_value = (base_value + total_bonus.additive)
            * (1.0 + total_bonus.percent)
            * total_bonus.multiplicative.max(1.0);

        final_value.max(0.0)
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
    let mut tags = source_tags.to_vec();
    tags.extend_from_slice(target_tags);
    bonus_stats.calculate_stat("damage", base_damage, &tags)
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

    #[test]
    fn test_calculate_generic() {
        let mut stats = BonusStats::default();

        stats.add(
            "research:autopsy",
            StatBonus {
                value: -0.25, // 25% reduction
                mode: StatMode::Percent,
            },
        );

        let tags = vec!["research:autopsy".to_string()];
        // 100 * (1 - 0.25) = 75
        assert_eq!(stats.calculate_stat("research", 100.0, &tags), 75.0);

        // Without tag -> no bonus
        assert_eq!(stats.calculate_stat("research", 100.0, &[]), 100.0);
    }
}
