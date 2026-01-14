use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
    std::cmp::Ordering,
};

pub struct DivinityComponentsPlugin;

impl Plugin for DivinityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Divinity>();
        app.register_type::<DivinityStats>();
        app.register_type::<MaxUnlockedDivinity>();
    }
}

pub const MAX_LEVEL: u32 = 99;

/// Represents the power progression (Tier and Level) of a major entity (e.g., Portal, Village).
///
/// This component is the primary state machine for difficulty scaling and feature unlocking.
/// It is used by:
/// - `PortalsPlugin`: To gate enemy spawns based on `SpawnCondition`.
/// - `VillagePlugin`: To track village growth and unlock recipes/buildings.
///
/// Use `DivinityStats` to track XP progress towards the next level.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[reflect(Component, Default)]
pub struct Divinity {
    /// The major power bracket (1-based). Increasing this resets `level` to 1.
    pub tier: u32,
    /// The minor power increment (1-99). Reaching `MAX_LEVEL` allows a Tier up.
    pub level: u32,
}

impl Divinity {
    pub fn new(tier: u32, level: u32) -> Self {
        Self { tier, level }
    }

    /// Increments the level. If level reaches MAX_LEVEL + 1, it resets to 1 and increments tier.
    /// Returns true if tier increased.
    pub fn level_up(&mut self) -> bool {
        if self.level >= MAX_LEVEL {
            self.tier += 1;
            self.level = 1;
            true
        } else {
            self.level += 1;
            false
        }
    }
}

impl PartialOrd for Divinity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Divinity {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tier.cmp(&other.tier) {
            Ordering::Equal => self.level.cmp(&other.level),
            other => other,
        }
    }
}

impl Default for Divinity {
    fn default() -> Self {
        Self { tier: 1, level: 1 }
    }
}

/// Tracks the experience progress for a `Divinity` entity.
///
/// This component acts as a buffer for raw experience points. When `current_xp` exceeds
/// `required_xp`, it triggers a level-up in the `Divinity` component.
///
/// It is queried by:
/// - `PortalsPlugin` and `VillagePlugin`: To accumulate XP and handle level-ups.
/// - UI Systems: To display the progress bar (current / required).
#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, Default)]
pub struct DivinityStats {
    /// The accumulated raw experience points.
    pub current_xp: f32,
    /// The raw experience threshold required to advance to the next `Divinity` level.
    pub required_xp: f32,
}

/// Tracks the highest unlocked Divinity level for this Portal.
/// This allows the player to potentially lower the current Divinity level
/// while knowing what the maximum achievable level they've reached is.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
#[reflect(Component, Default)]
pub struct MaxUnlockedDivinity(pub Divinity);

impl Default for MaxUnlockedDivinity {
    fn default() -> Self {
        Self(Divinity::default())
    }
}

impl DivinityStats {
    /// Calculate required XP for a given Divinity level
    pub fn required_xp_for(divinity: &Divinity) -> f32 {
        // Base formula: 100 * tier * level
        100.0 * divinity.tier as f32 * divinity.level as f32
    }

    /// Add XP and return true if a level up occurred
    pub fn add_xp(&mut self, amount: f32, divinity: &mut Divinity) -> bool {
        self.current_xp += amount;
        if self.current_xp >= self.required_xp {
            self.current_xp -= self.required_xp;
            divinity.level_up();
            self.required_xp = Self::required_xp_for(divinity);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divinity_ordering() {
        let low = Divinity::new(1, 1);
        let mid = Divinity::new(1, 99);
        let high = Divinity::new(2, 1);
        let same_mid = Divinity::new(1, 99);

        assert!(low < mid);
        assert!(mid < high);
        assert!(low < high);
        assert!(mid == same_mid);
        assert!(high > mid);
    }

    #[test]
    fn test_level_up_simple() {
        let mut div = Divinity::new(1, 1);
        let tier_up = div.level_up();
        assert!(!tier_up);
        assert_eq!(div.tier, 1);
        assert_eq!(div.level, 2);
    }

    #[test]
    fn test_level_up_tier() {
        let mut div = Divinity::new(1, 99);
        let tier_up = div.level_up();
        assert!(tier_up);
        assert_eq!(div.tier, 2);
        assert_eq!(div.level, 1);
    }

    #[test]
    fn test_level_up_tier_high() {
        let mut div = Divinity::new(10, 99);
        let tier_up = div.level_up();
        assert!(tier_up);
        assert_eq!(div.tier, 11);
        assert_eq!(div.level, 1);
    }
}
