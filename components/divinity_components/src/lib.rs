use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
    std::cmp::Ordering,
};

pub struct DivinityComponentsPlugin;

impl Plugin for DivinityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Divinity>();

        app.register_type::<CurrentDivinity>();
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



    pub fn from_dashed_str(value: &str) -> Result<Self, String> {
        let Some((tier, level)) = value.split_once('-') else {
            return Err(format!("invalid value for divinity: '{value}'"));
        };

        let Ok(tier) = tier.parse() else {
            return Err(format!("invalid tier number: {tier}"));
        };
        let Ok(level) = level.parse() else {
            return Err(format!("invalid level number {level}"));
        };

        Ok(Divinity { tier, level })
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



/// Represents the current active Divinity level of a Portal.
#[derive(
    Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize,
)]
#[reflect(Component, Default)]
pub struct CurrentDivinity(pub Divinity);

impl Default for CurrentDivinity {
    fn default() -> Self {
        Self(Divinity::default())
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


}
