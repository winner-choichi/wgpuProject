// This struct matches the CameraUniform struct in our Rust code.
// @group(0) and @binding(0) link this to our camera_bind_group.
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// This struct is the output of our vertex shader
// and the input to our fragment shader.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// --- VERTEX SHADER ---
// This function runs for every vertex in our mesh.
// @location(0) and @location(1) link these inputs to our Vertex::desc().
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
) -> VertexOutput {
    var out: VertexOutput;
    // Apply the camera's view-projection matrix to the vertex's position.
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    // Pass the vertex's color along to the fragment shader.
    out.color = color;
    return out;
}

// --- FRAGMENT SHADER ---
// This function runs for every pixel on the screen that our sphere covers.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return the color we received from the vertex shader,
    // with an alpha value of 1.0 (fully opaque).
    return vec4<f32>(in.color, 1.0);
}