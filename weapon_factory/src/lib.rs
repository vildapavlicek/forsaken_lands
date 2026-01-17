use {
    bevy::prelude::*,
    village_components::{Village, WeaponInventory},
    weapon_assets::{spawn_weapon, spawn_weapon_as_child, WeaponDefinition, WeaponMap},
    weapon_factory_events::SpawnWeaponRequest,
};

pub struct WeaponFactoryPlugin;

impl Plugin for WeaponFactoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_spawn_weapon_request);
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
            info!("Spawned weapon '{}' as child of {:?}", event.weapon_id, parent);
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
