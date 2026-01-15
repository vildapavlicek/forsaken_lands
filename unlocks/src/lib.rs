//! # Unlocks Framework
//!
//! A reactive, data-driven condition and unlock system using a Logic Circuit architecture.
//!
//! ## Overview
//!
//! This framework allows defining unlock conditions in RON asset files that get compiled
//! into an ECS-based logic graph at runtime. When game state changes (kills, resources,
//! research), the changes propagate through the graph until an unlock is achieved.
//!
//! ## Architecture
//!
//! The system uses a **Topic-Subscriber** pattern with **Logic Gates** for complex conditions:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         UNLOCK DEFINITION (RON)                         │
//! │  condition: And([Stat("goblin_kills", >= 2), Resource("bones", >= 10)]) │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼ compile_pending_unlocks
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         ECS LOGIC GRAPH                                 │
//! │                                                                         │
//! │   ┌──────────────┐                                                      │
//! │   │ UnlockRoot   │◄─────────────────────────────────────┐               │
//! │   │ (Entity 44)  │                                      │               │
//! │   └──────────────┘                                      │               │
//! │          ▲                                              │               │
//! │          │ LogicSignalEvent                             │               │
//! │          │                                              │               │
//! │   ┌──────────────┐                                      │               │
//! │   │ LogicGate    │ (AND, requires 2 signals)            │               │
//! │   │ (Entity 45)  │◄──────────────────────┐              │               │
//! │   └──────────────┘                       │              │               │
//! │          ▲                               │              │               │
//! │          │ LogicSignalEvent              │              │               │
//! │          │                               │              │               │
//! │   ┌──────────────┐               ┌──────────────┐       │               │
//! │   │ StatSensor   │               │ResourceSensor│       │               │
//! │   │ (Entity 46)  │               │ (Entity 47)  │       │               │
//! │   └──────────────┘               └──────────────┘       │               │
//! │          ▲                               ▲              │               │
//! │          │ subscribed to                 │              │               │
//! │          │                               │              │               │
//! │   ┌──────────────┐               ┌──────────────┐       │               │
//! │   │ TopicEntity  │               │ TopicEntity  │       │               │
//! │   │"stat:goblin_ │               │"resource:    │       │               │
//! │   │     kills"   │               │     bones"   │       │               │
//! │   └──────────────┘               └──────────────┘       │               │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Data Flow: Enemy Kill Example
//!
//! When a goblin is killed, here's the complete event chain:
//!
//! ```text
//! 1. GAME EVENT
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │ EnemyKilled { entity: goblin_entity }                           │
//!    │   triggered by: portals::despawn_dead_enemies                   │
//!    └─────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! 2. STAT CHANGE DETECTION
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │ on_enemy_killed_stat_update (observer)                          │
//!    │   - Reads MonsterId from killed entity ("goblin")               │
//!    │   - Reads kill count from EnemyEncyclopedia (e.g., 2)           │
//!    │   - Looks up TopicEntity for "stat:goblin_kills" in TopicMap    │
//!    │   - Triggers StatChangedEvent on that topic entity              │
//!    └─────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! 3. SENSOR UPDATE
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │ on_stat_changed (observer)                                      │
//!    │   - Receives StatChangedEvent { entity: topic, new_value: 2 }   │
//!    │   - Iterates TopicSubscribers to find subscribed sensors        │
//!    │   - For each StatSensor: evaluates condition (2 >= 2 = true)    │
//!    │   - If condition.is_met changed: triggers LogicSignalEvent      │
//!    │     targeting sensor's parent gate                              │
//!    └─────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! 4. LOGIC PROPAGATION
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │ propagate_logic_signal (observer)                               │
//!    │   - Receives LogicSignalEvent { entity: gate, is_high: true }   │
//!    │   - Updates gate.current_signals counter                        │
//!    │   - Evaluates gate condition (AND: all children met?)           │
//!    │   - If gate state changed: triggers LogicSignalEvent to parent  │
//!    │   - Repeats until reaching UnlockRoot                           │
//!    └─────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! 5. UNLOCK COMPLETION
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │ propagate_logic_signal (when target is UnlockRoot)              │
//!    │   - If is_high == true: triggers UnlockEvent                    │
//!    │                                                                 │
//!    │ handle_unlock_completion (observer)                             │
//!    │   - Adds unlock_id to UnlockState.completed                     │
//!    │   - Triggers UnlockCompletedEvent for dependent unlocks         │
//!    └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Components
//!
//! ### Entities
//! - **UnlockRoot**: Root entity representing an unlock. Receives final signal.
//! - **LogicGate**: AND/OR/NOT operator. Counts child signals, propagates when threshold met.
//! - **ConditionSensor**: Leaf node tracking a single condition (stat, resource, unlock).
//! - **TopicEntity**: Event channel entity. Sensors subscribe to topics for updates.
//!
//! ### Events (EntityEvent - target via `event.entity`)
//! - **LogicSignalEvent**: Propagates up the logic tree. `is_high` = condition met.
//! - **StatChangedEvent**: Notifies topic subscribers of stat value change.
//! - **ResourceChangedEvent**: Notifies topic subscribers of resource change.
//! - **UnlockCompletedEvent**: Notifies dependent unlocks when prerequisite completes.
//!
//! ### Resources
//! - **TopicMap**: Maps topic keys (e.g., "stat:goblin_kills") to TopicEntity entities.
//! - **UnlockState**: Tracks which unlocks have been completed.
//!
//! ## Asset Format
//!
//! Unlock definitions are RON files with `.unlock.ron` extension:
//!
//! ```ron
//! (
//!     id: "recipe_bone_sword",
//!     display_name: Some("Bone Sword Recipe"),
//!     reward_id: "crafting_recipe_bone_sword",
//!     condition: And([
//!         Stat(StatCheck(stat_id: "goblin_kills", value: 10.0, op: Ge)),
//!         Resource(ResourceCheck(resource_id: "bones", amount: 5)),
//!     ]),
//! )
//! ```
//!
//! **Important**: `stat_id` should NOT include the `stat:` prefix - it's added automatically.
//! For monster kills, use `"{monster_id}_kills"` format (e.g., `"goblin_kills"`).
//!
//! ## Debugging Tips
//!
//! Enable logging with `RUST_LOG=unlocks=debug` to see:
//! - Topic lookups and matches
//! - Condition evaluations with before/after state
//! - Signal propagation through the gate tree
//! - Unlock completion events

pub mod compiler;
mod systems;
#[cfg(test)]
mod tests;

use {bevy::prelude::*, systems::*};
pub use {unlocks_components::*, unlocks_events::*, unlocks_resources::*};

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
            // Systems for change detection
            .add_systems(Update, (check_wallet_changes, check_max_divinity_changes))
            // Observers for gate logic and event interception
            .add_observer(propagate_logic_signal)
            .add_observer(handle_unlock_completion)
            .add_observer(on_enemy_killed_stat_update)
            .add_observer(on_research_completed)
            // Observers for sensor updates
            .add_observer(on_stat_changed)
            .add_observer(on_resource_changed)
            .add_observer(on_unlock_topic_updated)
            .add_observer(cleanup_finished_unlock)
            .add_observer(on_max_divinity_changed);
    }
}
