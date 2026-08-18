// use bevy::{light::CascadeShadowConfigBuilder, platform::collections::HashMap, prelude::*};
// use bevy_voxel_world::prelude::*;
// use noise::{HybridMulti, NoiseFn, Perlin};
//
// #[derive(Resource, Clone, Default)]
// pub struct MainWorld;
//
// impl VoxelWorldConfig for MainWorld {
//     type MaterialIndex = u8;
//     type ChunkUserBundle = ();
//
//     fn spawning_distance(&self) -> u32 {
//         25
//     }
//
//     fn min_despawn_distance(&self) -> u32 {
//         1
//     }
//
//     fn voxel_lookup_delegate(&self) -> VoxelLookupDelegate<Self::MaterialIndex> {
//         Box::new(move |_chunk_pos, _lod, _previous| get_voxel_fn())
//     }
// }
//
// pub fn setup(mut commands: Commands) {
//     let light_color = Color::srgb(0.98, 0.85, 0.72);
//
//     // Sun
//     let cascade_shadow_config = CascadeShadowConfigBuilder {
//         maximum_distance: 7000.0,
//         ..default()
//     }
//     .build();
//
//     commands.spawn((
//         DirectionalLight {
//             color: light_color.darker(0.3),
//             shadow_maps_enabled: true,
//             ..default()
//         },
//         Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.1, 0.15), Vec3::Y),
//         cascade_shadow_config,
//     ));
//
//     // // Ambient light, same color as sun
//     commands.insert_resource(GlobalAmbientLight {
//         color: light_color,
//         brightness: 0.4,
//         affects_lightmapped_meshes: true,
//     });
// }
//
// fn get_voxel_fn() -> Box<dyn FnMut(IVec3, Option<WorldVoxel>) -> WorldVoxel + Send + Sync> {
//     // Set up some noise to use as the terrain height map
//     let mut noise = HybridMulti::<Perlin>::new(1234);
//     noise.octaves = 5;
//     noise.frequency = 1.1;
//     noise.lacunarity = 2.8;
//     noise.persistence = 0.4;
//
//     // We use this to cache the noise value for each y column so we only need
//     // to calculate it once per x/z coordinate
//     let mut cache = HashMap::<(i32, i32), f64>::new();
//
//     // Then we return this boxed closure that captures the noise and the cache
//     // This will get sent off to a separate thread for meshing by bevy_voxel_world
//     Box::new(move |pos: IVec3, _previous| {
//         // Sea level
//         if pos.y < 1 {
//             return WorldVoxel::Solid(3);
//         }
//
//         let [x, y, z] = pos.as_dvec3().to_array();
//
//         // If y is less than the noise sample, we will set the voxel to solid
//         let is_ground = y < match cache.get(&(pos.x, pos.z)) {
//             Some(sample) => *sample,
//             None => {
//                 let sample = noise.get([x / 1000.0, z / 1000.0]) * 50.0;
//                 cache.insert((pos.x, pos.z), sample);
//                 sample
//             }
//         };
//
//         if is_ground {
//             // Solid voxel of material type 0
//             WorldVoxel::Solid(0)
//         } else {
//             WorldVoxel::Air
//         }
//     })
// }
