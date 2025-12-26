use {
    bevy::prelude::*,
    enemy_components::{Enemy, Health},
    hero_components::{AttackRange, AttackSpeed, Damage, Hero},
    system_schedule::GameSchedule,
};

pub struct HeroesPlugin;

impl Plugin for HeroesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hero>()
            .register_type::<Damage>()
            .register_type::<AttackRange>()
            .register_type::<AttackSpeed>();

        app.add_systems(Update, hero_attack_system.in_set(GameSchedule::PerformAction));
    }
}

fn hero_attack_system(
    time: Res<Time>,
    mut heroes: Query<(&Transform, &Damage, &AttackRange, &mut AttackSpeed), With<Hero>>,
    mut enemies: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
) {
    for (hero_transform, damage, range, mut attack_speed) in heroes.iter_mut() {
        if attack_speed.timer.tick(time.delta()).just_finished() {
            let mut closest_enemy: Option<(Entity, f32)> = None;

            for (enemy_entity, enemy_transform, _) in enemies.iter() {
                let distance = hero_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance <= range.0 {
                    if let Some((_, closest_distance)) = closest_enemy {
                        if distance < closest_distance {
                            closest_enemy = Some((enemy_entity, distance));
                        }
                    } else {
                        closest_enemy = Some((enemy_entity, distance));
                    }
                }
            }

            if let Some((enemy_entity, _)) = closest_enemy {
                if let Ok((_, _, mut health)) = enemies.get_mut(enemy_entity) {
                    health.current -= damage.0;
                    info!("Hero attacked enemy {:?} for {} damage. Health: {}/{}", enemy_entity, damage.0, health.current, health.max);
                }
            }
        }
    }
}
