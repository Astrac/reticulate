mod mesher;
#[allow(dead_code)]
mod test_model;

use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use reticulate::*;

use crate::voxel::mesher::Mesher;

#[derive(Component)]
pub struct ChunkMesh;

pub fn redraw(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mesher: ResMut<Mesher>,
    query: Query<Entity, With<ChunkMesh>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Enter) || query.is_empty() {
        println!("REDRAWING");
        for entity in query {
            commands.entity(entity).despawn();
        }

        draw_mesh(&mut commands, mesher, meshes, materials);
    }
}

fn draw_mesh(
    commands: &mut Commands,
    mut mesher: ResMut<Mesher>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let plane_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(32.0)));
    // let plane_mat = materials.add(StandardMaterial {
    //     base_color: Color::xyza(0.5, 0.5, 0.5, 0.03),
    //     cull_mode: None,
    //     double_sided: true,
    //     alpha_mode: AlphaMode::Blend,
    //     ..default()
    // });

    // for x in 0..65 {
    //     commands.spawn((
    //         Mesh3d(plane_mesh.clone()),
    //         MeshMaterial3d(plane_mat.clone()),
    //         ChunkMesh,
    //         Transform::from_rotation(Quat::from_rotation_z(PI / 2.0))
    //             .with_translation(Vec3::new(x as f32, 32.0, 32.0)),
    //     ));
    // }
    // for y in 0..65 {
    //     commands.spawn((
    //         Mesh3d(plane_mesh.clone()),
    //         MeshMaterial3d(plane_mat.clone()),
    //         ChunkMesh,
    //         Transform::from_translation(Vec3::new(32.0, y as f32, 32.0)),
    //     ));
    // }
    // for z in 0..65 {
    //     commands.spawn((
    //         Mesh3d(plane_mesh.clone()),
    //         MeshMaterial3d(plane_mat.clone()),
    //         ChunkMesh,
    //         Transform::from_rotation(Quat::from_rotation_x(PI / 2.0))
    //             .with_translation(Vec3::new(32.0, 32.0, z as f32)),
    //     ));
    // }

    let octree = test_model::test_scene();
    {
        let mut walker = SimpleFaceWalker::new(mesher.on_mesh_face_handler());
        octree.walk_dense(&mut walker);
    }

    let mesh = mesher.build_mesh();

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChunkMesh,
    ));
}

pub fn setup(mut commands: Commands) {
    let light_color = Color::srgb(0.98, 0.85, 0.72);
    let mesher = Mesher::new();
    commands.insert_resource(mesher);

    // Sun
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        maximum_distance: 7000.0,
        ..default()
    }
    .build();

    commands.spawn((
        DirectionalLight {
            color: light_color.darker(0.3),
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.1, 0.15), Vec3::Y),
        cascade_shadow_config,
    ));

    // // Ambient light, same color as sun
    commands.insert_resource(GlobalAmbientLight {
        color: light_color,
        brightness: 100.4,
        affects_lightmapped_meshes: true,
    });
}
