struct BrushVertex {
    x: f32;
    y: f32;
    z: f32;
    cr: f32;
    cg: f32;
    cb: f32;
    score: f32;
    pad0_: u32;
    pad1_: u32;
};

struct vertexBuffer {
    brushVertices: [[stride(64)]] array<BrushVertex>;
};

struct ComputeInfo {
    num_objects: u32,
};

[[group(0), binding(2)]]
var<storage> global: vertexBuffer;
[[group(0), binding(3)]]
var<uniform> global_3: ComputeInfo;
var<private> gl_GlobalInvocationID: vec3<u32>;
[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

fn main_1() {
    var vertexIndex_2: u32;
    var result: BrushVertex;

    let _e11 = gl_GlobalInvocationID;
    vertexIndex_2 = _e11.x;
    let _e15 = vertexIndex_2;

    let _e16 = calcTangentBitangent(_e15);
    result = _e16;
    let _e18 = vertexIndex_2;
    let _e20 = result;
    global_1.dstVertices[_e18] = _e20;
    return;
}

[[stage(compute), workgroup_size(64, 1, 1)]]
fn main([[builtin(global_invocation_id)]] param: vec3<u32>) {
    gl_GlobalInvocationID = param;
    main_1();
    return;
}