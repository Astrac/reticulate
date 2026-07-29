struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn main() -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    return out;
}
