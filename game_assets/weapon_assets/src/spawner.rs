//! Weapon spawning utilities.
//!
//! Provides functions to spawn weapon entities from WeaponDefinition assets.

use {
    crate::{WeaponDefinition, WeaponType},
    bevy::prelude::*,
    hero_components::{
        AttackRange, AttackSpeed, Damage, MeleeArc, MeleeWeapon, RangedWeapon, Weapon, WeaponId,
        WeaponTags,
    },
    shared_components::DisplayName,
};

/// Spawns a weapon entity from a WeaponDefinition.
/// Returns the spawned entity ID.
pub fn spawn_weapon(commands: &mut Commands, def: &WeaponDefinition) -> Entity {
    let mut entity = commands.spawn((
        WeaponId(def.id.clone()),
        Weapon,
        DisplayName(def.display_name.clone()),
        Damage(def.damage),
        AttackRange(def.attack_range),
        WeaponTags(def.tags.clone()),
        AttackSpeed {
            timer: Timer::from_seconds(def.attack_speed_ms as f32 / 1000.0, TimerMode::Once),
        },
    ));

    match &def.weapon_type {
        WeaponType::Melee { arc_width } => {
            entity.insert((MeleeWeapon, MeleeArc { width: *arc_width }));
        }
        WeaponType::Ranged => {
            entity.insert(RangedWeapon);
        }
    }

    entity.id()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_weapon_with_tags() {
        let mut app = App::new();
        let def = WeaponDefinition {
            id: "test_weapon".to_string(),
            display_name: "Test Weapon".to_string(),
            weapon_type: WeaponType::Melee { arc_width: 1.0 },
            damage: 10.0,
            attack_range: 2.0,
            attack_speed_ms: 1000,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let entity = spawn_weapon(&mut app.world_mut().commands(), &def);
        app.update();

        let tags = app.world().get::<WeaponTags>(entity).expect("WeaponTags component missing");
        assert_eq!(tags.0, vec!["tag1", "tag2"]);

        let id = app.world().get::<WeaponId>(entity).expect("WeaponId component missing");
        assert_eq!(id.0, "test_weapon");
    }
}
