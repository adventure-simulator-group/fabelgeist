@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> original: array<f32>;
@group(0) @binding(2) var<storage, read> references: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read> bounds: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> coordinates: array<vec4<u32>>;
@group(0) @binding(6) var<storage, read_write> positions: array<f32>;
@group(0) @binding(7) var<storage, read_write> links: array<u32>;
@group(0) @binding(8) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let coordinate = coordinates[i];
    let side = select(0u, 1u, (coordinate.w & 0x40000000u) != 0u);
    let outer = (coordinate.w & 0x80000000u) != 0u;
    let course = coordinate.w & 0x3fffffffu;
    let blend = bitcast<f32>(coordinate.z);
    let ra = references[coordinate.x]; let rb = references[coordinate.y];
    let base = mix(original_at(ra.x), original_at(rb.x), blend);
    // Preserve the clipped physical wall offsets. Renormalizing interpolated
    // offsets changes the original outer facets, particularly at cut edges.
    let gauge = mix(original_at(ra.y) - original_at(ra.x),
        original_at(rb.y) - original_at(rb.x), blend);
    // Transform both physical walls in the same construction coordinates.
    // Moving the outer wall by the inner wall's lift instead would shear the
    // metal thickness through a steep carrier facet.
    let point = base + select(vec3<f32>(0.0), gauge, outer);
    let p = local(point);
    let slope = design[3u + side];
    let q = p.y - slope * abs(p.x - lateral_origin());
    let floor = bounds[side].x;
    let pitch = bounds[side].y;
    let authored_low = floor + pitch * f32(course) - select(design[2u], 0.0, course == 0u);
    let lap_t = clamp((q - authored_low) / (pitch + design[2u]), 0.0, 1.0);
    let lift = select(design[5u] * (1.0 - lap_t), 0.0, course == 0u);
    let flare = vec3<f32>((p.x - lateral_origin()) / bounds[side].z * bounds[side].w, 0.0, select(1.0, -1.0, side != 0u));
    let moved = p + flare * lift;
    let rise = slope * (abs(moved.x - lateral_origin()) - abs(p.x - lateral_origin()));
    positions_set(i, point + world(flare * lift + vec3<f32>(0.0, rise, 0.0)));
    links[i * 3u] = ra.x; links[i * 3u + 1u] = rb.x; links[i * 3u + 2u] = coordinate.z;
}
