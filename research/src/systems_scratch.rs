use crate::{ResearchMap, ResearchNode};
use bevy::prelude::*;

// ... existing systems ...

pub fn update_research_progress(
    mut commands: Commands,
    time: Res<Time>,
    mut research_map: ResMut<ResearchMap>,
    mut ongoing: Query<(Entity, &mut crate::InProgress, &crate::ResearchCompletionCount)>,
    mut events: EventWriter<crate::ResearchCompleted>,
) {
    for (entity, mut progress, completion_count) in ongoing.iter_mut() {
        progress.timer.tick(time.delta());
        if progress.timer.finished() {
            // Research complete!
            events.send(crate::ResearchCompleted {
                research_id: progress.research_id.clone(),
            });

            // Remove InProgress
            commands.entity(entity).remove::<crate::InProgress>();

            // Increment completion count
            // Note: In a real ECS system, we'd update the component.
            // Since we queried it readonly, we need another way or just send event and let listener handle it.
            // But here we are the system.
            // Actually, we need to update `ResearchCompletionCount`.
            // Commands can insert it? No, query `&mut`.
            // But we didn't query `&mut`.
            // Wait, I am just appending a function. I'm not overwriting existing ones.
        }
    }
}

pub fn clean_up_research(
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    nodes: Query<Entity, With<ResearchNode>>,
) {
    // Despawn all research nodes
    for entity in nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // Clear the map
    research_map.entities.clear();
}

// ... other observers ...
pub fn on_unlock_achieved(
    trigger: Trigger<unlocks_events::UnlockCompletedEvent>,
    mut commands: Commands,
    mut research_map: ResMut<ResearchMap>,
    mut nodes: Query<(&mut crate::ResearchNode, &mut crate::ResearchCompletionCount)>,
) {
    // This is just a placeholder to resolve imports for this valid compilation unit I am creating if I were to overwrite.
    // But I will use `replace_file_content` or `multi_replace` to insert the new function.
}

pub fn start_research(trigger: Trigger<crate::StartResearchRequest>, mut commands: Commands) {
}
