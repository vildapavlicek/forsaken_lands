use {bevy::prelude::*, std::collections::HashMap};

/// Marker: entity has skills equipped
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HasSkills;

/// References equipped skills by ID
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct EquippedSkills(pub Vec<String>);

/// Tracks cooldown state for active skills
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SkillCooldowns {
    pub timers: HashMap<String, Timer>,
}

impl SkillCooldowns {
    pub fn get_timer_mut(&mut self, id: &str) -> Option<&mut Timer> {
        self.timers.get_mut(id)
    }

    pub fn reset_timer(&mut self, id: &str) {
        self.timers.get_mut(id).map(Timer::reset);
    }
}

/// Temporary buff applied by a skill
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SkillBuff {
    pub source_skill_id: String,
    pub stat_key: String,
    pub value: f32,
    pub mode: u8, // 0=Add, 1=Percent, 2=Mult
    pub timer: Timer,
}

/// Status effect (stun, bleed, etc.)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct StatusEffect {
    pub status_id: String,
    pub timer: Timer,
}
