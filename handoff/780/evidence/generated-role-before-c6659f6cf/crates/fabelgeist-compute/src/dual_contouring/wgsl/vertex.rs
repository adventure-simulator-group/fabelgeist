use super::PARAMS;

const VERTEX_STREAM: &str = r#"
struct Vertex {
    position: vec4<f32>,
    normal: vec4<f32>,
}
{params_struct}
@group(0) @binding(4) var<uniform> params: Params;
@group(0) @binding(5) var<storage, read_write> cell_vertex_indices: array<u32>;

fn get_sdf(x: i32, y: i32, z: i32) -> f32 {
    let ux = clamp(u32(x), 0u, params.grid_x - 1u);
    let uy = clamp(u32(y), 0u, params.grid_y - 1u);
    let uz = clamp(u32(z), 0u, params.grid_z - 1u);
    return textureLoad(input, vec3<u32>(ux, uy, uz), 0).x;
}

fn interpolate_sdf(d: vec3<f32>, v0: f32, v1: f32, v2: f32, v3: f32, v4: f32, v5: f32, v6: f32, v7: f32) -> f32 {
    let c01 = mix(v0, v1, d.x);
    let c32 = mix(v3, v2, d.x);
    let c45 = mix(v4, v5, d.x);
    let c76 = mix(v7, v6, d.x);

    let c0132 = mix(c01, c32, d.y);
    let c4576 = mix(c45, c76, d.y);

    return mix(c0132, c4576, d.z);
}

fn get_sdf_interpolated(p: vec3<f32>) -> f32 {
    let base = vec3<u32>(floor(p));
    let d = p - vec3<f32>(base);
    let v0 = textureLoad(input, base, 0).x;
    let v1 = textureLoad(input, base + vec3<u32>(1u, 0u, 0u), 0).x;
    let v2 = textureLoad(input, base + vec3<u32>(1u, 1u, 0u), 0).x;
    let v3 = textureLoad(input, base + vec3<u32>(0u, 1u, 0u), 0).x;
    let v4 = textureLoad(input, base + vec3<u32>(0u, 0u, 1u), 0).x;
    let v5 = textureLoad(input, base + vec3<u32>(1u, 0u, 1u), 0).x;
    let v6 = textureLoad(input, base + vec3<u32>(1u, 1u, 1u), 0).x;
    let v7 = textureLoad(input, base + vec3<u32>(0u, 1u, 1u), 0).x;
    return interpolate_sdf(d, v0, v1, v2, v3, v4, v5, v6, v7);
}

fn calculate_normal_at(p: vec3<f32>) -> vec3<f32> {
    let eps = 0.01;
    let nx = get_sdf_interpolated(p + vec3<f32>(eps, 0.0, 0.0)) - get_sdf_interpolated(p - vec3<f32>(eps, 0.0, 0.0));
    let ny = get_sdf_interpolated(p + vec3<f32>(0.0, eps, 0.0)) - get_sdf_interpolated(p - vec3<f32>(0.0, eps, 0.0));
    let nz = get_sdf_interpolated(p + vec3<f32>(0.0, 0.0, eps)) - get_sdf_interpolated(p - vec3<f32>(0.0, 0.0, eps));
    return normalize(vec3<f32>(nx, ny, nz));
}

