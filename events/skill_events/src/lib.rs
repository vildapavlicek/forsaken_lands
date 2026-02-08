use bevy::prelude::*;

pub struct SkillEventsPlugin;

impl Plugin for SkillEventsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkillActivated>()
            .register_type::<SkillEffectApplied>();
    }
}

/// Request to activate a skill
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
