//! Define common resources used for asset loading
//!

use bevy::{asset::LoadedFolder, prelude::*};

#[derive(Debug, Resource)]
pub(super) struct EnemyPrefabsFolderHandle(pub Handle<LoadedFolder>);
