use {
    bevy::prelude::*,
    crafting_events::WeaponCrafted,
    village_components::{Village, WeaponInventory},
};

pub fn handle_weapon_crafted(
    trigger: On<WeaponCrafted>,
    mut village_query: Query<&mut WeaponInventory, With<Village>>,
) {
    let event = trigger.event();

    if let Ok(mut inventory) = village_query.single_mut() {
        inventory.weapons.push(event.weapon_id.clone());
        info!("Added crafted weapon '{}' to inventory", event.weapon_id);
    } else {
        warn!(
            "Could not find Village to add crafted weapon '{}'",
            event.weapon_id
        );
    }
}
