
[[block]]
struct CameraUniform {
    view_matrix: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct InstanceInput {
    [[location(2)]] position: vec2<f32>;
    [[location(3)]] size: vec2<f32>;
    [[location(4)]] selected: bool;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] selected: bool;
    [[location(2)]] aspect: f32;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.selected = instance.selected;
    out.aspect = instance.size.x / instance.size.y;
    out.clip_position = camera.view_matrix * vec4<f32>(instance.position.x + (model.position.x * instance.size.x), instance.position.y + (model.position.y * instance.size.y), 0.0, 1.0);
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
    var border_width: f32;
    border_width = 0.05;
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}