use {
    bevy::prelude::*,
    buildings_components::{EntropyGenerator, TheMaw},
    shared_components::IncludeInSave,
    unlocks_events::{StatusCompleted, ValueChanged, CRAFTING_TOPIC_PREFIX},
    wallet::Wallet,
};

pub struct TheMawPlugin;

impl Plugin for TheMawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, generate_entropy)
           .add_observer(on_construction_completed);
    }
}

/// Spawns 'The Maw' when construction is completed.
fn on_construction_completed(
    trigger: On<StatusCompleted>,
    mut commands: Commands,
) {
    let event = trigger.event();
    // Check if the completed crafting was "the_maw"
    // Topic format: "craft:{recipe_id}"
    if event.topic == format!("{}bone_idol", CRAFTING_TOPIC_PREFIX) {
        commands.spawn((
            TheMaw,
            EntropyGenerator::default(),
            IncludeInSave,
            Name::new("The Maw"),
        ));
        info!("Spawned 'The Maw' building (Construction Complete)");
    }
}

/// Generates entropy over time for entities with EntropyGenerator.
fn generate_entropy(
    time: Res<Time>,
    mut query: Query<&mut EntropyGenerator>,
    mut wallet: ResMut<Wallet>,
    mut commands: Commands,
) {
    for mut generator in &mut query {
        if generator.timer.tick(time.delta()).just_finished() {
            let entropy_amount = 1;
            let current = wallet.resources.entry("entropy".to_string()).or_insert(0);
            *current += entropy_amount;

            commands.trigger(ValueChanged {
                topic: "resource:entropy".to_string(),
                value: *current as f32,
            });

            trace!("Generated {} Entropy", entropy_amount);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use buildings_components::EntropyGenerator;
    use wallet::Wallet;
    use std::time::Duration;

    #[test]
    fn test_entropy_generation() {
        let mut app = App::new();
        // Don't use MinimalPlugins to avoid Time conflict, just add what we need
        app.init_resource::<Wallet>()
           .init_resource::<Time>() // Initialize Time explicitly
           .add_systems(Update, generate_entropy);

        // Spawn entity with EntropyGenerator
        app.world_mut().spawn(EntropyGenerator::default());

        // Initial check
        assert_eq!(*app.world().resource::<Wallet>().resources.get("entropy").unwrap_or(&0), 0);

        // First update to initialize systems (Time delta is 0)
        app.update();

        // Advance time manually
        let mut time = app.world().resource::<Time>().clone();
        time.advance_by(Duration::from_secs_f32(1.1));
        app.insert_resource(time);
        
        // Run update to process the time advance
        app.update();

        // Check wallet
        let wallet = app.world().resource::<Wallet>();
        let entropy = *wallet.resources.get("entropy").unwrap_or(&0);
        
        println!("Entropy in wallet: {}", entropy);
        assert_eq!(entropy, 1, "Expected 1 entropy after 1.1 seconds");
    }
}
