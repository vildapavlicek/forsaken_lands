use super::{BonusStat, BonusStats, DamageContext};

/// Core damage calculation pipeline.
/// Pure functions only.

/// Calculates the final damage based on context and global bonuses.
pub fn calculate(ctx: &DamageContext, bonus_stats: &BonusStats) -> f32 {
    let mut total_bonus = BonusStat::default();

    // We only care about the "damage" category
    if let Some(damage_bonuses) = bonus_stats.bonuses.get("damage") {
        // 1. Process Source Tags match against "damage:{tag}"
        for tag in &ctx.source_tags {
            if let Some(("damage", suffix)) = tag.split_once(':') {
                if let Some(bonus) = damage_bonuses.get(suffix) {
                    total_bonus = total_bonus + *bonus;
                }
            }
        }

        // 2. Process Target Tags match against "damage:{tag}"
        for tag in &ctx.target_tags {
            if let Some(bonus) = damage_bonuses.get(tag.as_str()) {
                total_bonus = total_bonus + *bonus;
            }
        }
    }

    // Calculation: (Base + Additive) * (1 + Percent) * Multiplicative
    let final_damage = (ctx.base_damage + total_bonus.additive)
        * (1.0 + total_bonus.percent)
        * total_bonus.multiplicative.max(1.0);

    final_damage.max(0.0)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{StatBonus, StatMode},
    };

    #[test]
    fn test_pipeline_basic() {
        let stats = BonusStats::default();
        let ctx = DamageContext::new(10.0, &[], &[]);
        assert_eq!(calculate(&ctx, &stats), 10.0);
    }

    #[test]
    fn test_pipeline_with_bonuses() {
        let mut stats = BonusStats::default();
        stats.add(
            "damage:melee",
            StatBonus {
                value: 0.5,
                mode: StatMode::Percent,
            },
        );

        let ctx = DamageContext::new(10.0, &["damage:melee".into()], &[]);
        assert_eq!(calculate(&ctx, &stats), 15.0);
    }

    #[test]
    fn test_pipeline_with_target_tags() {
        let mut stats = BonusStats::default();
        stats.add(
            "damage:goblin",
            StatBonus {
                value: 10.0,
                mode: StatMode::Additive,
            },
        );

        let ctx = DamageContext::new(10.0, &[], &["goblin".into()]);
        assert_eq!(calculate(&ctx, &stats), 20.0);
    }
}
