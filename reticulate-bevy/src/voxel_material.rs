use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{render_resource::AsBindGroup, storage::ShaderBuffer},
    shader::ShaderRef,
};

#[derive(Clone, Copy, Debug)]
pub struct VoxelPbr {
    pub base_color: LinearRgba,
    pub emissive: LinearRgba,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
}

impl Default for VoxelPbr {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            emissive: LinearRgba::BLACK,
            perceptual_roughness: 0.5,
            metallic: 0.0,
            reflectance: 0.5,
        }
    }
}

impl VoxelPbr {
    /// Matches `struct VoxelPbr` in voxel.wesl: three vec4s per material.
    pub fn pack(&self) -> [[f32; 4]; 3] {
        [
            self.base_color.to_f32_array(),
            self.emissive.to_f32_array(),
            [
                self.perceptual_roughness,
                self.metallic,
                self.reflectance,
                0.0,
            ],
        ]
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct VoxelExt {
    // Bindings 0-99 are reserved for the base StandardMaterial.
    #[storage(102, read_only)]
    pub palette: Handle<ShaderBuffer>,
    /// x = border width in voxel units (e.g. 0.04), y = border darkening 0..1.
    #[uniform(103)]
    pub border: Vec4,
}

impl MaterialExtension for VoxelExt {
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel.wesl".into()
    }
}

pub type VoxelMaterial = ExtendedMaterial<StandardMaterial, VoxelExt>;

#[derive(Resource)]
pub struct VoxelMaterialHandle(pub Handle<VoxelMaterial>);

pub fn setup_voxel_material(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let table = [
        VoxelPbr::default(), // 0: unused / fallback
        VoxelPbr {
            // 1: stone
            base_color: Color::srgb(0.45, 0.45, 0.47).to_linear(),
            perceptual_roughness: 0.9,
            ..default()
        },
        VoxelPbr {
            // 2: gold
            base_color: Color::srgb(0.9, 0.75, 0.2).to_linear(),
            perceptual_roughness: 0.3,
            metallic: 1.0,
            ..default()
        },
        VoxelPbr {
            // 3: lava (emissive)
            base_color: Color::srgb(0.1, 0.05, 0.05).to_linear(),
            emissive: LinearRgba::rgb(4.0, 1.5, 0.2),
            perceptual_roughness: 0.8,
            ..default()
        },
    ];

    let palette = buffers.add(ShaderBuffer::from(
        table.iter().map(VoxelPbr::pack).collect::<Vec<_>>(),
    ));

    let handle = materials.add(VoxelMaterial {
        // The base material's own values are overridden per voxel in the shader.
        // Keep it opaque (the default) unless you want Mask/Blend semantics.
        base: StandardMaterial::default(),
        extension: VoxelExt {
            palette,
            border: Vec4::new(0.04, 0.35, 0.0, 0.0),
        },
    });
    commands.insert_resource(VoxelMaterialHandle(handle));
}

// To change palette entries at runtime, mutate the buffer asset (as the official
// storage_buffer example does):
//   let buf = buffers.get_mut(&material.extension.palette).unwrap();
//   buf.clear();
//   buf.extend(table.iter().map(VoxelPbr::pack));