fn stream(index: vec3<u32>, offset: u32, _res: ptr<storage, array<Vertex>, read_write>) {
    if (index.x == 0u || index.y == 0u || index.z == 0u || index.x >= params.grid_x - 2u || index.y >= params.grid_y - 2u || index.z >= params.grid_z - 2u) {
        return;
    }

    let c0 = vec3<i32>(index);
    let c1 = c0 + vec3<i32>(1, 0, 0);
    let c2 = c0 + vec3<i32>(1, 1, 0);
    let c3 = c0 + vec3<i32>(0, 1, 0);
    let c4 = c0 + vec3<i32>(0, 0, 1);
    let c5 = c0 + vec3<i32>(1, 0, 1);
    let c6 = c0 + vec3<i32>(1, 1, 1);
    let c7 = c0 + vec3<i32>(0, 1, 1);

    let v0 = get_sdf(c0.x, c0.y, c0.z);
    let v1 = get_sdf(c1.x, c1.y, c1.z);
    let v2 = get_sdf(c2.x, c2.y, c2.z);
    let v3 = get_sdf(c3.x, c3.y, c3.z);
    let v4 = get_sdf(c4.x, c4.y, c4.z);
    let v5 = get_sdf(c5.x, c5.y, c5.z);
    let v6 = get_sdf(c6.x, c6.y, c6.z);
    let v7 = get_sdf(c7.x, c7.y, c7.z);

    let s0 = v0 < params.threshold;
    let s1 = v1 < params.threshold;
    let s2 = v2 < params.threshold;
    let s3 = v3 < params.threshold;
    let s4 = v4 < params.threshold;
    let s5 = v5 < params.threshold;
    let s6 = v6 < params.threshold;
    let s7 = v7 < params.threshold;

    var sum_pos = vec3<f32>(0.0);
    var count_intersections = 0.0;

    if (s0 != s1) {
        let t = clamp((params.threshold - v0) / (v1 - v0), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c0), vec3<f32>(c1), t);
        count_intersections += 1.0;
    }
    if (s1 != s2) {
        let t = clamp((params.threshold - v1) / (v2 - v1), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c1), vec3<f32>(c2), t);
        count_intersections += 1.0;
    }
    if (s3 != s2) {
        let t = clamp((params.threshold - v3) / (v2 - v3), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c3), vec3<f32>(c2), t);
        count_intersections += 1.0;
    }
    if (s0 != s3) {
        let t = clamp((params.threshold - v0) / (v3 - v0), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c0), vec3<f32>(c3), t);
        count_intersections += 1.0;
    }
    if (s4 != s5) {
        let t = clamp((params.threshold - v4) / (v5 - v4), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c4), vec3<f32>(c5), t);
        count_intersections += 1.0;
    }
    if (s5 != s6) {
        let t = clamp((params.threshold - v5) / (v6 - v5), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c5), vec3<f32>(c6), t);
        count_intersections += 1.0;
    }
    if (s7 != s6) {
        let t = clamp((params.threshold - v7) / (v6 - v7), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c7), vec3<f32>(c6), t);
        count_intersections += 1.0;
    }
    if (s4 != s7) {
        let t = clamp((params.threshold - v4) / (v7 - v4), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c4), vec3<f32>(c7), t);
        count_intersections += 1.0;
    }
    if (s0 != s4) {
        let t = clamp((params.threshold - v0) / (v4 - v0), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c0), vec3<f32>(c4), t);
        count_intersections += 1.0;
    }
    if (s1 != s5) {
        let t = clamp((params.threshold - v1) / (v5 - v1), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c1), vec3<f32>(c5), t);
        count_intersections += 1.0;
    }
    if (s2 != s6) {
        let t = clamp((params.threshold - v2) / (v6 - v2), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c2), vec3<f32>(c6), t);
        count_intersections += 1.0;
    }
    if (s3 != s7) {
        let t = clamp((params.threshold - v3) / (v7 - v3), 0.0, 1.0);
        sum_pos += mix(vec3<f32>(c3), vec3<f32>(c7), t);
        count_intersections += 1.0;
    }

    if (count_intersections == 0.0) {
        return;
    }

    var v_pos = sum_pos / count_intersections;
    var p_curr = v_pos;
    for (var step = 0; step < 2; step += 1) {
        let val = get_sdf_interpolated(p_curr);
        let grad = calculate_normal_at(p_curr);
        let diff = val - params.threshold;
        p_curr = p_curr - diff * grad;
    }
    p_curr = clamp(p_curr, vec3<f32>(c0), vec3<f32>(c0) + vec3<f32>(1.0));
    v_pos = p_curr;

    let scale = vec3<f32>(params.scale_x, params.scale_y, params.scale_z);
    let pos_scaled = v_pos * scale + vec3<f32>(params.offset_x, params.offset_y, params.offset_z);
    let normal = normalize(calculate_normal_at(v_pos) / scale);

    if (offset < params.max_vertices) {
        output[offset] = Vertex(vec4<f32>(pos_scaled, 1.0), vec4<f32>(normal, 0.0));
        let grid_xy = params.grid_x * params.grid_y;
        let linear_index = index.z * grid_xy + index.y * params.grid_x + index.x;
        cell_vertex_indices[linear_index] = offset;
    }
}
"#;

pub(crate) fn vertex_stream() -> String {
    VERTEX_STREAM.replace("{params_struct}", PARAMS)
}
