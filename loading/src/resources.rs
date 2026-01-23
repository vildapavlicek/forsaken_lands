//! Define common resources used for asset loading

use bevy::{asset::LoadedFolder, prelude::*};

#[derive(Debug, Resource)]
pub(super) struct EnemyPrefabsFolderHandle(pub Handle<LoadedFolder>);

#[derive(Debug, Resource)]
pub(super) struct UnlocksFolderHandle(pub Handle<LoadedFolder>);

#[derive(Debug, Resource)]
pub(super) struct ResearchFolderHandle(pub Handle<LoadedFolder>);

#[derive(Debug, Resource)]
pub(super) struct RecipesFolderHandle(pub Handle<LoadedFolder>);

#[derive(Debug, Resource)]
pub(super) struct WeaponsFolderHandle(pub Handle<LoadedFolder>);

#[derive(Debug, Resource)]
pub(super) struct BlessingsFolderHandle(pub Handle<LoadedFolder>);
