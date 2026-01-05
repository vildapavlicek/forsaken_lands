//! Scene builder - generates Bevy-compatible scene RON output.
//!
//! This module creates the exact RON format that Bevy expects for dynamic scenes.

use crate::components::EnemyComponent;

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
        EnemyComponent::Enemy => {
            Some(r#""enemy_components::Enemy": ()"#.to_string())
        }
        EnemyComponent::MonsterId(id) => {
            Some(format!(r#""enemy_components::MonsterId": ("{}")"#, escape_ron_string(id)))
        }
        EnemyComponent::DisplayName(name) => {
            Some(format!(r#""shared_components::DisplayName": ("{}")"#, escape_ron_string(name)))
        }
        EnemyComponent::Health { current, max } => {
            Some(format!(
                r#""enemy_components::Health": (current: {}, max: {})"#,
                format_f32(*current),
                format_f32(*max)
            ))
        }
        EnemyComponent::MovementSpeed(speed) => {
            Some(format!(r#""enemy_components::MovementSpeed": ({})"#, format_f32(*speed)))
        }
        EnemyComponent::Lifetime { secs, nanos } => {
            Some(format!(
                r#""enemy_components::Lifetime": ((stopwatch: (elapsed: (secs: 0, nanos: 0), is_paused: false), duration: (secs: {}, nanos: {}), mode: Once, finished: false, times_finished_this_tick: 0))"#,
                secs, nanos
            ))
        }
        EnemyComponent::Transform { x, y, z } => {
            Some(format!(
                r#""bevy_transform::components::transform::Transform": (translation: ({}, {}, {}))"#,
                format_f32(*x),
                format_f32(*y),
                format_f32(*z)
            ))
        }
        EnemyComponent::Sprite { r, g, b, a, width, height } => {
            Some(format!(
                r#""bevy_sprite::sprite::Sprite": (color: Srgba(Srgba(red: {}, green: {}, blue: {}, alpha: {})), custom_size: Some(({}, {})))"#,
                format_f32(*r),
                format_f32(*g),
                format_f32(*b),
                format_f32(*a),
                format_f32(*width),
                format_f32(*height)
            ))
        }
        EnemyComponent::ResourceRewards(rewards) => {
            if rewards.is_empty() {
                Some(r#""enemy_components::ResourceRewards": ([])"#.to_string())
            } else {
                let rewards_str = rewards
                    .iter()
                    .map(|r| format!(r#"(id: "{}", value: {})"#, escape_ron_string(&r.id), r.value))
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!(r#""enemy_components::ResourceRewards": ([{}])"#, rewards_str))
            }
        }
        EnemyComponent::RewardCoefficient(coeff) => {
            Some(format!(r#""enemy_components::RewardCoefficient": ({})"#, format_f32(*coeff)))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::default_required_components;

    #[test]
    fn test_default_components_generate_valid_ron() {
        let components = default_required_components();
        let ron = build_scene_ron(&components);

        // Check it contains expected fields
        assert!(ron.contains("enemy_components::Enemy"));
        assert!(ron.contains("enemy_components::MonsterId"));
        assert!(ron.contains("shared_components::DisplayName"));
        assert!(ron.contains("enemy_components::Health"));
        assert!(ron.contains("bevy_transform::components::transform::Transform"));
        assert!(ron.contains("bevy_sprite::sprite::Sprite"));
    }

    #[test]
    fn test_format_f32_adds_decimal() {
        assert_eq!(format_f32(1.0), "1");
        assert_eq!(format_f32(1.5), "1.5");
        // Integer case
        let v: f32 = 5.0;
        let s = format_f32(v);
        assert!(s.contains('.'), "Expected decimal in: {}", s);
    }
}
