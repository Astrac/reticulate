mod camera;
mod material;
mod player;
mod voxel;
mod world;

use bevy::{
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    prelude::*,
};

use crate::camera::{CameraPlugin, CameraSettingsPlugin};

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
        .add_systems(Startup, voxel::setup)
        .add_systems(Startup, setup_grid)
        .add_systems(Update, voxel::redraw)
        .run();
}

fn setup_grid(mut commands: Commands) {
    commands.spawn((InfiniteGrid, InfiniteGridSettings::default()));
}

// use bevy::{pbr::ExtendedMaterial, prelude::*};
// use bevy_voxel_world::prelude::VoxelWorldPlugin;
//
// use crate::{
//     camera::{CameraPlugin, CameraSettingsPlugin},
//     material::{GridMaterialUniforms, GridMaterialVoxel},
//     world::MainWorld,
// };

// fn main() {
//     App::new()
//         .add_plugins((
//             DefaultPlugins.set(AssetPlugin {
//                 watch_for_changes_override: Some(true),
//                 ..Default::default()
//             }),
//             CameraPlugin,
//             CameraSettingsPlugin,
//             // MaterialPlugin::<ExtendedMaterial<StandardMaterial, GridMaterialMesh>>::default(),
//             MaterialPlugin::<ExtendedMaterial<StandardMaterial, GridMaterialVoxel>>::default(),
//             VoxelWorldPlugin::with_config(MainWorld).with_material(ExtendedMaterial {
//                 base: StandardMaterial {
//                     base_color: LinearRgba::from_vec3(Vec3::new(0.176, 0.106, 0.412)).into(),
//                     metallic: 0.8,
//                     emissive: LinearRgba::from_vec3(Vec3::new(0.176, 0.106, 0.412) * 0.05),
//                     ..default()
//                 },
//                 extension: GridMaterialVoxel {
//                     uniforms: GridMaterialUniforms {
//                         grid_scale: 5.0,
//                         line_width: 0.02,
//                     },
//                 },
//             }),
//         ))
//         .add_systems(Startup, world::setup)
//         .add_systems(PostStartup, player::setup)
//         .run();
// }
