@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> faces: array<u32>;
@group(0) @binding(2) var<storage, read_write> normals: array<f32>;
@group(0) @binding(3) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(4) var<uniform> params: Params;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let a = positions_at(faces[i * 3u]);
    let b = positions_at(faces[i * 3u + 1u]);
    let c = positions_at(faces[i * 3u + 2u]);
    let n = unit(host_cross(host_sub3(b, a), host_sub3(c, a)));
    if (n.w == 0.0) { fail(STATUS_DEGENERATE); }
    normals_set(i, n.xyz);
}
