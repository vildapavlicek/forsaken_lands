//! # Unlocks Framework
//!
//! A reactive, data-driven condition and unlock system using a Logic Circuit architecture.
//!
//! ## Overview
//!
//! This framework allows defining unlock conditions in RON asset files that get compiled
//! into an ECS-based logic graph at runtime. When game state changes, the game triggers
//! generic events (`ValueChanged`, `StatusCompleted`) that propagate through the graph
//! until an unlock is achieved.
//!
//! ## Usage
//!
//! 1. Define unlock assets in `.unlock.ron` files
//! 2. Call `compile_unlocks()` or use the LoadingManager to compile them
//! 3. Trigger `ValueChanged` / `StatusCompleted` events when game state changes
//! 4. Listen for `UnlockAchieved` events to react to unlocks
//!
//! ## Asset Format
//!
//! ```ron
//! (
//!     id: "recipe_bone_sword",
//!     display_name: Some("Bone Sword Recipe"),
//!     reward_id: "crafting_recipe_bone_sword",
//!     condition: And([
//!         Value { topic: "kills:goblin", op: Ge, target: 10.0 },
//!         Value { topic: "resource:bones", op: Ge, target: 5.0 },
//!     ]),
//! )
//! ```
//!
//! ## Events
//!
//! - `ValueChanged { topic, value }` - Trigger when numeric values change
//! - `StatusCompleted { topic }` - Trigger when something is completed
//! - `UnlockAchieved { unlock_id, display_name, reward_id }` - Emitted when unlock conditions are met

pub mod compiler;
mod systems;

use {bevy::prelude::*, systems::*};
pub use {
    systems::clean_up_unlocks, unlocks_assets::*, unlocks_components::*, unlocks_events::*,
    unlocks_resources::*,
};

pub struct UnlocksPlugin;

impl Plugin for UnlocksPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<TopicMap>()
            .init_resource::<UnlockState>()
            // Registration
            .register_type::<UnlockState>()
            .register_type::<TopicSubscribers>()
            // Observers for gate logic
            .add_observer(propagate_logic_signal)
            .add_observer(handle_unlock_completion)
            .add_observer(cleanup_finished_unlock)
            // Observers for generic events
            .add_observer(on_value_changed)
            .add_observer(on_status_completed);
    }
}
