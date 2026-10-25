//! Built-in marching-cubes native programs.

const PARAMS: &str = r#"
struct Params {
    grid_x: u32,
    grid_y: u32,
    grid_z: u32,
    max_vertices: u32,
    threshold: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
}
"#;

const COUNT: &str = r#"
{params_struct}
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read> tri_count_table: array<u32>;

fn gather(index: u32) -> u32 {
    let grid_xy = params.grid_x * params.grid_y;
    let z = index / grid_xy;
    let y = (index % grid_xy) / params.grid_x;
    let x = index % params.grid_x;
    let id0 = vec3<u32>(x, y, z);

    if (id0.x == 0u || id0.y == 0u || id0.z == 0u || id0.x >= params.grid_x - 2u || id0.y >= params.grid_y - 2u || id0.z >= params.grid_z - 2u) {
        return 0u;
    }

    let v0 = textureLoad(input, id0, 0).x;
    let v1 = textureLoad(input, id0 + vec3<u32>(1u, 0u, 0u), 0).x;
    let v2 = textureLoad(input, id0 + vec3<u32>(1u, 1u, 0u), 0).x;
    let v3 = textureLoad(input, id0 + vec3<u32>(0u, 1u, 0u), 0).x;
    let v4 = textureLoad(input, id0 + vec3<u32>(0u, 0u, 1u), 0).x;
    let v5 = textureLoad(input, id0 + vec3<u32>(1u, 0u, 1u), 0).x;
    let v6 = textureLoad(input, id0 + vec3<u32>(1u, 1u, 1u), 0).x;
    let v7 = textureLoad(input, id0 + vec3<u32>(0u, 1u, 1u), 0).x;

    var cubeIndex = 0u;
    if (v0 < params.threshold) { cubeIndex |= 1u; }
    if (v1 < params.threshold) { cubeIndex |= 2u; }
    if (v2 < params.threshold) { cubeIndex |= 4u; }
    if (v3 < params.threshold) { cubeIndex |= 8u; }
    if (v4 < params.threshold) { cubeIndex |= 16u; }
    if (v5 < params.threshold) { cubeIndex |= 32u; }
    if (v6 < params.threshold) { cubeIndex |= 64u; }
    if (v7 < params.threshold) { cubeIndex |= 128u; }

    return tri_count_table[cubeIndex];
}
"#;

const STREAM: &str = r#"
struct Vertex {
    position: vec4<f32>,
    normal: vec4<f32>,
}
{params_struct}
@group(0) @binding(4) var<uniform> params: Params;
@group(0) @binding(5) var<storage, read> triTable: array<i32>;
@group(0) @binding(6) var<storage, read> edgeTable: array<u32>;

fn get_sdf(x: i32, y: i32, z: i32) -> f32 {
    let ux = clamp(u32(x), 0u, params.grid_x - 1u);
    let uy = clamp(u32(y), 0u, params.grid_y - 1u);
    let uz = clamp(u32(z), 0u, params.grid_z - 1u);
    return textureLoad(input, vec3<u32>(ux, uy, uz), 0).x;
}

fn calculate_normal(x: u32, y: u32, z: u32) -> vec3<f32> {
    let ix = i32(x); let iy = i32(y); let iz = i32(z);
    let nx = get_sdf(ix + 1, iy, iz) - get_sdf(ix - 1, iy, iz);
    let ny = get_sdf(ix, iy + 1, iz) - get_sdf(ix, iy - 1, iz);
    let nz = get_sdf(ix, iy, iz + 1) - get_sdf(ix, iy, iz - 1);
    return normalize(vec3<f32>(nx, ny, nz));
}

fn interpolate(id1: vec3<u32>, id2: vec3<u32>, pos1: vec3<f32>, pos2: vec3<f32>) -> Vertex {
    let val1 = textureLoad(input, id1, 0).x;
    let val2 = textureLoad(input, id2, 0).x;
    let t = clamp((params.threshold - val1) / (val2 - val1), 0.0, 1.0);
    let n1 = calculate_normal(id1.x, id1.y, id1.z);
    let n2 = calculate_normal(id2.x, id2.y, id2.z);
    let scale = vec3<f32>(params.scale_x, params.scale_y, params.scale_z);
    let pos = mix(pos1, pos2, t) * scale + vec3<f32>(params.offset_x, params.offset_y, params.offset_z);
    let norm = normalize(mix(n1, n2, t) / scale);
    return Vertex(vec4<f32>(pos, 1.0), vec4<f32>(norm, 0.0));
}

