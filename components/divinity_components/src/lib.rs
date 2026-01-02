use bevy::prelude::*;
use std::cmp::Ordering;

pub struct DivinityComponentsPlugin;

impl Plugin for DivinityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Divinity>();
        app.register_type::<DivinityStats>();
    }
}

pub const MAX_LEVEL: u32 = 99;

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct Divinity {
    pub tier: u32,
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

#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, Default)]
pub struct DivinityStats {
    pub current_xp: f32,
    pub required_xp: f32,
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
