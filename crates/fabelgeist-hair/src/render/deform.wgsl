// Every drawn strand follows the simulated strands it is bound to: its rest
// pose carried by the head, plus the weighted displacement of those strands
// from their own rest poses at the same share of the way along -- and,
// where there is a body, kept off it as the simulated ones are: out to the
// offset, or as near as the groom has the point. The head's distance field
// (`head_distance`, `push_out`) is appended to this source.

struct Params {
    head: mat4x4<f32>,
    points: u32,
    sim_points: u32,
    strands: u32,
    body: u32,
    offset: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> deformed: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> guide_rest: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> guide_now: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> bind_ids: array<vec4<u32>>;
@group(0) @binding(5) var<storage, read> bind_weights: array<vec4<f32>>;
@group(0) @binding(6) var<uniform> params: Params;
@group(0) @binding(7) var<storage, read> head_shape: array<vec4<f32>>;

@compute @workgroup_size(64)
fn deform(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = id.x + id.y * groups.x * 64u;
    let total = params.points * params.strands;
    if (i >= total) {
        return;
    }
    let strand = i / params.points;
    let j = i % params.points;
    let along = f32(j) / f32(max(params.points, 2u) - 1u) * f32(params.sim_points - 1u);
    let a = min(u32(floor(along)), params.sim_points - 1u);
    let b = min(a + 1u, params.sim_points - 1u);
    let w = along - f32(a);
    let ids = bind_ids[strand];
    let weights = bind_weights[strand];
    var moved = vec3<f32>(0.0);
    for (var k = 0u; k < 4u; k = k + 1u) {
        let weight = weights[k];
        if (weight > 0.0) {
            let base = ids[k] * params.sim_points;
            let now = mix(guide_now[base + a].xyz, guide_now[base + b].xyz, w);
            let before = mix(guide_rest[base + a].xyz, guide_rest[base + b].xyz, w);
            let carried = (params.head * vec4<f32>(before, 1.0)).xyz;
            moved = moved + (now - carried) * weight;
        }
    }
    let own = rest[i];
    var p = (params.head * vec4<f32>(own.xyz, 1.0)).xyz + moved;
    if (params.body != 0u) {
        let turn = mat3x3<f32>(params.head[0].xyz, params.head[1].xyz, params.head[2].xyz);
        let local = transpose(turn) * (p - params.head[3].xyz);
        let allowed = min(params.offset, head_distance(own.xyz));
        if (head_distance(local) < allowed) {
            p = turn * push_out(local, allowed) + params.head[3].xyz;
        }
    }
    deformed[i] = vec4<f32>(p, own.w);
}
