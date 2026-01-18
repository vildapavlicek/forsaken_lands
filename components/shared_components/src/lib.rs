use bevy::prelude::*;

pub struct SharedComponentsPlugin;

impl Plugin for SharedComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackBonus>();
        app.register_type::<BonusValue>();
        app.register_type::<DisplayName>();
        app.register_type::<HitIndicator>();
    }
}

/// Marker for entities that should be included in save files.
/// Add `#[require(IncludeInSave)]` to root components like Village, Hero, Portal, etc.
#[derive(Component, Default)]
pub struct IncludeInSave;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component, Default)]
pub struct HitIndicator {
    /// Timer for the duration of each blink state (white vs original).
    pub timer: Timer,
    /// The color currently *not* on the sprite (swapped out).
    pub saved_color: Color,
    /// Number of times left to swap back.
    pub blink_count: u32,
}

impl HitIndicator {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            saved_color: bevy::color::palettes::basic::WHITE.into(),
            blink_count: 4,
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Deref, DerefMut)]
#[reflect(Component, Default)]
pub struct DisplayName(pub String);

impl From<&str> for DisplayName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for DisplayName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Default)]
pub struct BonusValue {
    pub flat: f32,
    pub percentage: f32, // e.g. 0.1 for 10%
}

/// Aggregates damage modifiers for an entity's attacks, separating source types.
///
/// This component is the primary data source for damage calculation systems to apply
/// buffs, gear stats, or passive abilities to outgoing damage.
///
/// Systems should query this component on the *attacker* entity and apply bonuses
/// by summing the `all` value with the specific attack type value (`melee` or `ranged`).
///
/// Example formula:
/// `total_flat = base + bonus.all.flat + bonus.specific.flat`
/// `total_mult = 1.0 + bonus.all.percentage + bonus.specific.percentage`
/// `final_damage = total_flat * total_mult`
#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, Default)]
pub struct AttackBonus {
    /// Bonuses applied to *all* forms of attack damage.
    pub all: BonusValue,
    /// Bonuses applied specifically to melee attacks.
    pub melee: BonusValue,
    /// Bonuses applied specifically to ranged/projectile attacks.
    pub ranged: BonusValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_deref() {
        let name = DisplayName("Wooden Bow".to_string());
        assert_eq!(*name, "Wooden Bow");
    }

    #[test]
    fn test_display_name_from_str() {
        let name: DisplayName = "Wooden Bow".into();
        assert_eq!(name.0, "Wooden Bow");
    }
}
