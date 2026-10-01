@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> original: array<f32>;
@group(0) @binding(2) var<storage, read> references: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read> bounds: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read> coordinates: array<vec4<u32>>;
@group(0) @binding(6) var<storage, read_write> positions: array<f32>;
@group(0) @binding(7) var<storage, read_write> links: array<u32>;
@group(0) @binding(8) var<uniform> params: Params;

fn level(mid: u32, slope: f32) -> f32 {
    let p = local(original_at(references[mid].x));
    return p.y - slope * abs(p.x - lateral_origin());
}

fn row_up(row: u32) -> u32 {
    if (row < SKIRT_SAMPLES - 1u) { return V_SAMPLES + SKIRT_SAMPLES - 2u - row; }
    return row - (SKIRT_SAMPLES - 1u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let coordinate = coordinates[i];
    let side = coordinate.w & 1u;
    let outer = (coordinate.w & 0x80000000u) != 0u;
    let width = select(params.width, params.extra, side != 0u);
    let offset = select(0u, params.front_count, side != 0u);
    let column = coordinate.x;
    let t = bitcast<f32>(coordinate.y);
    let course = coordinate.z;
    let slope = design[3u + side];
    let floor = bounds[side].x;
    let height = bounds[side].y;
    let low = floor + height * f32(course) - select(design[2u], 0.0, course == 0u);
    let high = select(floor + height * f32(course + 1u),
        level(offset + (V_SAMPLES - 1u) * width + column, slope), course == u32(design[0u]));
    let q = mix(low, high, t);
    var a = offset + (V_SAMPLES - 1u) * width + column;
    var b = a;
    var blend = 0.0;
    for (var row = 0u; row < V_SAMPLES + SKIRT_SAMPLES - 2u; row += 1u) {
        let first = offset + row_up(row) * width + column;
        let second = offset + row_up(row + 1u) * width + column;
        let bottom = level(first, slope);
        let top = level(second, slope);
        if (q <= top && top > bottom) {
            a = first; b = second; blend = clamp((q - bottom) / (top - bottom), 0.0, 1.0); break;
        }
    }
    let ra = references[a]; let rb = references[b];
    let normal = normalize(mix(original_at(ra.y) - original_at(ra.x),
        original_at(rb.y) - original_at(rb.x), blend));
    let lift = select(design[5u] * (1.0 - t), 0.0, course == 0u);
    positions_set(i, mix(original_at(ra.x), original_at(rb.x), blend)
        + normal * (lift + select(0.0, wall_thickness(), outer)));
    links[i * 3u] = ra.x; links[i * 3u + 1u] = rb.x; links[i * 3u + 2u] = bitcast<u32>(blend);
}
