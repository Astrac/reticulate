use bevy::prelude::*;

use crate::{demo_mesher::DemoMesher, voxel_material::VoxelMaterialHandle};

#[derive(Component)]
pub struct ChunkMesh;

impl ChunkMesh {
    pub fn redraw(
        mut commands: Commands,
        material: Res<VoxelMaterialHandle>,
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
    }
}