fn stream(index: vec3<u32>, offset: u32, _res: ptr<storage, array<Vertex>, read_write>) {
    if (index.x == 0u || index.y == 0u || index.z == 0u || index.x >= params.grid_x - 2u || index.y >= params.grid_y - 2u || index.z >= params.grid_z - 2u) {
        return;
    }

    let id0 = index;
    let id1 = id0 + vec3<u32>(1u, 0u, 0u);
    let id2 = id0 + vec3<u32>(1u, 1u, 0u);
    let id3 = id0 + vec3<u32>(0u, 1u, 0u);
    let id4 = id0 + vec3<u32>(0u, 0u, 1u);
    let id5 = id0 + vec3<u32>(1u, 0u, 1u);
    let id6 = id0 + vec3<u32>(1u, 1u, 1u);
    let id7 = id0 + vec3<u32>(0u, 1u, 1u);

    let v0 = textureLoad(input, id0, 0).x;
    let v1 = textureLoad(input, id1, 0).x;
    let v2 = textureLoad(input, id2, 0).x;
    let v3 = textureLoad(input, id3, 0).x;
    let v4 = textureLoad(input, id4, 0).x;
    let v5 = textureLoad(input, id5, 0).x;
    let v6 = textureLoad(input, id6, 0).x;
    let v7 = textureLoad(input, id7, 0).x;

    var cubeIndex = 0u;
    if (v0 < params.threshold) { cubeIndex |= 1u; }
    if (v1 < params.threshold) { cubeIndex |= 2u; }
    if (v2 < params.threshold) { cubeIndex |= 4u; }
    if (v3 < params.threshold) { cubeIndex |= 8u; }
    if (v4 < params.threshold) { cubeIndex |= 16u; }
    if (v5 < params.threshold) { cubeIndex |= 32u; }
    if (v6 < params.threshold) { cubeIndex |= 64u; }
    if (v7 < params.threshold) { cubeIndex |= 128u; }

    let edgeState = edgeTable[cubeIndex];
    if (edgeState == 0u) { return; }

    let p0 = vec3<f32>(id0); let p1 = vec3<f32>(id1);
    let p2 = vec3<f32>(id2); let p3 = vec3<f32>(id3);
    let p4 = vec3<f32>(id4); let p5 = vec3<f32>(id5);
    let p6 = vec3<f32>(id6); let p7 = vec3<f32>(id7);

    var vertList: array<Vertex, 12>;
    if ((edgeState & 1u) != 0u) { vertList[0] = interpolate(id0, id1, p0, p1); }
    if ((edgeState & 2u) != 0u) { vertList[1] = interpolate(id1, id2, p1, p2); }
    if ((edgeState & 4u) != 0u) { vertList[2] = interpolate(id2, id3, p2, p3); }
    if ((edgeState & 8u) != 0u) { vertList[3] = interpolate(id3, id0, p3, p0); }
    if ((edgeState & 16u) != 0u) { vertList[4] = interpolate(id4, id5, p4, p5); }
    if ((edgeState & 32u) != 0u) { vertList[5] = interpolate(id5, id6, p5, p6); }
    if ((edgeState & 64u) != 0u) { vertList[6] = interpolate(id6, id7, p6, p7); }
    if ((edgeState & 128u) != 0u) { vertList[7] = interpolate(id7, id4, p7, p4); }
    if ((edgeState & 256u) != 0u) { vertList[8] = interpolate(id0, id4, p0, p4); }
    if ((edgeState & 512u) != 0u) { vertList[9] = interpolate(id1, id5, p1, p5); }
    if ((edgeState & 1024u) != 0u) { vertList[10] = interpolate(id2, id6, p2, p6); }
    if ((edgeState & 2048u) != 0u) { vertList[11] = interpolate(id3, id7, p3, p7); }

    var tableOffset = cubeIndex * 16u;
    for (var i = 0u; i < 16u; i += 3u) {
        let tri0 = triTable[tableOffset + i];
        if (tri0 == -1) { break; }

        let t0 = u32(tri0);
        let t1 = u32(triTable[tableOffset + i + 1u]);
        let t2 = u32(triTable[tableOffset + i + 2u]);

        if (offset + i + 3u <= params.max_vertices) {
            output[offset + i] = vertList[t0];
            output[offset + i + 1u] = vertList[t1];
            output[offset + i + 2u] = vertList[t2];
        }
    }
}
"#;

pub(crate) fn count() -> String {
    COUNT.replace("{params_struct}", PARAMS)
}

pub(crate) fn stream() -> String {
    STREAM.replace("{params_struct}", PARAMS)
}
