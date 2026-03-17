use bevy::prelude::*;

/// Represents a request to instantiate a weapon entity from an asset definition.
///
/// This **Observer** event (triggered via `commands.trigger`) decouples the intent
/// to create a weapon (e.g., from crafting or save-file reconstruction) from the
/// complex instantiation logic and inventory management handled by the Weapon Factory.
///
/// # Observers
/// - `weapon_factory::handle_spawn_weapon_request`: Receives the event, loads the
///   `WeaponDefinition` asset, spawns the correct hierarchy and components, and
///   optionally attaches it to a parent and/or registers it in the persistent inventory.
///
/// # Usage
/// - **Crafting**: Triggered by `weapon_factory::on_crafting_completed` when a weapon
///   recipe is finished, setting `add_to_inventory` to `true`.
/// - **Reconstruction**: Triggered by the save/load system during session startup to
///   rebuild equipped and stored weapons, setting `add_to_inventory` to `false` (as the
///   inventory is already being restored from save data).
#[derive(Event, Clone, Debug)]
pub struct SpawnWeaponRequest {
    pub weapon_id: String,
    /// If provided, the newly spawned weapon will be attached as a child to this
    /// target entity (e.g., the `Hero`), establishing its transform hierarchy.
    pub parent: Option<Entity>,
    /// Determines if the newly created weapon should be registered in the singleton
    /// `WeaponInventory`. Must be `false` during save-state reconstruction to prevent
    /// duplicate entries.
    pub add_to_inventory: bool,
}
