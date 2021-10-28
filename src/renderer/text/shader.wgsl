
[[group(0), binding(1)]] var font_sampler: sampler;
[[group(0), binding(2)]] var font_texture: texture_2d<f32>;

[[block]]
struct CameraUniform {
    view_matrix: mat4x4<f32>;
    aspect_ratio: f32;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

// The vertices only get stepped per instance.
//
// This allows saving some data from being sent multiple times if we had
// multiple vertices. Instead this is more of a VertexInput per glyph.
struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] z: f32,
    [[location(1)]] left_top: vec2<f32>;
    [[location(2)]] right_bottom: vec2<f32>;
    [[location(3)]] texture_left_top: vec2<f32>;
    [[location(4)]] texture_right_bottom: vec2<f32>;
    [[location(5)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_matrix * vec4<f32>(instance.inst_position.x + (model.vert_position.x * instance.size.x), -instance.inst_position.y + (model.vert_position.y * instance.size.y), instance.inst_position.z, 1.0);
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
