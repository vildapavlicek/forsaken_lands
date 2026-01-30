//! Weapon asset definitions and spawning utilities.
//!
//! Weapons are defined as `.weapon.ron` asset files and spawned via code
//! to allow proper save/load of dynamic weapon state.

mod spawner;

pub use spawner::*;
use {
    bevy::{platform::collections::HashMap, prelude::*},
    bevy_common_assets::ron::RonAssetPlugin,
    serde::{Deserialize, Serialize},
};

pub struct WeaponAssetsPlugin;

impl Plugin for WeaponAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<WeaponDefinition>::new(&["weapon.ron"]))
            .init_resource::<WeaponMap>()
            .register_type::<WeaponType>();
    }
}

/// Weapon definition loaded from `.weapon.ron` asset files.
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDefinition {
    /// Unique identifier for the weapon (e.g., "bone_sword")
    pub id: String,
    /// Display name shown in UI
    pub display_name: String,
    /// Type of weapon (melee or ranged) with type-specific attributes
    pub weapon_type: WeaponType,
    /// Base damage dealt by this weapon
    pub damage: f32,
    /// Attack range in game units
    pub attack_range: f32,
    /// Time between attacks in milliseconds
    pub attack_speed_ms: u32,
    /// Tags associated with this weapon (e.g., "melee", "bone_sword")
    pub tags: Vec<String>,
    /// Bonus stats applied by this weapon
    #[serde(default)]
    pub bonuses: HashMap<String, bonus_stats::StatBonus>,
}

/// Type of weapon with type-specific attributes.
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WeaponType {
    Melee { arc_width: f32 },
    // TODO: Add projectile_speed: f32 field for ranged weapons
    Ranged,
}

/// Resource mapping weapon IDs to their asset handles.
/// Populated during asset loading phase.
#[derive(Resource, Default)]
pub struct WeaponMap {
    pub handles: HashMap<String, Handle<WeaponDefinition>>,
}
