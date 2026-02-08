pub mod systems;

#[cfg(test)]
mod tests;

use {bevy::prelude::*, skill_components::*, skill_events::*, skills_assets::*};

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SkillsAssetsPlugin, SkillEventsPlugin))
            .add_systems(Update, (systems::tick_cooldowns, systems::tick_buffs))
            .add_observer(systems::process_skill_activation);
    }
}
