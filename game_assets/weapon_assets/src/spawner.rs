//! Weapon spawning utilities.
//!
//! Provides functions to spawn weapon entities from WeaponDefinition assets.

use {
    crate::{WeaponDefinition, WeaponType},
    bevy::prelude::*,
    hero_components::{
        AttackRange, AttackSpeed, Damage, MeleeArc, MeleeWeapon, RangedWeapon, Weapon, WeaponId,
    },
    shared_components::DisplayName,
};

/// Spawns a weapon entity from a WeaponDefinition.
/// Returns the spawned entity ID.
pub fn spawn_weapon(commands: &mut Commands, def: &WeaponDefinition) -> Entity {
    match &def.weapon_type {
        WeaponType::Melee { arc_width } => commands
            .spawn((
                WeaponId(def.id.clone()),
                Weapon,
                MeleeWeapon,
                DisplayName(def.display_name.clone()),
                Damage(def.damage),
                AttackRange(def.attack_range),
                MeleeArc { width: *arc_width },
                AttackSpeed {
                    timer: Timer::from_seconds(def.attack_speed_ms as f32 / 1000.0, TimerMode::Repeating),
                },
            ))
            .id(),
        WeaponType::Ranged => commands
            .spawn((
                WeaponId(def.id.clone()),
                Weapon,
                RangedWeapon,
                DisplayName(def.display_name.clone()),
                Damage(def.damage),
                AttackRange(def.attack_range),
                AttackSpeed {
                    timer: Timer::from_seconds(def.attack_speed_ms as f32 / 1000.0, TimerMode::Repeating),
                },
            ))
            .id(),
    }
}

/// Spawns a weapon as a child of the given parent entity.
/// Returns the spawned entity ID.
pub fn spawn_weapon_as_child(
    commands: &mut Commands,
    def: &WeaponDefinition,
    parent: Entity,
) -> Entity {
    let weapon = spawn_weapon(commands, def);
    commands.entity(parent).add_child(weapon);
    weapon
}
