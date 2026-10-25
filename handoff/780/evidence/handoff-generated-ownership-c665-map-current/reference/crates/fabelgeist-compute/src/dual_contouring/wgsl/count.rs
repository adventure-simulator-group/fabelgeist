use super::PARAMS;
use super::index::QUAD_SUPPORT;

const VERTEX_COUNT: &str = r#"
{params_struct}
@group(0) @binding(2) var<uniform> params: Params;

fn get_sdf(x: i32, y: i32, z: i32) -> f32 {
    let ux = clamp(u32(x), 0u, params.grid_x - 1u);
    let uy = clamp(u32(y), 0u, params.grid_y - 1u);
    let uz = clamp(u32(z), 0u, params.grid_z - 1u);
    return textureLoad(input, vec3<u32>(ux, uy, uz), 0).x;
}

fn gather(index: u32) -> u32 {
    let grid_xy = params.grid_x * params.grid_y;
    let z = index / grid_xy;
    let y = (index % grid_xy) / params.grid_x;
    let x = index % params.grid_x;
    let id0 = vec3<u32>(x, y, z);

    if (id0.x == 0u || id0.y == 0u || id0.z == 0u || id0.x >= params.grid_x - 2u || id0.y >= params.grid_y - 2u || id0.z >= params.grid_z - 2u) {
        return 0u;
    }

    let c0 = vec3<i32>(id0);
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

    let active_edges =
        (s0 != s1) || (s1 != s2) || (s2 != s3) || (s3 != s0) ||
        (s4 != s5) || (s5 != s6) || (s6 != s7) || (s7 != s4) ||
        (s0 != s4) || (s1 != s5) || (s2 != s6) || (s3 != s7);

    if (active_edges) {
        return 1u;
    }
    return 0u;
}
"#;

const INDEX_COUNT: &str = r#"
{params_struct}
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read> cell_vertex_indices: array<u32>;
{quad_support}

fn get_sdf(x: i32, y: i32, z: i32) -> f32 {
    let ux = clamp(u32(x), 0u, params.grid_x - 1u);
    let uy = clamp(u32(y), 0u, params.grid_y - 1u);
    let uz = clamp(u32(z), 0u, params.grid_z - 1u);
    return textureLoad(input, vec3<u32>(ux, uy, uz), 0).x;
}

fn gather(index: u32) -> u32 {
    let grid_xy = params.grid_x * params.grid_y;
    let z = index / grid_xy;
    let y = (index % grid_xy) / params.grid_x;
    let x = index % params.grid_x;
    let id0 = vec3<u32>(x, y, z);

    if (id0.x == 0u || id0.y == 0u || id0.z == 0u || id0.x >= params.grid_x - 2u || id0.y >= params.grid_y - 2u || id0.z >= params.grid_z - 2u) {
        return 0u;
    }

    let c0 = vec3<i32>(id0);
    let v0 = get_sdf(c0.x, c0.y, c0.z);
    let v1 = get_sdf(c0.x + 1, c0.y, c0.z);
    let v3 = get_sdf(c0.x, c0.y + 1, c0.z);
    let v4 = get_sdf(c0.x, c0.y, c0.z + 1);

    let s0 = v0 < params.threshold;
    let s1 = v1 < params.threshold;
    let s3 = v3 < params.threshold;
    let s4 = v4 < params.threshold;

    var count = 0u;
    // Edge X
    if (s0 != s1 && id0.x >= 1u && id0.x < params.grid_x - 2u && id0.y >= 2u && id0.y < params.grid_y - 2u && id0.z >= 2u && id0.z < params.grid_z - 2u) {
        if (quad_is_written(quad_vertex_ids(id0, 0u))) {
            count += 6u;
        }
    }
    // Edge Y
    if (s0 != s3 && id0.x >= 2u && id0.x < params.grid_x - 2u && id0.y >= 1u && id0.y < params.grid_y - 2u && id0.z >= 2u && id0.z < params.grid_z - 2u) {
        if (quad_is_written(quad_vertex_ids(id0, 1u))) {
            count += 6u;
        }
    }
    // Edge Z
    if (s0 != s4 && id0.x >= 2u && id0.x < params.grid_x - 2u && id0.y >= 2u && id0.y < params.grid_y - 2u && id0.z >= 1u && id0.z < params.grid_z - 2u) {
        if (quad_is_written(quad_vertex_ids(id0, 2u))) {
            count += 6u;
        }
    }
    return count;
}
"#;

pub(crate) fn vertex_count() -> String {
    VERTEX_COUNT.replace("{params_struct}", PARAMS)
}

pub(crate) fn index_count() -> String {
    INDEX_COUNT
        .replace("{params_struct}", PARAMS)
        .replace("{quad_support}", QUAD_SUPPORT)
}
