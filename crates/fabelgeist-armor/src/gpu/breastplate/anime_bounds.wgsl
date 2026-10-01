@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> original: array<f32>;
@group(0) @binding(2) var<storage, read> references: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read_write> bounds: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let side = id.x;
    let width = select(params.width, params.extra, side != 0u);
    let offset = select(0u, params.front_count, side != 0u);
    let slope = design[3u + side];
    var floor = -1e30;
    var ceiling = 1e30;
    for (var c = 0u; c < width; c += 1u) {
        let lower = local(original_at(references[offset + (V_SAMPLES + SKIRT_SAMPLES - 2u) * width + c].x));
        let upper = local(original_at(references[offset + (V_SAMPLES - 1u) * width + c].x));
        floor = max(floor, lower.y - slope * abs(lower.x - lateral_origin()));
        ceiling = min(ceiling, upper.y - slope * abs(upper.x - lateral_origin()));
    }
    let height = (ceiling - floor) * design[1u] / design[0u];
    if (height <= design[2u] * 2.0) { fail(STATUS_INVALID_SURFACE); }
    bounds[side] = vec2<f32>(floor, height);
}
