#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions,
}

struct Uniforms {
    grid_scale: f32,
    line_width: f32,
}
@group(3) @binding(75) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(
    input: VertexOutput,
    @builtin(front_facing) is_front: bool, // Fixed attribute for Bevy
) -> @location(0) vec4<f32> {
    // Grid
    let grid_color = vec4<f32>(0.0, 0.941, 1.0, 1.0);
    let uv: vec2<f32> = input.uv * uniforms.grid_scale; // Using your uniform scale
    let uvDDXY = vec4<f32>(dpdx(uv), dpdy(uv));
    let uvDeriv = vec2<f32>(length(uvDDXY.xz), length(uvDDXY.yw));

    let invertLine: bool = uniforms.line_width > 0.5;
    let targetWidth: f32 = select(uniforms.line_width, 1.0 - uniforms.line_width, invertLine);
    let drawWidth = clamp(vec2<f32>(targetWidth), uvDeriv, vec2<f32>(0.5));

    let lineAA = uvDeriv * 1.5;
    var gridUV = abs(fract(uv) * 2.0 - vec2<f32>(1.0));
    gridUV = select(1.0 - gridUV, gridUV, invertLine);

    var grid2 = smoothstep(drawWidth + lineAA, drawWidth - lineAA, gridUV);
    grid2 *= saturate(targetWidth / drawWidth);
    grid2 = mix(grid2, vec2<f32>(targetWidth), saturate(uvDeriv * 2.0 - vec2<f32>(1.0)));
    grid2 = select(grid2, vec2<f32>(1.0) - grid2, invertLine);

    let grid = mix(grid2.x, 1.0, grid2.y);

    // Rendering
    var pbr_input = pbr_input_from_standard_material(input, is_front);
    pbr_input.material.base_color = mix(pbr_input.material.base_color, grid_color, grid);
    let glow_intensity = 0.7;
    pbr_input.material.emissive = mix(
        pbr_input.material.emissive,
        grid_color * glow_intensity,
        grid
    );

    var out_color = pbr_functions::apply_pbr_lighting(pbr_input);
    out_color = pbr_functions::main_pass_post_lighting_processing(pbr_input, out_color);

    return out_color;
}

fn saturate(v: vec2<f32>) -> vec2<f32> {
    return clamp(v, vec2<f32>(0.0), vec2<f32>(1.0));
}
