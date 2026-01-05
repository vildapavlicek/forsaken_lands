//! Component definitions and data structures for the editor.
//!
//! This module provides editable representations of enemy components
//! that can be serialized to Bevy scene format.

use serde::{Deserialize, Serialize};

/// All available enemy components that can be added to a prefab.
#[derive(Clone, Debug)]
pub enum EnemyComponent {
    // Required components (auto-added)
    Enemy,
    MonsterId(String),
    DisplayName(String),
    Health { current: f32, max: f32 },
    MovementSpeed(f32),
    Lifetime { secs: u64, nanos: u32 },
    Transform { x: f32, y: f32, z: f32 },
    Sprite { r: f32, g: f32, b: f32, a: f32, width: f32, height: f32 },

    // Optional components
    ResourceRewards(Vec<Reward>),
    RewardCoefficient(f32),
    NeedsHydration,
}

impl EnemyComponent {
    /// Get the display name of this component type.
    pub fn display_name(&self) -> &'static str {
        match self {
            EnemyComponent::Enemy => "Enemy",
            EnemyComponent::MonsterId(_) => "Monster ID",
            EnemyComponent::DisplayName(_) => "Display Name",
            EnemyComponent::Health { .. } => "Health",
            EnemyComponent::MovementSpeed(_) => "Movement Speed",
            EnemyComponent::Lifetime { .. } => "Lifetime",
            EnemyComponent::Transform { .. } => "Transform",
            EnemyComponent::Sprite { .. } => "Sprite",
            EnemyComponent::ResourceRewards(_) => "Resource Rewards",
            EnemyComponent::RewardCoefficient(_) => "Reward Coefficient",
            EnemyComponent::NeedsHydration => "Needs Hydration",
        }
    }

    /// Check if this is an auto-added (required) component.
    pub fn is_required(&self) -> bool {
        matches!(
            self,
            EnemyComponent::Enemy
                | EnemyComponent::MonsterId(_)
                | EnemyComponent::DisplayName(_)
                | EnemyComponent::Health { .. }
                | EnemyComponent::MovementSpeed(_)
                | EnemyComponent::Lifetime { .. }
                | EnemyComponent::Transform { .. }
                | EnemyComponent::Sprite { .. }
        )
    }
}

/// A resource reward entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reward {
    pub id: String,
    pub value: u32,
}

impl Default for Reward {
    fn default() -> Self {
        Self {
            id: String::new(),
            value: 0,
        }
    }
}

/// List of optional components that can be added.
pub fn optional_components() -> Vec<(&'static str, EnemyComponent)> {
    vec![
        ("Resource Rewards", EnemyComponent::ResourceRewards(vec![])),
        ("Reward Coefficient", EnemyComponent::RewardCoefficient(1.0)),
        ("Needs Hydration", EnemyComponent::NeedsHydration),
    ]
}

/// Create default required components for a new prefab.
pub fn default_required_components() -> Vec<EnemyComponent> {
    vec![
        EnemyComponent::Enemy,
        EnemyComponent::MonsterId("new_enemy".to_string()),
        EnemyComponent::DisplayName("New Enemy".to_string()),
        EnemyComponent::Health { current: 1.0, max: 1.0 },
        EnemyComponent::MovementSpeed(100.0),
        EnemyComponent::Lifetime { secs: 5, nanos: 0 },
        EnemyComponent::Transform { x: 0.0, y: 290.0, z: 0.0 },
        EnemyComponent::Sprite {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
            width: 16.0,
            height: 16.0,
        },
    ]
}
