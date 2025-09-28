struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) weight: f32,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) weight: f32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    out.weight = clamp(weight, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let intensity = clamp(in.weight, 0.0, 1.0);
    let color = vec3<f32>(0.2 + 0.6 * intensity, 0.4, 1.0);
    return vec4<f32>(color, clamp(intensity * 0.9 + 0.1, 0.0, 1.0));
}
