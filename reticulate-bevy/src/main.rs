mod chunk_mesher;
mod demo_mesher;
mod demo_scene;
mod voxel_material;

#[allow(dead_code, unused)]
mod grid_material;

use bevy::{
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    prelude::*,
};

use crate::{
    demo_scene::{CameraPlugin, CameraSettingsPlugin},
    voxel_material::{VoxelMaterial, setup_voxel_material},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..Default::default()
            }),
            CameraPlugin,
            CameraSettingsPlugin,
            InfiniteGridPlugin,
        ))
        .add_systems(Startup, demo_scene::setup)
        .add_systems(Startup, setup_grid)
        .add_systems(Update, demo_scene::ChunkMesh::redraw)
        .add_plugins(MaterialPlugin::<VoxelMaterial>::default())
        .add_systems(Startup, setup_voxel_material)
        .run();
}

fn setup_grid(mut commands: Commands) {
    commands.spawn((InfiniteGrid, InfiniteGridSettings::default()));
}
