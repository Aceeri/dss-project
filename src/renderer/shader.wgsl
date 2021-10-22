
[[block]]
struct CameraUniform {
    view_matrix: mat4x4<f32>;
    aspect_ratio: f32;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct InstanceInput {
    [[location(2)]] position: vec3<f32>;
    [[location(3)]] size: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_matrix * vec4<f32>(instance.position.x + (model.position.x * instance.size.x), -instance.position.y + (model.position.y * instance.size.y), instance.position.z, 1.0);
    return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn main(
    in: VertexOutput
) -> [[location(0)]] vec4<f32> {
    var sampled: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // If the color of this pixel is nothing or basically nothing, just discard it.
    // otherwise transparent pixels would overwrite other pixels when compared in the depth buffer
    // and you get a weird clipping with transparent images.
    if (sampled.a <= 0.0) {
        discard;
    }

    return sampled;
}
