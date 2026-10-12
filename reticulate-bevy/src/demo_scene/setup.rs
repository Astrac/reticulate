use bevy::{ecs::system::Commands, light::CascadeShadowConfigBuilder, prelude::*};

use crate::demo_mesher::DemoMesher;

pub fn setup(mut commands: Commands) {
    let light_color = Color::srgb(0.98, 0.85, 0.72);
    let mesher = DemoMesher::new();
    commands.insert_resource(mesher);

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

    commands.insert_resource(GlobalAmbientLight {
        color: light_color,
        brightness: 100.4,
        affects_lightmapped_meshes: true,
    });
}
