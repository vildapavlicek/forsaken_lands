use {
    crate::*,
    bevy::prelude::*,
    skill_events::SkillActivated,
    skills_assets::{SkillDefinition, SkillEffect, SkillType, TargetType},
};

#[test]
fn test_skill_activation_damage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<bonus_stats::BonusStats>() // Mock bonus stats
        .add_observer(systems::process_skill_activation);

    // Create a mock skill
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_handle = skills.add(SkillDefinition {
        id: "test_fireball".to_string(),
        display_name: "Test Fireball".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 1000,
        target: TargetType::Identity, // Easier to test self-target
        effects: vec![SkillEffect::Damage { amount: 10.0 }],
        tags: vec![],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles
        .insert("test_fireball".to_string(), skill_handle);

    // Spawn caster
    let caster = app.world_mut().spawn(Transform::default()).id();

    // We expect a DamageRequest to be triggered.
    // However, DamageRequest is an observer event. We need to spy on it.
    app.add_observer(
        |trigger: On<hero_events::DamageRequest>, mut commands: Commands| {
            let event = trigger.event();
            // We can't assert here easily, but we can verify it runs usage of event.
            // For a unit test, we might want to store results in a resource.
            commands.spawn(TestResult {
                damage: event.base_damage,
            });
        },
    );

    // Trigger skill
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: "test_fireball".to_string(),
        target: None,
        target_position: None,
    });

    app.update();

    let mut query = app.world_mut().query::<&TestResult>();
    assert_eq!(query.iter(app.world()).count(), 1);
    let result = query.single(app.world());
    assert_eq!(result.unwrap().damage, 10.0);
}

#[derive(Component)]
struct TestResult {
    damage: f32,
}
