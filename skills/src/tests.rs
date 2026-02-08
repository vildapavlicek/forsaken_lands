use {
    crate::*,
    bevy::prelude::*,
    bonus_stats::BonusStats,
    skill_events::SkillActivated,
    skills_assets::{SkillDefinition, SkillEffect, SkillType, TargetType},
    std::time::Duration,
};

#[test]
fn test_skill_activation_damage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>() // Mock bonus stats
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
    let result = query
        .iter(app.world())
        .next()
        .expect("Should have one result");
    assert_eq!(result.damage, 10.0);
}

#[test]
fn test_skill_stat_modifier() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>()
        .add_observer(systems::process_skill_activation);

    // Create a mock skill
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_id = "test_buff".to_string();
    let skill_handle = skills.add(SkillDefinition {
        id: skill_id.clone(),
        display_name: "Test Buff".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 0,
        target: TargetType::Identity,
        effects: vec![SkillEffect::StatModifier {
            stat_key: "damage:melee".to_string(),
            value: 0.5,
            mode: StatModifierMode::Percent,
            duration_ms: 5000,
        }],
        tags: vec![],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles.insert(skill_id.clone(), skill_handle);

    // Spawn caster
    let caster = app.world_mut().spawn(Transform::default()).id();

    // Trigger skill
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: None,
        target_position: None,
    });

    app.update();

    // Verify BonusStats
    let bonus_stats = app.world().resource::<BonusStats>();
    let stat = bonus_stats
        .get("damage:melee")
        .expect("Stat bonus should be added");
    assert_eq!(stat.percent, 0.5);

    // Verify SkillBuff entity
    let mut query = app.world_mut().query::<&SkillBuff>();
    let buff = query
        .iter(app.world())
        .next()
        .expect("Should have one buff");
    assert_eq!(buff.source_skill_id, skill_id);
    assert_eq!(buff.stat_key, "damage:melee");
}

#[test]
fn test_skill_apply_status() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>()
        .add_observer(systems::process_skill_activation);

    // Create a mock skill
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_id = "test_stun".to_string();
    let skill_handle = skills.add(SkillDefinition {
        id: skill_id.clone(),
        display_name: "Test Stun".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 0,
        target: TargetType::Identity,
        effects: vec![SkillEffect::ApplyStatus {
            status_id: "stun".to_string(),
            duration_ms: 2000,
        }],
        tags: vec![],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles.insert(skill_id.clone(), skill_handle);

    // Spawn caster
    let caster = app.world_mut().spawn(Transform::default()).id();

    // Trigger skill
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: None,
        target_position: None,
    });

    app.update();

    // Verify StatusEffect component on caster (since it's TargetType::Identity)
    let status = app
        .world()
        .get::<StatusEffect>(caster)
        .expect("Target should have StatusEffect");
    assert_eq!(status.status_id, "stun");
    assert_eq!(status.timer.duration().as_millis(), 2000);
}

#[test]
fn test_skill_cooldown() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>()
        .add_observer(systems::process_skill_activation);

    // Create a mock skill with cooldown
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_id = "test_cooldown".to_string();
    let skill_handle = skills.add(SkillDefinition {
        id: skill_id.clone(),
        display_name: "Test Cooldown".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 1000,
        target: TargetType::Identity,
        effects: vec![SkillEffect::Damage { amount: 10.0 }],
        tags: vec![],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles.insert(skill_id.clone(), skill_handle);

    // Spawn caster
    let caster = app.world_mut().spawn(Transform::default()).id();

    // Spy on DamageRequest
    app.add_observer(
        |trigger: On<hero_events::DamageRequest>, mut commands: Commands| {
            commands.spawn(TestResult {
                damage: trigger.event().base_damage,
            });
        },
    );

    // Add ticking system
    app.add_systems(Update, systems::tick_cooldowns);

    // Trigger 1: Should succeed
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: None,
        target_position: None,
    });
    app.update();

    let mut query = app.world_mut().query::<&TestResult>();
    assert_eq!(
        query.iter(app.world()).count(),
        1,
        "First activation should succeed"
    );

    // Trigger 2: Should fail (on cooldown)
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: None,
        target_position: None,
    });
    app.update();

    let mut query = app.world_mut().query::<&TestResult>();
    assert_eq!(
        query.iter(app.world()).count(),
        1,
        "Second activation should fail due to cooldown"
    );

    // Advance time manually
    {
        let mut cooldowns = app
            .world_mut()
            .get_mut::<SkillCooldowns>(caster)
            .expect("Should have cooldowns");
        let timer = cooldowns
            .timers
            .get_mut(&skill_id)
            .expect("Should have timer");
        timer.tick(Duration::from_millis(1100));
    }

    app.update();

    // Trigger 3: Should succeed (cooldown finished)
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: None,
        target_position: None,
    });
    app.update();

    let mut query = app.world_mut().query::<&TestResult>();
    assert_eq!(
        query.iter(app.world()).count(),
        2,
        "Third activation should succeed after cooldown"
    );
}

