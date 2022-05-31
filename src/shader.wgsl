// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] color: vec4<f32>;

};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var output_data: VertexOutput;
    output_data.tex_coords = model.tex_coords;
    output_data.color = model.color;
    output_data.clip_position = vec4<f32>(model.position, 1.0);
    return output_data;
}

// Fragment shader

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(orig_image_data: VertexOutput) -> [[location(0)]] vec4<f32> {
     //let data: vec4<f32> = textureSample(t_diffuse, s_diffuse, orig_image_data.tex_coords);
     let color: vec4<f32> = orig_image_data.color;
     //let transformed_data = vec4<f32>(data[0] - color[0] * color[3], data[1] - color[1] * color[3], data[2] - color[2] * color[3], data[3]);
     return color;
}