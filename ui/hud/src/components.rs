use {bevy::prelude::*, shared_components::IncludeInSave};

#[derive(Component)]
#[require(IncludeInSave)]
pub struct EnemyStatusRoot;

#[derive(Component)]
pub struct HpBarFill;

#[derive(Component)]
pub struct TimeBarFill;
