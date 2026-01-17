#[cfg(test)]
mod tests {
    // Assuming these are available via super usage in lib.rs or I need correct imports
    // Since this is unlocks/src/tests.rs, it is a module of unlocks.
    // So `super` refers to `unlocks`.
    use {
        crate::{
            ConditionSensor, LogicSignalEvent, MaxUnlockedDivinityChangedEvent,
            MaxUnlockedDivinitySensor, TopicEntity, TopicMap, TopicSubscribers,
        },
        bevy::prelude::*,
        divinity_components::Divinity,
    };

    #[test]
    fn test_divinity_sensor_updateds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<TopicMap>();
        // Add observer
        app.add_observer(crate::systems::on_max_divinity_changed);

        // Track signals
        #[derive(Resource, Default)]
        struct SignalTracker(Vec<bool>);
        app.init_resource::<SignalTracker>();
        app.add_observer(
            |trigger: On<LogicSignalEvent>, mut tracker: ResMut<SignalTracker>| {
                tracker.0.push(trigger.event().is_high);
            },
        );

        // 1. Setup Topic
        let topic_key = "stat:max_unlocked_divinity".to_string();
        let topic_entity = app
            .world_mut()
            .spawn((
                TopicEntity {
                    key: topic_key.clone(),
                },
                TopicSubscribers::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<TopicMap>()
            .topics
            .insert(topic_key, topic_entity);
        app.update(); // Apply commands

        // 2. Setup Sensor (Child)
        // Required: Tier 1, Level 5
        let required = Divinity::new(1, 5);
        let sensor = app
            .world_mut()
            .spawn((
                ConditionSensor { is_met: false },
                MaxUnlockedDivinitySensor(required),
            ))
            .id();

        // 3. Subscribe
        app.world_mut()
            .entity_mut(topic_entity)
            .get_mut::<TopicSubscribers>()
            .unwrap()
            .sensors
            .push(sensor);

        // 4. Test: Level too low (Tier 1, Level 2) => No Signal (false -> false, no change)
        let event = MaxUnlockedDivinityChangedEvent {
            entity: topic_entity,
            new_divinity: Divinity::new(1, 2),
        };
        app.world_mut().trigger(event);
        app.update();

        {
            let tracker = app.world().resource::<SignalTracker>();
            assert!(
                tracker.0.is_empty(),
                "Should not trigger if state remains false"
            );
        }

        // 5. Test: Level met (Tier 1, Level 10) => Signal High
        let event = MaxUnlockedDivinityChangedEvent {
            entity: topic_entity,
            new_divinity: Divinity::new(1, 10),
        };
        app.world_mut().trigger(event);
        app.update();

        {
            let tracker = app.world().resource::<SignalTracker>();
            assert_eq!(tracker.0.len(), 1);
            assert_eq!(tracker.0[0], true);
        }

        // 6. Test: Level drops (Tier 1, Level 1) => Signal Low
        let event = MaxUnlockedDivinityChangedEvent {
            entity: topic_entity,
            new_divinity: Divinity::new(1, 1),
        };
        app.world_mut().trigger(event);
        app.update();

        {
            let tracker = app.world().resource::<SignalTracker>();
            assert_eq!(tracker.0.len(), 2);
            assert_eq!(tracker.0[1], false);
        }
    }
}
