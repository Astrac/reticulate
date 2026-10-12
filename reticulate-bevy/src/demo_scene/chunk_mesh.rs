use bevy::prelude::*;

use crate::{
    demo_mesher::DemoMesher,
    voxel_material::{VoxelMaterial, VoxelMaterialHandle},
};

const Z_TRIANGLES_ACTIVE: f32 = 0.8;
const W_TRIANGLES_ACTIVE: f32 = 0.7;

#[derive(Component)]
pub struct ChunkMesh;

impl ChunkMesh {
    pub fn redraw(
        mut commands: Commands,
        material: Res<VoxelMaterialHandle>,
        mut materials: ResMut<Assets<VoxelMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut demo_mesher: ResMut<DemoMesher>,
        query: Query<Entity, With<ChunkMesh>>,
        keys: Res<ButtonInput<KeyCode>>,
    ) {
        if keys.just_pressed(KeyCode::Enter) {
            for entity in query {
                commands.entity(entity).despawn();
            }

            let mesh = demo_mesher.next_demo();

            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.0.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
                ChunkMesh,
            ));
        }

        if keys.just_pressed(KeyCode::Tab) {
            let mut material = materials
                .get_mut(material.0.id())
                .expect("VoxelMaterial not found");

            if material.extension.border.z == Z_TRIANGLES_ACTIVE {
                material.extension.border.z = 0.0;
                material.extension.border.w = 0.0;
            } else {
                material.extension.border.z = Z_TRIANGLES_ACTIVE;
                material.extension.border.w = W_TRIANGLES_ACTIVE;
            }
        }
    }
}