#[test]
fn test_skill_target_single_enemy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>()
        .add_observer(systems::process_skill_activation);

    // Create a mock skill
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_id = "test_single".to_string();
    let skill_handle = skills.add(SkillDefinition {
        id: skill_id.clone(),
        display_name: "Test Single".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 0,
        target: TargetType::SingleEnemy { range: 100.0 },
        effects: vec![SkillEffect::Damage { amount: 15.0 }],
        tags: vec![],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles.insert(skill_id.clone(), skill_handle);

    // Spawn caster and target
    let caster = app.world_mut().spawn(Transform::default()).id();
    let target = app.world_mut().spawn(Transform::default()).id();

    // Spy on DamageRequest
    app.add_observer(
        move |trigger: On<hero_events::DamageRequest>, mut commands: Commands| {
            let event = trigger.event();
            if event.target == target {
                commands.spawn(TestResult {
                    damage: event.base_damage,
                });
            }
        },
    );

    // Trigger skill with target
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: Some(target),
        target_position: None,
    });

    app.update();

    let mut query = app.world_mut().query::<&TestResult>();
    let result = query
        .iter(app.world())
        .next()
        .expect("Should have hit the target");
    assert_eq!(result.damage, 15.0);
}

#[test]
fn test_skill_target_aoe_range() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>()
        .add_observer(systems::process_skill_activation);

    // Create a mock skill
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_id = "test_aoe".to_string();
    let skill_handle = skills.add(SkillDefinition {
        id: skill_id.clone(),
        display_name: "Test AOE".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 0,
        target: TargetType::AllEnemiesInRange { radius: 10.0 },
        effects: vec![SkillEffect::Damage { amount: 5.0 }],
        tags: vec![],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles.insert(skill_id.clone(), skill_handle);

    // Spawn caster
    let caster = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Spawn enemies in and out of range
    // Enemy in range (distance ~5)
    app.world_mut()
        .spawn((enemy_components::Enemy, Transform::from_xyz(5.0, 0.0, 0.0)));
    // Enemy out of range (distance ~15)
    app.world_mut()
        .spawn((enemy_components::Enemy, Transform::from_xyz(15.0, 0.0, 0.0)));

    // Spy on DamageRequest
    app.add_observer(
        |trigger: On<hero_events::DamageRequest>, mut commands: Commands| {
            commands.spawn(TestResult {
                damage: trigger.event().base_damage,
            });
        },
    );

    // Trigger skill
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: None,
        target_position: None,
    });

    app.update();

    let mut query = app.world_mut().query::<&TestResult>();
    assert_eq!(
        query.iter(app.world()).count(),
        1,
        "Only one enemy should be in range"
    );
}

#[test]
fn test_skill_projectile_spawns() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(SkillEventsPlugin)
        .init_resource::<SkillMap>()
        .init_resource::<Assets<SkillDefinition>>()
        .init_resource::<BonusStats>()
        .add_observer(systems::process_skill_activation);

    // Create a mock skill with Projectile effect
    let mut skills = app.world_mut().resource_mut::<Assets<SkillDefinition>>();
    let skill_id = "test_projectile".to_string();
    let skill_handle = skills.add(SkillDefinition {
        id: skill_id.clone(),
        display_name: "Test Projectile".to_string(),
        skill_type: SkillType::Active,
        cooldown_ms: 0,
        target: TargetType::SingleEnemy { range: 100.0 },
        effects: vec![SkillEffect::Projectile {
            speed: 500.0,
            damage: 25.0,
        }],
        tags: vec!["skill:fire".to_string()],
    });

    let mut map = app.world_mut().resource_mut::<SkillMap>();
    map.handles.insert(skill_id.clone(), skill_handle);

    // Spawn Village (needed for source position)
    app.world_mut().spawn((
        village_components::Village,
        Transform::from_xyz(10.0, 20.0, 0.0),
    ));

    // Spawn caster and target
    let caster = app.world_mut().spawn(Transform::default()).id();
    let target = app.world_mut().spawn(Transform::default()).id();

    // Spy on ProjectileSpawnRequest
    #[derive(Component)]
    struct ProjectileSpawnResult {
        pos: Vec3,
        target: Entity,
        speed: f32,
        damage: f32,
    }

    app.add_observer(
        move |trigger: On<hero_events::ProjectileSpawnRequest>, mut commands: Commands| {
            let event = trigger.event();
            commands.spawn(ProjectileSpawnResult {
                pos: event.source_position,
                target: event.target,
                speed: event.speed,
                damage: event.base_damage,
            });
        },
    );

    // Trigger skill
    app.world_mut().trigger(SkillActivated {
        caster,
        skill_id: skill_id.clone(),
        target: Some(target),
        target_position: None,
    });

    app.update();

    let mut query = app.world_mut().query::<&ProjectileSpawnResult>();
    let result = query
        .iter(app.world())
        .next()
        .expect("Should have triggered ProjectileSpawnRequest");

    assert_eq!(result.pos, Vec3::new(10.0, 20.0, 0.0));
    assert_eq!(result.target, target);
    assert_eq!(result.speed, 500.0);
    assert_eq!(result.damage, 25.0);
}

#[derive(Component)]
struct TestResult {
    damage: f32,
}
