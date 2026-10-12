use bevy::prelude::*;

use crate::demo_mesher::DemoMesher;

#[derive(Component)]
pub struct ChunkMesh;

impl ChunkMesh {
    pub fn redraw(
        mut commands: Commands,
        materials: ResMut<Assets<StandardMaterial>>,
        meshes: ResMut<Assets<Mesh>>,
        mesher: ResMut<DemoMesher>,
        query: Query<Entity, With<ChunkMesh>>,
        keys: Res<ButtonInput<KeyCode>>,
    ) {
        if keys.just_pressed(KeyCode::Enter) {
            for entity in query {
                commands.entity(entity).despawn();
            }

            Self::draw_mesh(&mut commands, mesher, meshes, materials);
        }
    }

    fn draw_mesh(
        commands: &mut Commands,
        mut demo_mesher: ResMut<DemoMesher>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let mesh = demo_mesher.next_demo();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChunkMesh,
        ));
    }
}
