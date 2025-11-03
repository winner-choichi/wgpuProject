struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) weight: f32,
    @location(1) kind: u32,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) weight: f32,
    @location(2) kind: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    out.weight = weight;
    out.kind = kind;
    return out;
}

fn chapter_color(kind: u32, weight: f32, highlight: f32) -> vec3<f32> {
    switch kind {
        case 0u: { return vec3<f32>(0.35 + 0.45 * weight, 0.55, 1.0); }
        case 1u: { return vec3<f32>(0.75, 0.48 + 0.2 * weight, 1.0); }
        case 2u: { return vec3<f32>(0.35, 0.85, 1.0); }
        case 3u: { return vec3<f32>(0.45, 0.9, 0.85 + 0.1 * weight); }
        case 4u: { return vec3<f32>(1.0, 0.95, 0.85); }
        case 6u: { return vec3<f32>(0.55, 0.8, 1.0); }
        case 7u: { return vec3<f32>(1.0, 0.65 + 0.3 * highlight, 0.2); }
        case 8u: { return vec3<f32>(1.0, 0.9, 0.4); }
        default: { return vec3<f32>(0.5, 0.5, 0.5); }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let highlight = clamp(in.weight - 1.0, 0.0, 1.5);
    let base_weight = clamp(in.weight, 0.0, 1.0);
    let color = chapter_color(in.kind, base_weight, highlight);
    var alpha = clamp(base_weight * 0.6 + highlight * 0.5, 0.1, 1.0);
    if (in.kind == 3u) {
        alpha = clamp(0.25 + base_weight * 0.4, 0.1, 0.7);
    }
    if (in.kind == 4u) {
        alpha = 1.0;
    }
    if (in.kind == 6u) {
        alpha = clamp(0.25 + base_weight * 0.35, 0.15, 0.55);
    }
    if (in.kind == 7u) {
        alpha = clamp(0.55 + highlight * 0.45, 0.35, 1.0);
    }
    if (in.kind == 8u) {
        alpha = clamp(0.25 + base_weight * 0.5, 0.2, 0.9);
    }
    return vec4<f32>(color, alpha);
}
