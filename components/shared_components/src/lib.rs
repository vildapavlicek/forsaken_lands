use bevy::prelude::*;

pub struct SharedComponentsPlugin;

impl Plugin for SharedComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackBonus>();
        app.register_type::<BonusValue>();
        app.register_type::<DisplayName>();
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

#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, Default)]
pub struct AttackBonus {
    pub all: BonusValue,
    pub melee: BonusValue,
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
