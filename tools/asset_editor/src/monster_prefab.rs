//! Monster prefab components and scene generation.
//!
//! This module provides editable representations of enemy components
//! that can be serialized to Bevy scene format.

use serde::{Deserialize, Serialize};

/// Defines which distance range section an enemy belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyRange {
    #[default]
    CloseRange,
    MediumRange,
    LongRange,
}

impl EnemyRange {
    pub fn all() -> &'static [EnemyRange] {
        &[
            EnemyRange::CloseRange,
            EnemyRange::MediumRange,
            EnemyRange::LongRange,
        ]
    }
}

/// All available enemy components that can be added to a prefab.
#[derive(Clone, Debug)]
pub enum EnemyComponent {
    // Required components (auto-added)
    Enemy,
    MonsterId(String),
    EnemyRange(EnemyRange),
    DisplayName(String),
    Health {
        current: f32,
        max: f32,
    },
    MovementSpeed(f32),
    Lifetime {
        secs: u64,
        nanos: u32,
    },
    Transform {
        x: f32,
        y: f32,
        z: f32,
    },
    Sprite {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        width: f32,
        height: f32,
    },

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
            EnemyComponent::EnemyRange(_) => "Enemy Range",
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
                | EnemyComponent::EnemyRange(_)
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
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Reward {
    pub id: String,
    pub value: u32,
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
        EnemyComponent::EnemyRange(EnemyRange::CloseRange),
        EnemyComponent::DisplayName("New Enemy".to_string()),
        EnemyComponent::Health {
            current: 1.0,
            max: 1.0,
        },
        EnemyComponent::MovementSpeed(170.0),
        EnemyComponent::Lifetime { secs: 5, nanos: 0 },
        EnemyComponent::Transform {
            x: 0.0,
            y: 290.0,
            z: 0.0,
        },
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

/// Build the scene RON string from the current components.
pub fn build_scene_ron(components: &[EnemyComponent]) -> String {
    let mut component_entries = Vec::new();

    for component in components {
        if let Some(entry) = component_to_ron(component) {
            component_entries.push(entry);
        }
    }

    // Build the full scene format
    format!(
        r#"(
  resources: {{}},
  entities: {{
    1: (
      components: {{
{}
      }},
    ),
  }},
)
"#,
        component_entries
            .iter()
            .map(|s| format!("        {},", s))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Convert a single component to its RON representation.
fn component_to_ron(component: &EnemyComponent) -> Option<String> {
    match component {
        EnemyComponent::Enemy => Some(r#""enemy_components::Enemy": ()"#.to_string()),
        EnemyComponent::MonsterId(id) => Some(format!(
            r#""enemy_components::MonsterId": ("{}")"#,
            escape_ron_string(id)
        )),
        EnemyComponent::EnemyRange(range) => {
            Some(format!(r#""enemy_components::EnemyRange": {:?}"#, range))
        }
        EnemyComponent::DisplayName(name) => Some(format!(
            r#""shared_components::DisplayName": ("{}")"#,
            escape_ron_string(name)
        )),
        EnemyComponent::Health { current, max } => Some(format!(
            r#""enemy_components::Health": (current: {}, max: {})"#,
            format_f32(*current),
            format_f32(*max)
        )),
        EnemyComponent::MovementSpeed(speed) => Some(format!(
            r#""enemy_components::MovementSpeed": ({})"#,
            format_f32(*speed)
        )),
        EnemyComponent::Lifetime { secs, nanos } => Some(format!(
            r#""enemy_components::Lifetime": ((stopwatch: (elapsed: (secs: 0, nanos: 0), is_paused: false), duration: (secs: {}, nanos: {}), mode: Once, finished: false, times_finished_this_tick: 0))"#,
            secs, nanos
        )),
        EnemyComponent::Transform { x, y, z } => Some(format!(
            r#""bevy_transform::components::transform::Transform": (translation: ({}, {}, {}))"#,
            format_f32(*x),
            format_f32(*y),
            format_f32(*z)
        )),
        EnemyComponent::Sprite {
            r,
            g,
            b,
            a,
            width,
            height,
        } => Some(format!(
            r#""bevy_sprite::sprite::Sprite": (color: Srgba(Srgba(red: {}, green: {}, blue: {}, alpha: {})), custom_size: Some(({}, {})))"#,
            format_f32(*r),
            format_f32(*g),
            format_f32(*b),
            format_f32(*a),
            format_f32(*width),
            format_f32(*height)
        )),
        EnemyComponent::ResourceRewards(rewards) => {
            if rewards.is_empty() {
                Some(r#""enemy_components::ResourceRewards": ([])"#.to_string())
            } else {
                let rewards_str = rewards
                    .iter()
                    .map(|r| {
                        format!(
                            r#"(id: "{}", value: {})"#,
                            escape_ron_string(&r.id),
                            r.value
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!(
                    r#""enemy_components::ResourceRewards": ([{}])"#,
                    rewards_str
                ))
            }
        }
        EnemyComponent::RewardCoefficient(coeff) => Some(format!(
            r#""enemy_components::RewardCoefficient": ({})"#,
            format_f32(*coeff)
        )),
        EnemyComponent::NeedsHydration => {
            Some(r#""enemy_components::NeedsHydration": ()"#.to_string())
        }
    }
}

/// Format a f32 to ensure it always has a decimal point (Bevy/RON requirement).
fn format_f32(v: f32) -> String {
    let s = v.to_string();
    if s.contains('.') {
        s
    } else {
        format!("{}.0", s)
    }
}

/// Escape special characters in RON strings.
fn escape_ron_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse components from a RON prefab file content.
/// Returns a vector of EnemyComponent if parsing is successful.
pub fn parse_components_from_ron(content: &str) -> Option<Vec<EnemyComponent>> {
    use regex::Regex;

    let mut components = Vec::new();

    // Enemy marker
    if content.contains("\"enemy_components::Enemy\"") {
        components.push(EnemyComponent::Enemy);
    }

    // MonsterId
    let monster_id_re = Regex::new(r#""enemy_components::MonsterId":\s*\("([^"]+)"\)"#).ok()?;
    if let Some(caps) = monster_id_re.captures(content) {
        components.push(EnemyComponent::MonsterId(caps.get(1)?.as_str().to_string()));
    }

    // EnemyRange
    // Matches "enemy_components::EnemyRange": VariantName
    let range_re = Regex::new(r#""enemy_components::EnemyRange":\s*(\w+)"#).ok()?;
    if let Some(caps) = range_re.captures(content) {
        let variant_str = caps.get(1)?.as_str();
        let range = match variant_str {
            "CloseRange" => EnemyComponent::EnemyRange(EnemyRange::CloseRange),
            "MediumRange" => EnemyComponent::EnemyRange(EnemyRange::MediumRange),
            "LongRange" => EnemyComponent::EnemyRange(EnemyRange::LongRange),
            _ => EnemyComponent::EnemyRange(EnemyRange::CloseRange), // Fallback
        };
        components.push(range);
    }

    // DisplayName
    let display_name_re =
        Regex::new(r#""shared_components::DisplayName":\s*\("([^"]+)"\)"#).ok()?;
    if let Some(caps) = display_name_re.captures(content) {
        components.push(EnemyComponent::DisplayName(
            caps.get(1)?.as_str().to_string(),
        ));
    }

    // Health
    let health_re =
        Regex::new(r#""enemy_components::Health":\s*\(current:\s*([\d.]+),\s*max:\s*([\d.]+)\)"#)
            .ok()?;
    if let Some(caps) = health_re.captures(content) {
        let current: f32 = caps.get(1)?.as_str().parse().ok()?;
        let max: f32 = caps.get(2)?.as_str().parse().ok()?;
        components.push(EnemyComponent::Health { current, max });
    }

    // MovementSpeed
    let speed_re = Regex::new(r#""enemy_components::MovementSpeed":\s*\(([\d.]+)\)"#).ok()?;
    if let Some(caps) = speed_re.captures(content) {
        let speed: f32 = caps.get(1)?.as_str().parse().ok()?;
        components.push(EnemyComponent::MovementSpeed(speed));
    }

    // Lifetime - extract duration secs and nanos
    let lifetime_re = Regex::new(r#"duration:\s*\(secs:\s*(\d+),\s*nanos:\s*(\d+)\)"#).ok()?;
    if let Some(caps) = lifetime_re.captures(content) {
        let secs: u64 = caps.get(1)?.as_str().parse().ok()?;
        let nanos: u32 = caps.get(2)?.as_str().parse().ok()?;
        components.push(EnemyComponent::Lifetime { secs, nanos });
    }

    // Transform
    let transform_re = Regex::new(r#""bevy_transform::components::transform::Transform":\s*\(translation:\s*\(([-\d.]+),\s*([-\d.]+),\s*([-\d.]+)\)\)"#).ok()?;
    if let Some(caps) = transform_re.captures(content) {
        let x: f32 = caps.get(1)?.as_str().parse().ok()?;
        let y: f32 = caps.get(2)?.as_str().parse().ok()?;
        let z: f32 = caps.get(3)?.as_str().parse().ok()?;
        components.push(EnemyComponent::Transform { x, y, z });
    }

    // Sprite
    let sprite_re = Regex::new(r#"red:\s*([\d.]+),\s*green:\s*([\d.]+),\s*blue:\s*([\d.]+),\s*alpha:\s*([\d.]+).*?custom_size:\s*Some\(\(([\d.]+),\s*([\d.]+)\)\)"#).ok()?;
    if let Some(caps) = sprite_re.captures(content) {
        let r: f32 = caps.get(1)?.as_str().parse().ok()?;
        let g: f32 = caps.get(2)?.as_str().parse().ok()?;
        let b: f32 = caps.get(3)?.as_str().parse().ok()?;
        let a: f32 = caps.get(4)?.as_str().parse().ok()?;
        let width: f32 = caps.get(5)?.as_str().parse().ok()?;
        let height: f32 = caps.get(6)?.as_str().parse().ok()?;
        components.push(EnemyComponent::Sprite {
            r,
            g,
            b,
            a,
            width,
            height,
        });
    }

    // ResourceRewards
    if content.contains("\"enemy_components::ResourceRewards\"") {
        let rewards_re = Regex::new(r#"\(id:\s*"([^"]+)",\s*value:\s*(\d+)\)"#).ok()?;
        let mut rewards = Vec::new();
        for caps in rewards_re.captures_iter(content) {
            if let (Some(id), Some(value)) = (caps.get(1), caps.get(2)) {
                rewards.push(Reward {
                    id: id.as_str().to_string(),
                    value: value.as_str().parse().unwrap_or(0),
                });
            }
        }
        components.push(EnemyComponent::ResourceRewards(rewards));
    }

    // RewardCoefficient
    let coeff_re = Regex::new(r#""enemy_components::RewardCoefficient":\s*\(([\d.]+)\)"#).ok()?;
    if let Some(caps) = coeff_re.captures(content) {
        let coeff: f32 = caps.get(1)?.as_str().parse().ok()?;
        components.push(EnemyComponent::RewardCoefficient(coeff));
    }

    // NeedsHydration marker
    if content.contains("\"enemy_components::NeedsHydration\"") {
        components.push(EnemyComponent::NeedsHydration);
    }

    if components.is_empty() {
        None
    } else {
        Some(components)
    }
}
