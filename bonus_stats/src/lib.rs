use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StatMode {
    #[default]
    Additive, // +10
    Percent,        // +10% (0.10)
    Multiplicative, // x2.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatBonus {
    pub value: f32,
    pub mode: StatMode,
}

/// Aggregated bonuses for a specific key (e.g., "damage:melee").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStat {
    pub additive: f32,
    pub percent: f32,        // Sum of percentages (e.g., 0.1 + 0.2 = 0.3)
    pub multiplicative: f32, // Product of multipliers (starts at 1.0)
}

impl Default for RawStat {
    fn default() -> Self {
        Self {
            additive: 0.0,
            percent: 0.0,
            multiplicative: 1.0,
        }
    }
}

impl RawStat {
    pub fn add(&mut self, bonus: &StatBonus) {
        match bonus.mode {
            StatMode::Additive => self.additive += bonus.value,
            StatMode::Percent => self.percent += bonus.value,
            StatMode::Multiplicative => self.multiplicative *= bonus.value,
        }
    }

    pub fn reset(&mut self) {
        self.additive = 0.0;
        self.percent = 0.0;
        self.multiplicative = 1.0;
    }
}

/// Component attached to entities to manage their **bonuses**.
#[derive(Component, Default)]
pub struct BonusStats {
    /// Stores aggregated bonuses for each key.
    /// Key example: "damage", "damage:melee".
    pub bonuses: HashMap<String, RawStat>,
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

    /// Computes the final value using hierarchical lookup.
    ///
    /// # Arguments
    /// * `base` - The base value provided by the caller (e.g. Weapon Damage).
    /// * `stat_id` - The primary stat key (e.g. "damage").
    /// * `tags` - Context tags (e.g. "melee", "bone_sword").
    ///
    /// # Logic
    /// 1. Collects bonuses for `stat_id` (e.g. "damage").
    /// 2. Collects bonuses for `stat_id:tag` for each tag (e.g. "damage:melee").
    /// 3. Aggregates all collected bonuses:
    ///    - Total Additive = Sum(All Additives)
    ///    - Total Percent = Sum(All Percents)
    ///    - Total Mult = Product(All Multipliers)
    /// 4. Result = (base + Total Additive) * (1.0 + Total Percent) * Total Mult
    pub fn compute(&self, base: f32, stat_id: &str, tags: &[&str]) -> f32 {
        let mut total_raw = RawStat::default();

        // 1. Primary Key
        if let Some(raw) = self.bonuses.get(stat_id) {
            total_raw.additive += raw.additive;
            total_raw.percent += raw.percent;
            total_raw.multiplicative *= raw.multiplicative;
        }

        // 2. Tag Keys (e.g., "damage:melee")
        for tag in tags {
            // Avoid allocating string for lookup if possible, but HashMap needs &str or String.
            // Constructing key: "{}:{}", stat_id, tag
            let key = format!("{}:{}", stat_id, tag);
            if let Some(raw) = self.bonuses.get(&key) {
                total_raw.additive += raw.additive;
                total_raw.percent += raw.percent;
                total_raw.multiplicative *= raw.multiplicative;
            }
        }

        (base + total_raw.additive) * (1.0 + total_raw.percent) * total_raw.multiplicative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_computation() {
        let mut stats = BonusStats::default();

        // Base: 10
        // Bonus: +5 (Additive)
        stats.add(
            "damage",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );

        // Compute
        let val = stats.compute(10.0, "damage", &[]);
        assert_eq!(val, 15.0);
    }

    #[test]
    fn test_complex_math() {
        let mut stats = BonusStats::default();

        // Base: 10
        // +5 Additive -> 15
        stats.add(
            "damage",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );

        // +20% Percent -> 1.0 + 0.2 = 1.2
        stats.add(
            "damage",
            StatBonus {
                value: 0.2,
                mode: StatMode::Percent,
            },
        );

        // x2 Multiplicative -> 2.0
        stats.add(
            "damage",
            StatBonus {
                value: 2.0,
                mode: StatMode::Multiplicative,
            },
        );

        // Math: (10 + 5) * 1.2 * 2.0 = 15 * 1.2 * 2.0 = 18 * 2 = 36
        let val = stats.compute(10.0, "damage", &[]);
        assert_eq!(val, 36.0);
    }

    #[test]
    fn test_hierarchical_tags() {
        let mut stats = BonusStats::default();

        // Base: 10
        // "damage": +5 Additive
        stats.add(
            "damage",
            StatBonus {
                value: 5.0,
                mode: StatMode::Additive,
            },
        );

        // "damage:melee": +10% Percent
        stats.add(
            "damage:melee",
            StatBonus {
                value: 0.1,
                mode: StatMode::Percent,
            },
        );

        // "damage:sword": x2 Multiplicative
        stats.add(
            "damage:sword",
            StatBonus {
                value: 2.0,
                mode: StatMode::Multiplicative,
            },
        );

        // Unrelated tag "damage:ranged"
        stats.add(
            "damage:ranged",
            StatBonus {
                value: 999.0,
                mode: StatMode::Additive,
            },
        );

        // Compute with melee, sword
        let val = stats.compute(10.0, "damage", &["melee", "sword"]);

        // (10 + 5) * (1.0 + 0.1) * 2.0 = 15 * 1.1 * 2.0 = 16.5 * 2 = 33
        assert_eq!(val, 33.0);
    }

    #[test]
    fn test_multiple_sources() {
        let mut stats = BonusStats::default();

        // Source A: +10%
        stats.add(
            "damage",
            StatBonus {
                value: 0.1,
                mode: StatMode::Percent,
            },
        );
        // Source B: +20%
        stats.add(
            "damage",
            StatBonus {
                value: 0.2,
                mode: StatMode::Percent,
            },
        );

        // Total Percent: 0.3 -> 1.3 mult
        let val = stats.compute(100.0, "damage", &[]);
        assert_eq!(val, 130.0);
    }
}
