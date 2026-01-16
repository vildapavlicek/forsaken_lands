use bevy::prelude::*;
use equipment_events::{EquipWeaponRequest, UnequipWeaponRequest};
use hero_components::{Hero, Weapon};

pub fn handle_equip_weapon(
    trigger: On<EquipWeaponRequest>,
    mut commands: Commands,
    weapon_query: Query<(), With<Weapon>>,
    hero_children_query: Query<&Children, With<Hero>>,
    child_weapon_query: Query<Entity, With<Weapon>>,
) {
    let event = trigger.event();

    // Validate weapon exists
    if weapon_query.get(event.weapon).is_err() {
        warn!("Weapon entity {:?} not found", event.weapon);
        return;
    }

    // Unequip current weapon if hero has one
    if let Ok(children) = hero_children_query.get(event.hero) {
        for child in children.iter() {
            if child_weapon_query.get(child).is_ok() {
                // Remove parent relationship (weapon becomes unequipped)
                commands.entity(child).remove::<ChildOf>();
            }
        }
    }

    // Equip new weapon (add parent relationship)
    commands.entity(event.weapon).insert(ChildOf(event.hero));

    info!(
        "Equipped weapon {:?} to hero {:?}",
        event.weapon, event.hero
    );
}

pub fn handle_unequip_weapon(
    trigger: On<UnequipWeaponRequest>,
    mut commands: Commands,
    hero_children_query: Query<&Children, With<Hero>>,
    weapon_query: Query<Entity, With<Weapon>>,
) {
    let event = trigger.event();

    if let Ok(children) = hero_children_query.get(event.hero) {
        for child in children.iter() {
            if weapon_query.get(child).is_ok() {
                // Remove parent relationship (weapon becomes unequipped)
                commands.entity(child).remove::<ChildOf>();
                info!(
                    "Unequipped weapon {:?} from hero {:?}",
                    child, event.hero
                );
            }
        }
    }
}
