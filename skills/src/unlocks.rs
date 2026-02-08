use {bevy::prelude::*, skill_components::*, unlocks_events::UnlockAchieved};

/// Observer that listens for UnlockAchieved events and updates UnlockedSkills component.
pub fn handle_skill_unlocks(trigger: On<UnlockAchieved>, mut unlocked: ResMut<UnlockedSkills>) {
    let event = trigger.event();

    // We expect the reward_id to match the skill_id (e.g. "fireball")
    // or optionally have a prefix like "skill:fireball".
    // For now, let's just use the reward_id directly as the skill_id.

    let Some(skill_id) = event.reward_id.strip_prefix("skill:") else {
        return;
    };

    debug!(skill_id = %skill_id, "Skill unlocked! Updating UnlockedSkills.");

    if unlocked.0.insert(skill_id.to_owned()) {
        info!(skill_id = %skill_id, "Added skill to unlocked set.");
    }
}
