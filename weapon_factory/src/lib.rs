use {
    bevy::prelude::*,
    village_components::{Village, WeaponInventory},
    weapon_assets::{WeaponDefinition, WeaponMap, spawn_weapon, spawn_weapon_as_child},
    weapon_factory_events::SpawnWeaponRequest,
};

pub struct WeaponFactoryPlugin;

impl Plugin for WeaponFactoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_spawn_weapon_request)
            .add_observer(on_crafting_completed);
    }
}

fn handle_spawn_weapon_request(
    trigger: On<SpawnWeaponRequest>,
    mut commands: Commands,
    weapon_map: Res<WeaponMap>,
    weapon_assets: Res<Assets<WeaponDefinition>>,
    mut inventory_query: Query<&mut WeaponInventory, With<Village>>,
) {
    let event = trigger.event();

    let Some(handle) = weapon_map.handles.get(&event.weapon_id) else {
        warn!("Weapon '{}' not found in WeaponMap", event.weapon_id);
        return;
    };

    let Some(def) = weapon_assets.get(handle) else {
        warn!("Weapon definition not loaded for '{}'", event.weapon_id);
        return;
    };

    // Spawn with or without parent
    match event.parent {
        Some(parent) => {
            spawn_weapon_as_child(&mut commands, def, parent);
            info!(
                "Spawned weapon '{}' as child of {:?}",
                event.weapon_id, parent
            );
        }
        None => {
            spawn_weapon(&mut commands, def);
            info!("Spawned weapon '{}'", event.weapon_id);
        }
    };

    // Add to inventory if requested
    if event.add_to_inventory {
        if let Ok(mut inventory) = inventory_query.single_mut() {
            inventory.weapons.push(event.weapon_id.clone());
            info!("Added '{}' to WeaponInventory", event.weapon_id);
        } else {
            warn!("No Village found to add weapon to inventory");
        }
    }
}

/// Observer for StatusCompleted events.
/// Checks if the completed topic is a craft:{recipe_id} and if that recipe corresponds to a weapon.
/// If so, it spawns the weapon and adds it to the village inventory.
fn on_crafting_completed(
    trigger: On<unlocks_events::StatusCompleted>,
    mut commands: Commands,
    weapon_map: Res<WeaponMap>,
    weapon_assets: Res<Assets<WeaponDefinition>>,
    mut inventory_query: Query<&mut WeaponInventory, With<Village>>,
) {
    let event = trigger.event();

    if let Some(recipe_id) = event
        .topic
        .strip_prefix(unlocks_events::CRAFTING_TOPIC_PREFIX)
    {
        // Ensure this recipe ID corresponds to a registered weapon
        let Some(handle) = weapon_map.handles.get(recipe_id) else {
            // Not a weapon, ignore
            return;
        };

        let Some(def) = weapon_assets.get(handle) else {
            // Weapon asset not loaded yet?
            warn!("Weapon definition not loaded for '{}'", recipe_id);
            return;
        };

        // Spawn weapon
        spawn_weapon(&mut commands, def);
        info!("Spawned weapon '{}' from crafting", recipe_id);

        // Add to inventory
        if let Ok(mut inventory) = inventory_query.single_mut() {
            inventory.weapons.push(recipe_id.to_string());
            info!("Added '{}' to WeaponInventory (crafted)", recipe_id);
        } else {
            warn!("No Village found to add crafted weapon to inventory");
        }
    }
}
