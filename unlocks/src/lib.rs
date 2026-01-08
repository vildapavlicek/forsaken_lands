mod assets;
mod components;
mod compiler;
mod events;
mod resources;
mod systems;

pub use assets::*;
pub use components::*;
pub use events::*;
pub use resources::*;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use systems::*;

pub struct UnlocksPlugin;

impl Plugin for UnlocksPlugin {
    fn build(&self, app: &mut App) {
        app
            // Asset loading
            .add_plugins(RonAssetPlugin::<UnlockDefinition>::new(&["unlock.ron"]))
            // Resources
            .init_resource::<TopicMap>()
            .init_resource::<UnlockState>()
            // Registration
            .register_type::<UnlockState>()
            // Systems for compilation and change detection
            .add_systems(
                Update,
                (
                    compile_pending_unlocks,
                    check_wallet_changes,
                ),
            )
            // Observers for gate logic and event interception
            .add_observer(propagate_logic_signal)
            .add_observer(handle_unlock_completion)
            .add_observer(on_enemy_killed_stat_update)
            .add_observer(on_research_completed);
    }
}
