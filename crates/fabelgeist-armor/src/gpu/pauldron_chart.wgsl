fn carrier_sample(u: f32, v: f32, start: u32) -> vec3<f32> {
    let uv = vec2<f32>(u * f32(CARRIER_COLUMNS), v * f32(CARRIER_ROWS));
    let cell = min(vec2<u32>(uv), vec2<u32>(CARRIER_COLUMNS - 1u, CARRIER_ROWS - 1u));
    let t = uv - vec2<f32>(cell);
    let index = cell.y * (CARRIER_COLUMNS + 1u) + cell.x;
    let stride = CARRIER_COLUMNS + 1u;
    var p: array<vec3<f32>, 4>;
    let offsets = array<u32, 4>(index, index + 1u, index + stride, index + stride + 1u);
    for (var k = 0u; k < 4u; k += 1u) {
        let at = start + offsets[k] * 3u;
        p[k] = vec3<f32>(frames[at], frames[at + 1u], frames[at + 2u]);
    }
    return mix(mix(p[0], p[1], t.x), mix(p[2], p[3], t.x), t.y);
}
fn chart_origin() -> vec3<f32> { return vec3<f32>(0.0); }
fn chart_offset(u: f32, axial: f32) -> f32 { return 0.0; }
fn chart_point(u: f32, v: f32) -> vec3<f32> {
    if (params.value3 >= 0.0) { return arm_lame(u, v, params.value3); }
    let along = mix(params.value0, params.value1, v);
    var p = carrier_sample(u, along, FITTED_START);
    if (params.value2 > 0.0) {
        let du = carrier_sample(min(u + 0.001, 1.0), along, FORMED_START)
            - carrier_sample(max(u - 0.001, 0.0), along, FORMED_START);
        let dv = carrier_sample(u, min(along + 0.001, 1.0), FORMED_START)
            - carrier_sample(u, max(along - 0.001, 0.0), FORMED_START);
        p += normalize(cross(du, dv)) * params.value2;
    }
    return p;
}
