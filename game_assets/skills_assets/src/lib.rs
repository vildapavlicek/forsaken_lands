use {
    bevy::{platform::collections::HashMap, prelude::*},
    bevy_common_assets::ron::RonAssetPlugin,
    serde::{Deserialize, Serialize},
};

pub struct SkillsAssetsPlugin;

impl Plugin for SkillsAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SkillDefinition>::new(&["skill.ron"]))
            .init_resource::<SkillMap>()
            .register_type::<SkillType>()
            .register_type::<SkillEffect>()
            .register_type::<TargetType>()
            .register_type::<StatModifierMode>();
    }
}

/// Top-level skill definition loaded from `.skill.ron`.
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Unique identifier (e.g., "fireball", "iron_skin")
    pub id: String,
    /// Display name shown in UI
    pub display_name: String,
    /// Skill categorization
    pub skill_type: SkillType,
    /// Cooldown in milliseconds (0 = no cooldown, passive skills ignore this)
    #[serde(default)]
    pub cooldown_ms: u32,
    /// Target selection
    #[serde(default)]
    pub target: TargetType,
    /// Composable list of effects applied when skill activates
    pub effects: Vec<SkillEffect>,
    /// Tags for bonus stat lookups (e.g., ["skill:fire", "skill:aoe"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional inline unlock definition for when this skill becomes available
    #[serde(default)]
    pub unlock: Option<unlocks_assets::UnlockDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Serialize, Deserialize)]
pub enum SkillType {
    /// Triggered on input/condition
    #[default]
    Active,
    /// Always-on effect, applies buffs/debuffs continuously
    Passive,
    /// Triggered automatically on specific events (e.g., on_hit, on_kill)
    Reactive { trigger: ReactiveTrigger },
    /// Triggered automatically when cooldown finishes
    AutoActivate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Serialize, Deserialize)]
pub enum ReactiveTrigger {
    #[default]
    OnHit,
    OnKill,
    OnDamageTaken,
}

/// Defines how a skill selects its targets.
///
/// This enum determines both the targeting logic (who gets hit) and the
/// validation constraints (range/radius).
///
/// # Usage
/// - **Skill Definition**: Used in `.skill.ron` assets to configure the skill.
/// - **Execution**: The `process_skill_activation` system uses this to resolve
///   the final list of affected entities.
/// - **Validation**: Systems triggering `SkillActivated` (like AI or Input) must
///   respect the `range`/`radius` constraints defined here, as `process_skill_activation`
///   assumes the target is valid.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Serialize, Deserialize)]
pub enum TargetType {
    /// Applies the effect to the caster itself (e.g., self-buffs).
    #[default]
    Identity,
    /// Targets a specific enemy entity.
    ///
    /// The triggering system must ensure the target is within `range`.
    SingleEnemy {
        /// Maximum allowed distance to target in pixels.
        range: f32,
    },
    /// Targets all enemies within a radius around the caster.
    AllEnemiesInRange {
        /// Radius of the area of effect in pixels.
        radius: f32,
    },
    /// Targets a specific position in the world.
    ///
    /// Useful for ground-targeted AoE spells (e.g., "Meteor Strike").
    Point {
        /// Radius of the area of effect in pixels.
        radius: f32,
    },
}

/// Atomic skill effect - compose these to build complex skills.
/// Extensible: add new variants without modifying core resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum SkillEffect {
    // === Damage Effects ===
    /// Deal flat damage to targets
    Damage { amount: f32 },
    /// Deal damage as percentage of target's max HP
    DamagePercent { percent: f32 },

    // === Buff/Debuff Effects ===
    /// Apply a stat modifier for a duration
    StatModifier {
        stat_key: String,
        value: f32,
        mode: StatModifierMode,
        duration_ms: u32,
    },

    // === Status Effects ===
    /// Apply a named status effect
    ApplyStatus { status_id: String, duration_ms: u32 },

    // === Utility Effects ===
    /// Heal the caster or target
    Heal { amount: f32 },
    /// Spawn an entity (trap, projectile, summon)
    Spawn { prefab_id: String },

    /// Spawn a projectile that deals damage on impact
    Projectile {
        /// Projectile travel speed
        speed: f32,
        /// Damage dealt on impact
        damage: f32,
    },

    // === Conditional/Meta Effects ===
    /// Only apply inner effects if condition is met
    Conditional {
        condition: EffectCondition,
        effects: Vec<SkillEffect>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect, Default)]
#[reflect(Serialize, Deserialize)]
pub enum StatModifierMode {
    #[default]
    Additive,
    Percent,
    Multiplicative,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum EffectCondition {
    /// Target HP below percentage (0.0-1.0)
    TargetHealthBelow(f32),
    /// Random chance (0.0-1.0)
    Chance(f32),
    /// Caster has a specific status
    HasStatus(String),
}

/// Resource mapping skill IDs to asset handles.
#[derive(Resource, Default)]
pub struct SkillMap {
    pub handles: HashMap<String, Handle<SkillDefinition>>,
}
