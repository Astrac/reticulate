mod vertices;
#[allow(dead_code)]
mod test_model;

use bevy::asset::RenderAssetUsages;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::voxel::vertices::Vertices;

#[derive(Component)]
pub struct ChunkMesh;

pub fn redraw(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mesher: ResMut<Vertices>,
    query: Query<Entity, With<ChunkMesh>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        println!("REDRAWING");
        for entity in query {
            commands.entity(entity).despawn();
        }

        draw_mesh(&mut commands, mesher, meshes, materials);
    }
}

fn draw_mesh(
    commands: &mut Commands,
    mut mesher: ResMut<Vertices>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let octree = test_model::test_scene();
    mesher.build_mesh(octree);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // TODO - Optimize clone (use mem::take)
    mesh.insert_indices(Indices::U32(mesher.indices.clone()));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesher.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesher.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesher.colors.clone());

    if !mesher.positions.is_empty() {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                // cull_mode: None,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChunkMesh,
        ));
    } else {
        println!("NO POSITIONS!");
    }
}

pub fn setup(mut commands: Commands) {
    let light_color = Color::srgb(0.98, 0.85, 0.72);
    let mesher = Vertices::new();
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
