use super::PARAMS;

pub(super) const QUAD_SUPPORT: &str = r#"
fn quad_vertex_ids(index: vec3<u32>, axis: u32) -> vec4<u32> {
    let grid_xy = params.grid_x * params.grid_y;
    let cell0 = index.z * grid_xy + index.y * params.grid_x + index.x;
    var cell1 = (index.z - 1u) * grid_xy + index.y * params.grid_x + index.x;
    var cell2 = (index.z - 1u) * grid_xy + (index.y - 1u) * params.grid_x + index.x;
    var cell3 = index.z * grid_xy + (index.y - 1u) * params.grid_x + index.x;
    if (axis == 1u) {
        cell1 = index.z * grid_xy + index.y * params.grid_x + (index.x - 1u);
        cell2 = (index.z - 1u) * grid_xy + index.y * params.grid_x + (index.x - 1u);
        cell3 = (index.z - 1u) * grid_xy + index.y * params.grid_x + index.x;
    } else if (axis == 2u) {
        cell1 = index.z * grid_xy + (index.y - 1u) * params.grid_x + index.x;
        cell2 = index.z * grid_xy + (index.y - 1u) * params.grid_x + (index.x - 1u);
        cell3 = index.z * grid_xy + index.y * params.grid_x + (index.x - 1u);
    }
    return vec4<u32>(cell_vertex_indices[cell0], cell_vertex_indices[cell1],
                     cell_vertex_indices[cell2], cell_vertex_indices[cell3]);
}

fn quad_is_written(vertices: vec4<u32>) -> bool {
    return all(vertices != vec4<u32>(0xFFFFFFFFu));
}
"#;

const INDEX_STREAM: &str = r#"
struct Vertex {
    position: vec4<f32>,
    normal: vec4<f32>,
}
{params_struct}
@group(0) @binding(4) var<uniform> params: Params;
@group(0) @binding(5) var<storage, read> cell_vertex_indices: array<u32>;
@group(0) @binding(6) var<storage, read> front_vertices: array<Vertex>;
{quad_support}

fn write_front_quad(offset: u32, a: u32, b: u32, c: u32, d: u32, forward: bool) {
    // SAFT-style candidate selection: for an advancing front, choose the
    // shorter of the two legal diagonals. Dual contouring keeps its historical
    // fixed diagonal when front_mode is disabled.
    let diagonal_ac = distance(front_vertices[a].position.xyz, front_vertices[c].position.xyz);
    let diagonal_bd = distance(front_vertices[b].position.xyz, front_vertices[d].position.xyz);
    let use_bd = params.front_mode != 0u && diagonal_bd < diagonal_ac;

    if (forward) {
        if (use_bd) {
            output[offset] = a; output[offset + 1u] = b; output[offset + 2u] = d;
            output[offset + 3u] = b; output[offset + 4u] = c; output[offset + 5u] = d;
        } else {
            output[offset] = a; output[offset + 1u] = b; output[offset + 2u] = c;
            output[offset + 3u] = a; output[offset + 4u] = c; output[offset + 5u] = d;
        }
    } else {
        if (use_bd) {
            output[offset] = a; output[offset + 1u] = d; output[offset + 2u] = b;
            output[offset + 3u] = b; output[offset + 4u] = d; output[offset + 5u] = c;
        } else {
            output[offset] = a; output[offset + 1u] = c; output[offset + 2u] = b;
            output[offset + 3u] = a; output[offset + 4u] = d; output[offset + 5u] = c;
        }
    }
}

fn get_sdf(x: i32, y: i32, z: i32) -> f32 {
    let ux = clamp(u32(x), 0u, params.grid_x - 1u);
    let uy = clamp(u32(y), 0u, params.grid_y - 1u);
    let uz = clamp(u32(z), 0u, params.grid_z - 1u);
    return textureLoad(input, vec3<u32>(ux, uy, uz), 0).x;
}

fn stream(index: vec3<u32>, offset: u32, _res: ptr<storage, array<u32>, read_write>) {
    if (index.x == 0u || index.y == 0u || index.z == 0u || index.x >= params.grid_x - 2u || index.y >= params.grid_y - 2u || index.z >= params.grid_z - 2u) {
        return;
    }

    let c0 = vec3<i32>(index);
    let v0 = get_sdf(c0.x, c0.y, c0.z);
    let v1 = get_sdf(c0.x + 1, c0.y, c0.z);
    let v3 = get_sdf(c0.x, c0.y + 1, c0.z);
    let v4 = get_sdf(c0.x, c0.y, c0.z + 1);

    let s0 = v0 < params.threshold;
    let s1 = v1 < params.threshold;
    let s3 = v3 < params.threshold;
    let s4 = v4 < params.threshold;

    var cur_offset = offset;

    // Edge X
    if (s0 != s1 && index.x >= 1u && index.x < params.grid_x - 2u && index.y >= 2u && index.y < params.grid_y - 2u && index.z >= 2u && index.z < params.grid_z - 2u) {
        let quad = quad_vertex_ids(index, 0u);
        let v0_idx = quad.x;
        let v1_idx = quad.y;
        let v2_idx = quad.z;
        let v3_idx = quad.w;

        if (quad_is_written(quad)) {
            if (cur_offset + 6u <= params.max_indices) {
                write_front_quad(cur_offset, v0_idx, v1_idx, v2_idx, v3_idx, v0 < v1);
            }
            cur_offset += 6u;
        }
    }

    // Edge Y
    if (s0 != s3 && index.x >= 2u && index.x < params.grid_x - 2u && index.y >= 1u && index.y < params.grid_y - 2u && index.z >= 2u && index.z < params.grid_z - 2u) {
        let quad = quad_vertex_ids(index, 1u);
        let v0_idx = quad.x;
        let v1_idx = quad.y;
        let v2_idx = quad.z;
        let v3_idx = quad.w;

        if (quad_is_written(quad)) {
            if (cur_offset + 6u <= params.max_indices) {
                write_front_quad(cur_offset, v0_idx, v1_idx, v2_idx, v3_idx, v0 < v3);
            }
            cur_offset += 6u;
        }
    }

    // Edge Z
    if (s0 != s4 && index.x >= 2u && index.x < params.grid_x - 2u && index.y >= 2u && index.y < params.grid_y - 2u && index.z >= 1u && index.z < params.grid_z - 2u) {
        let quad = quad_vertex_ids(index, 2u);
        let v0_idx = quad.x;
        let v1_idx = quad.y;
        let v2_idx = quad.z;
        let v3_idx = quad.w;

        if (quad_is_written(quad)) {
            if (cur_offset + 6u <= params.max_indices) {
                write_front_quad(cur_offset, v0_idx, v1_idx, v2_idx, v3_idx, v0 < v4);
            }
            cur_offset += 6u;
        }
    }
}
"#;

pub(crate) fn index_stream() -> String {
    INDEX_STREAM
        .replace("{params_struct}", PARAMS)
        .replace("{quad_support}", QUAD_SUPPORT)
}
