// use bevy::{color::palettes::tailwind::GREEN_400, prelude::*};
// use bevy_voxel_world::prelude::*;
//
// use crate::world::MainWorld;
//
// pub fn setup(
//     _voxel_world: VoxelWorld<MainWorld>,
//     _camera_info: Query<(&Camera3d, &Transform)>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let empty_pos = Vec3::new(0.0, 0.0, 0.0);
//
//     commands.spawn((
//         Mesh3d(meshes.add(Sphere::new(10.0))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color: GREEN_400.into(),
//             ..default()
//         })),
//         Transform::from_xyz(empty_pos.x, empty_pos.y, empty_pos.z),
//     ));
// }
