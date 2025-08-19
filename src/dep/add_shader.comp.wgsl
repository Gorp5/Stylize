[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
[[group(0), binding(2)]]
var<storage> output: Buffer;
var<private> gl_GlobalInvocationID: vec3<u32>;

fn addPixels() {
    var rowIndex = &(output[local_invocation_id]);
    atomicAdd(outputPointer, pixelIndex);
    return;
}

[[stage(compute), workgroup_size(1080, 1, 1)]]
fn main([[builtin(global_invocation_id)]] param: vec3<u32>) {
    gl_GlobalInvocationID = param;
    addPixels();
    return;
}