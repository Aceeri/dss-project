
[[block]]
struct CameraUniform {
    view_matrix: mat4x4<f32>;
    scaling: f32;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] vert_position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct InstanceInput {
    [[location(2)]] inst_position: vec3<f32>;
    [[location(3)]] size: vec2<f32>;
    [[location(4)]] alpha: f32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] alpha: f32;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_matrix
        * vec4<f32>(instance.inst_position.x + (model.vert_position.x * instance.size.x), -instance.inst_position.y + (model.vert_position.y * instance.size.y), instance.inst_position.z, 1.0);
    out.alpha = instance.alpha;
    return out;
}

[[group(0), binding(0)]]
var sprite_texture: texture_2d<f32>;
[[group(0), binding(1)]]
var sprite_sampler: sampler;

[[stage(fragment)]]
fn main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    var sampled: vec4<f32> = textureSample(sprite_texture, sprite_sampler, input.tex_coords);
    sampled = sampled * vec4<f32>(1.0, 1.0, 1.0, input.alpha);

    if (sampled.a <= 0.0) {
        discard;
    }

    return sampled;
}
