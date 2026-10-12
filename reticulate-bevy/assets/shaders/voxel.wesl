// assets/shaders/voxel.wesl  (Bevy 0.20)
//
// Per-voxel PBR for chunk meshes from the triangle mesher.
//   UV_0   = lattice-plane coordinates (floor = voxel cell, fract = position in cell)
//   UV_1.x = voxel material id (constant over each triangle)

import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
};

// Layout matches the Rust-side `VoxelPbr::pack`: three vec4s, 48 bytes per material.
struct VoxelPbr {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    // x = perceptual roughness, y = metallic, z = reflectance, w = unused
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(102)
var<storage, read> palette: array<VoxelPbr>;

// x = border width in voxel units, y = border darkening (0 = none, 1 = black)
@group(constants::MATERIAL_BIND_GROUP) @binding(103)
var<uniform> border: vec4<f32>;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Start from the StandardMaterial bindings, then override with per-voxel values.
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Material lookup. The id is constant across a triangle, so rounding removes
    // any interpolation noise.
    let last = arrayLength(&palette) - 1u;
    let id = min(u32(max(round(in.uv_b.x), 0.0)), last);
    let voxel = palette[id];

    // Voxel borders from the lattice-plane UV. Derivatives are taken before any discard.
    let cell_uv = fract(in.uv);
    let to_edge = min(
        min(cell_uv.x, 1.0 - cell_uv.x),
        min(cell_uv.y, 1.0 - cell_uv.y),
    );
    let footprint = max(fwidth(in.uv.x), fwidth(in.uv.y));
    // Fade borders out once a voxel covers only a few pixels, to avoid shimmer.
    let distance_fade = 1.0 - smoothstep(0.1, 0.5, footprint);
    let edge_mask = (1.0 - smoothstep(border.x - footprint, border.x + footprint, to_edge))
        * distance_fade;

    pbr_input.material.base_color = vec4(
        voxel.base_color.rgb * (1.0 - border.y * edge_mask),
        voxel.base_color.a,
    );
    pbr_input.material.emissive = vec4(voxel.emissive.rgb, 1.0);
    pbr_input.material.perceptual_roughness = voxel.params.x;
    pbr_input.material.metallic = voxel.params.y;
    pbr_input.material.reflectance = vec3(voxel.params.z);

    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
