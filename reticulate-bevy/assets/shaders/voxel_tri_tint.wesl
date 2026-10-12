// assets/shaders/voxel_tri_tint.wesl  (Bevy 0.20)
//
// Voxel PBR + per-triangle debug view (tint + edge lines). The vertex stage derives
// the triangle index and the corner (0, 1, 2) from @builtin(vertex_index),
// which is valid because the chunk mesh is non-indexed with three vertices per
// triangle. It forwards a flat triangle id and an interpolated barycentric
// vector (varyings live on-chip).
//
// Mesh attributes: position, normal, UV_0 (lattice-plane uv), UV_1.x (material id).

import bevy_pbr::render::{
    mesh_functions,
    forward_io::{FragmentOutput, Vertex, VertexOutput, decompress_vertex},
    mesh_bindings::mesh,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    view_transformations::position_world_to_clip
};

struct VoxelPbr {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    // x = perceptual roughness, y = metallic, z = reflectance, w = unused
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(102)
var<storage, read> palette: array<VoxelPbr>;

// x = voxel grid width (voxel units), y = voxel grid darkening 0..1,
// z = triangle tint strength 0..1, w = triangle edge line strength 0..1
@group(constants::MATERIAL_BIND_GROUP) @binding(103)
var<uniform> border: vec4<f32>;

// Triangle edge line width in pixels.
const WIRE_PX: f32 = 1.5;

// ---------------------------------------------------------------------------
// Vertex stage: a copy of Bevy's `vertex` (no skinning / morph targets, which
// chunk meshes don't use) with two extra outputs: tri_id and bary.
// ---------------------------------------------------------------------------

// Same fields and locations as bevy_pbr's VertexOutput, plus tri_id and bary.
struct TriVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @if(VERTEX_UVS_A)
    @location(2) uv: vec2<f32>,
    @if(VERTEX_UVS_B)
    @location(3) uv_b: vec2<f32>,
    @if(VERTEX_TANGENTS)
    @location(4) world_tangent: vec4<f32>,
    @if(VERTEX_COLORS)
    @location(5) color: vec4<f32>,
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    @interpolate(flat)
    @location(6) instance_index: u32,
    @if(VISIBILITY_RANGE_DITHER)
    @interpolate(flat)
    @location(7) visibility_range_dither: i32,
    @interpolate(flat)
    @location(8) tri_id: u32,
    @location(9) bary: vec3<f32>,
}

@vertex
fn vertex(vertex_in: Vertex, @builtin(vertex_index) vertex_index: u32) -> TriVertexOutput {
    var out: TriVertexOutput;
    let v = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);

    @if(VERTEX_NORMALS)
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
            v.normal,
            vertex_in.instance_index,
        );

    @if(VERTEX_POSITIONS)
    out.world_position = mesh_functions::mesh_position_local_to_world(
            world_from_local,
            vec4<f32>(v.position, 1.0),
        );
    @if(VERTEX_POSITIONS)
    out.position = position_world_to_clip(out.world_position.xyz);

    @if(VERTEX_UVS_A)
    out.uv = v.uv;
    @if(VERTEX_UVS_B)
    out.uv_b = v.uv_b;

    @if(VERTEX_TANGENTS)
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
            world_from_local,
            v.tangent,
            vertex_in.instance_index,
        );
    @if(VERTEX_COLORS)
    out.color = v.color;

    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = vertex_in.instance_index;
    @if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
            vertex_in.instance_index,
            world_from_local[3],
        );

    // Local vertex index within this mesh; three vertices per triangle.
    let first_vertex = mesh[vertex_in.instance_index].first_vertex_index;
    let local_index = vertex_index - first_vertex;
    let corner = local_index % 3u;
    out.tri_id = local_index / 3u;
    out.bary = vec3<f32>(f32(corner == 0u), f32(corner == 1u), f32(corner == 2u));

    return out;
}

// ---------------------------------------------------------------------------
// Fragment stage
// ---------------------------------------------------------------------------

// The extra varyings, received alongside Bevy's own VertexOutput.
struct TriExtra {
    @interpolate(flat)
    @location(8) tri_id: u32,
    @location(9) bary: vec3<f32>,
}

// Golden-ratio hue spread: consecutive triangle ids land ~0.618 apart on the
// hue wheel, so triangles emitted next to each other get very different colors.
// (Integer wraparound keeps this exact for any id.)
fn triangle_tint(id: u32) -> vec3<f32> {
    let h = id * 2654435769u;
    let hue = f32(h >> 8u) / 16777216.0;
    let rgb = clamp(
            abs(fract(vec3(hue) + vec3(0.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0) - 1.0,
            vec3(0.0),
            vec3(1.0),
        );
    // Slightly pastel so lighting and shading stay readable.
    return mix(vec3(1.0), rgb, 0.75);
}

@fragment
fn fragment(
    in: VertexOutput,
    extra: TriExtra,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let last = arrayLength(&palette) - 1u;
    let id = min(u32(max(round(in.uv_b.x), 0.0)), last);
    let voxel = palette[id];

    // Voxel grid lines (derivatives before any discard).
    let cell_uv = fract(in.uv);
    let to_edge = min(min(cell_uv.x, 1.0 - cell_uv.x), min(cell_uv.y, 1.0 - cell_uv.y));
    let footprint = max(fwidth(in.uv.x), fwidth(in.uv.y));
    let distance_fade = 1.0 - smoothstep(0.1, 0.5, footprint);
    let edge_mask = (1.0 - smoothstep(border.x - footprint, border.x + footprint, to_edge))
        * distance_fade;

    // Triangle edge lines: distance to the nearest edge in barycentric space,
    // converted to a roughly constant pixel width with fwidth.
    let bary_fw = fwidth(extra.bary);
    let wire_t = smoothstep(vec3(0.0), bary_fw * WIRE_PX, extra.bary);
    let wire = (1.0 - min(min(wire_t.x, wire_t.y), wire_t.z)) * border.w;

    // Triangle tint replaces the voxel color by `border.z`.
    let tinted = mix(voxel.base_color.rgb, triangle_tint(extra.tri_id), border.z);

    pbr_input.material.base_color = vec4(tinted * (1.0 - border.y * edge_mask), voxel.base_color.a);
    pbr_input.material.emissive = vec4(voxel.emissive.rgb, 1.0);
    pbr_input.material.perceptual_roughness = voxel.params.x;
    pbr_input.material.metallic = voxel.params.y;
    pbr_input.material.reflectance = vec3(voxel.params.z);

    pbr_input.material.base_color = alpha_discard(
            pbr_input.material.flags,
            pbr_input.material.alpha_cutoff,
            pbr_input.material.base_color,
        );

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    // Edge lines go on after lighting so they stay crisp.
    out.color = vec4(mix(out.color.rgb, vec3(0.0), wire), out.color.a);
    return out;
}
