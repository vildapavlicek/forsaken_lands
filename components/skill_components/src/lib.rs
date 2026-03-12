use {bevy::prelude::*, std::collections::{HashMap, HashSet}};

pub struct SkillComponentsPlugin;

impl Plugin for SkillComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnlockedSkills>()
            .register_type::<HasSkills>()
            .register_type::<UnlockedSkills>()
            .register_type::<EquippedSkills>()
            .register_type::<SkillCooldowns>()
            .register_type::<SkillBuff>()
            .register_type::<StatusEffect>();
    }
}

/// Marker: entity has skills equipped
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HasSkills;

/// Tracks the global pool of skills an entity has gained access to.
///
/// This resource serves as the progression gate for abilities. Skills must be present
/// here before they can be equipped and utilized in combat.
///
/// # Usage
/// - **Progression**: Queried and mutated by progression systems (e.g., unlocking via research
///   or level up) to add new skill IDs.
/// - **UI Management**: Read by the `hero_ui` system to populate the list of available
///   skills that a player can choose to equip.
#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct UnlockedSkills(pub HashSet<String>);

/// Represents the active skillset currently available for an entity to use in combat.
///
/// This component acts as the execution container for skills. While an entity might have
/// many `UnlockedSkills`, only the IDs listed here can be triggered during gameplay.
///
/// # Usage
/// - **Combat Execution**: The `hero_auto_activate_skills_system` and `enemy_auto_activate_skills_system`
///   query this component to determine which skills to evaluate and potentially trigger.
/// - **Asset Lookup**: The `String` elements are stable IDs used to retrieve `Handle<SkillDefinition>`
///   from the `SkillMap` resource, allowing systems to access the actual skill logic and stats.
/// - **UI Management**: Queried and mutated by `hero_ui` to display active skills and allow
///   the player to equip/unequip different abilities.
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
