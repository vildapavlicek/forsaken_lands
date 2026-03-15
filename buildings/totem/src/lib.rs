use {
    bevy::prelude::*,
    shared_components::IncludeInSave,
    unlocks_events::StatusCompleted,
    recipes_assets::CONSTRUCTION_TOPIC_PREFIX,
    skill_components::{HasSkills, EquippedSkills},
};

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
#[require(Pickable, IncludeInSave)]
pub struct Totem;

pub struct TotemPlugin;

impl Plugin for TotemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Totem>()
           .add_observer(on_construction_completed);
    }
}

/// Spawns the 'Totem' when construction is completed.
fn on_construction_completed(
    trigger: On<StatusCompleted>,
    mut commands: Commands,
    existing_totem: Query<(), With<Totem>>,
) {
    let event = trigger.event();
    let construction_topic = format!("{}totem", CONSTRUCTION_TOPIC_PREFIX);

    if event.topic == construction_topic {
        if !existing_totem.is_empty() {
            warn!(
                "Prevented duplicate spawn of 'Totem'. Event topic: {}",
                event.topic
            );
            return;
        }

        commands.spawn((
            Totem,
            IncludeInSave,
            HasSkills,
            EquippedSkills(vec!["totem_base_damage".to_string()]),
            Transform::from_translation(Vec3::ZERO),
            Name::new("Totem"),
        ));
        info!(
            "Spawned 'Totem' building (Construction Complete). Topic: {}",
            event.topic
        );
    }
}
