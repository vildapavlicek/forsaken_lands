use bevy::prelude::*;

pub struct SkillEventsPlugin;

impl Plugin for SkillEventsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkillActivated>()
            .register_type::<SkillEffectApplied>();
    }
}

/// Represents a confirmed request to execute a skill's effects.
///
/// This **Observer** event (triggered via `commands.trigger`) decouples the decision
/// to use a skill (from AI or Input) from the actual execution of its logic.
///
/// # Observers
/// - `process_skill_activation` (in `skills/src/systems.rs`): Receives the event,
///   evaluates the skill's effects, and triggers subsequent damage or healing events.
///
/// # Implicit Dependencies
/// - **Range Validation**: Triggering systems are entirely responsible for validating target
///   range before emitting this event, especially for `TargetType::SingleEnemy`. The
///   execution logic assumes the target is already valid.
#[derive(Event, Debug, Reflect)]
#[reflect(Default)]
pub struct SkillActivated {
    pub caster: Entity,
    pub skill_id: String,
    pub target: Option<Entity>,
    pub target_position: Option<Vec2>,
}

impl Default for SkillActivated {
    fn default() -> Self {
        Self {
            caster: Entity::PLACEHOLDER,
            skill_id: String::new(),
            target: None,
            target_position: None,
        }
    }
}

/// Emitted after each effect is processed (for VFX/Audio hooks)
#[derive(Event, Debug, Reflect)]
#[reflect(Default)]
pub struct SkillEffectApplied {
    pub caster: Entity,
    pub skill_id: String,
    pub effect_index: usize,
    pub targets: Vec<Entity>,
}

impl Default for SkillEffectApplied {
    fn default() -> Self {
        Self {
            caster: Entity::PLACEHOLDER,
            skill_id: String::new(),
            effect_index: 0,
            targets: Vec::new(),
        }
    }
}
