//! The scalp cut on the GPU: [`crate::head::Head::scalp`], naive surface
//! nets over the head's field and the part above the hairline, in the
//! host's order -- the same triangles, the same vertex numbering, the same
//! running area -- so roots and guides land where the host's would.

use super::common::{HEAD, INDEX, MATH};
use super::guides::HAIRLINE;

fn kernel(bindings: &str, helpers: &[&str], body: &str) -> String {
    let mut source = String::from(bindings);
    for h in helpers {
        source.push_str(h);
    }
    source.push_str(INDEX);
    source.push_str(body);
    source
}

/// The grid: the corners `nx` by `ny` by `nz` from `lo` a `cell` apart, its
/// rows from `j0` of the whole bust's grid (corner `j` at `lo + (j0 + j) *
/// cell`, computed as the host computes it), and what is kept: above the
/// hairline and above `keep_y`.
const GRID_PARAMS: &str = r#"
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    cell: f32,
    nx: u32,
    ny: u32,
    nz: u32,
    j0: u32,
    hairline_front: f32,
    hairline_side: f32,
    hairline_back: f32,
    recession: f32,
    peak: f32,
    keep_y: f32,
    count: u32,
    pad0: u32,
"#;

const GRID: &str = r#"
fn corner_at(i: u32, j: u32, k: u32) -> vec3<f32> {
    return vec3<f32>(params.lo_x, params.lo_y, params.lo_z)
        + vec3<f32>(f32(i) * params.cell, f32(params.j0 + j) * params.cell, f32(k) * params.cell);
}

fn corner_index(i: u32, j: u32, k: u32) -> u32 {
    return (k * params.ny + j) * params.nx + i;
}

fn cell_index(i: u32, j: u32, k: u32) -> u32 {
    return (k * (params.ny - 1u) + j) * (params.nx - 1u) + i;
}
"#;

fn with_grid(bindings: &str) -> String {
    format!(
        "{bindings}struct Params {{{GRID_PARAMS}}};\n",
        bindings = bindings,
        GRID_PARAMS = GRID_PARAMS,
    )
}

/// The field at every corner.
pub fn nets_values() -> String {
    kernel(
        &(with_grid(
            r#"
@group(0) @binding(0) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> values: array<f32>;
"#,
        ) + "@group(0) @binding(2) var<uniform> params: Params;\n"),
        &[MATH, HEAD, GRID],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let c = item(id, groups);
    if (c >= params.count) {
        return;
    }
    let i = c % params.nx;
    let j = (c / params.nx) % params.ny;
    let k = c / (params.nx * params.ny);
    values[c] = head_distance(corner_at(i, j, k));
}
"#,
    )
}

/// A vertex in every cell the surface crosses: the mean of its edges'
/// crossings, two Newton steps onto the surface. `w` is 0 for none, 1 for
/// a vertex, 2 for one the scalp keeps.
pub fn nets_cells() -> String {
    kernel(
        &(with_grid(
            r#"
@group(0) @binding(0) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> values: array<f32>;
@group(0) @binding(2) var<storage, read_write> cells: array<vec4<f32>>;
"#,
        ) + "@group(0) @binding(3) var<uniform> params: Params;\n"),
        &[MATH, HEAD, GRID, &HAIRLINE.replace("hairdo.", "params.")],
        r#"
// The cube's twelve edges as corner pairs (corner `c` at `c & 1`,
// `c >> 1 & 1`, `c >> 2 & 1`), in the host's order.
fn edge_corner(e: u32, end: u32) -> u32 {
    switch e {
        case 0u: { return select(0u, 1u, end == 1u); }
        case 1u: { return select(2u, 3u, end == 1u); }
        case 2u: { return select(4u, 5u, end == 1u); }
        case 3u: { return select(6u, 7u, end == 1u); }
        case 4u: { return select(0u, 2u, end == 1u); }
        case 5u: { return select(1u, 3u, end == 1u); }
        case 6u: { return select(4u, 6u, end == 1u); }
        case 7u: { return select(5u, 7u, end == 1u); }
        case 8u: { return select(0u, 4u, end == 1u); }
        case 9u: { return select(1u, 5u, end == 1u); }
        case 10u: { return select(2u, 6u, end == 1u); }
        default: { return select(3u, 7u, end == 1u); }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let c = item(id, groups);
    let cx = params.nx - 1u;
    let cy = params.ny - 1u;
    if (c >= cx * cy * (params.nz - 1u)) {
        return;
    }
    let i = c % cx;
    let j = (c / cx) % cy;
    let k = c / (cx * cy);
    var sum = vec3<f32>(0.0);
    var count = 0u;
    for (var e = 0u; e < 12u; e++) {
        let a = edge_corner(e, 0u);
        let b = edge_corner(e, 1u);
        let ia = vec3<u32>(i + (a & 1u), j + ((a >> 1u) & 1u), k + ((a >> 2u) & 1u));
        let ib = vec3<u32>(i + (b & 1u), j + ((b >> 1u) & 1u), k + ((b >> 2u) & 1u));
        let va = values[corner_index(ia.x, ia.y, ia.z)];
        let vb = values[corner_index(ib.x, ib.y, ib.z)];
        if ((va < 0.0) != (vb < 0.0)) {
            let pa = corner_at(ia.x, ia.y, ia.z);
            let pb = corner_at(ib.x, ib.y, ib.z);
            sum += lerp3(pa, pb, va / (va - vb));
            count += 1u;
        }
    }
    if (count == 0u) {
        cells[c] = vec4<f32>(0.0);
        return;
    }
    var p = sum * (1.0 / f32(count));
    // Two Newton steps onto the surface, staying near the cell.
    for (var n = 0; n < 2; n++) {
        let e = params.cell * 0.05;
        let d = head_distance(p);
        let g = vec3<f32>(
            head_distance(p + vec3<f32>(e, 0.0, 0.0)) - head_distance(p - vec3<f32>(e, 0.0, 0.0)),
            head_distance(p + vec3<f32>(0.0, e, 0.0)) - head_distance(p - vec3<f32>(0.0, e, 0.0)),
            head_distance(p + vec3<f32>(0.0, 0.0, e)) - head_distance(p - vec3<f32>(0.0, 0.0, e)),
        ) * (0.5 / e);
        let g2 = dot(g, g);
        if (g2 > 1e-12) {
            let step = g * (d / g2);
            if (length(step) < params.cell) {
                p -= step;
            }
        }
    }
    let kept = above_hairline(p) > 0.0 && p.y > params.keep_y;
    cells[c] = vec4<f32>(p, select(1.0, 2.0, kept));
}
"#,
    )
}

/// The quads: one across every grid edge the surface cuts, joining the
/// four cells around it, split in two; each triangle the scalp keeps
/// (all its vertices kept) counted, or (`emit`) written at its offset.
pub fn nets_quads(emit: bool) -> String {
    let out = if emit {
        "@group(0) @binding(2) var<storage, read> offsets: array<u32>;\n\
         @group(0) @binding(3) var<storage, read_write> triangles: array<vec4<u32>>;\n"
    } else {
        "@group(0) @binding(2) var<storage, read_write> counts: array<u32>;\n"
    };
    let uniform = if emit { 4 } else { 3 };
    let body = r#"
// The cell `du`, `dw` back from corner `c` across the edge's two other
// axes, if it is in the grid and has a vertex: its index, or none.
fn around(c: vec3<u32>, u: u32, w: u32, du: u32, dw: u32) -> i32 {
    var at = vec3<i32>(c);
    at[u] -= i32(du);
    at[w] -= i32(dw);
    let n = vec3<i32>(i32(params.nx) - 1, i32(params.ny) - 1, i32(params.nz) - 1);
    if (any(at < vec3<i32>(0)) || any(at >= n)) {
        return -1;
    }
    let index = cell_index(u32(at.x), u32(at.y), u32(at.z));
    if (cells[index].w < 0.5) {
        return -1;
    }
    return i32(index);
}

fn kept(cell: i32) -> bool {
    return cells[u32(cell)].w > 1.5;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let e = item(id, groups);
    if (e >= params.count) {
        return;
    }
    let corner = e / 3u;
    let axis = e % 3u;
    let i = corner % params.nx;
    let j = (corner / params.nx) % params.ny;
    let k = corner / (params.nx * params.ny);
    var c1 = vec3<u32>(i, j, k);
    c1[axis] += 1u;
    var total = 0u;
    if (all(c1 < vec3<u32>(params.nx, params.ny, params.nz))) {
        let v0 = values_at(i, j, k);
        let v1 = values_at(c1.x, c1.y, c1.z);
        if ((v0 < 0.0) != (v1 < 0.0)) {
            var u = 1u;
            var w = 2u;
            if (axis == 1u) {
                u = 2u;
                w = 0u;
            } else if (axis == 2u) {
                u = 0u;
                w = 1u;
            }
            let c = vec3<u32>(i, j, k);
            let a = around(c, u, w, 0u, 0u);
            let b = around(c, u, w, 1u, 0u);
            let cc = around(c, u, w, 1u, 1u);
            let d = around(c, u, w, 0u, 1u);
            if (a >= 0 && b >= 0 && cc >= 0 && d >= 0) {
                // Inside to outside along the axis: counter-clockwise seen
                // from outside.
                var first = vec3<i32>(a, b, cc);
                var second = vec3<i32>(a, cc, d);
                if (v0 >= 0.0) {
                    first = vec3<i32>(a, cc, b);
                    second = vec3<i32>(a, d, cc);
                }
                let keep_first = kept(first.x) && kept(first.y) && kept(first.z);
                let keep_second = kept(second.x) && kept(second.y) && kept(second.z);
                EMIT
            }
        }
    }
    COUNT
}
"#;
    let (emit_code, count_code) = if emit {
        (
            r#"var at = offsets[e];
                if (keep_first) {
                    triangles[at] = vec4<u32>(vec3<u32>(first), 0u);
                    at += 1u;
                }
                if (keep_second) {
                    triangles[at] = vec4<u32>(vec3<u32>(second), 0u);
                }"#,
            "",
        )
    } else {
        (
            "total = u32(keep_first) + u32(keep_second);",
            "counts[e] = total;",
        )
    };
    let body = body.replace("EMIT", emit_code).replace("COUNT", count_code);
    kernel(
        &(with_grid(&format!(
            "@group(0) @binding(0) var<storage, read> values: array<f32>;\n\
             @group(0) @binding(1) var<storage, read> cells: array<vec4<f32>>;\n{out}"
        )) + &format!("@group(0) @binding({uniform}) var<uniform> params: Params;\n")),
        &[GRID],
        &(String::from(
            r#"
fn values_at(i: u32, j: u32, k: u32) -> f32 {
    return values[corner_index(i, j, k)];
}
"#,
        ) + &body),
    )
}

/// Where each kept vertex first appears among the triangles (its triangle
/// times three plus its corner), the least of them kept inverted in `first`
/// so a cleared buffer starts empty: the host numbers vertices so.
pub fn nets_first() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> triangles: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read_write> first: array<atomic<u32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(2) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let t = item(id, groups);
    if (t >= params.count) {
        return;
    }
    let tri = triangles[t];
    atomicMax(&first[tri.x], ~(t * 3u));
    atomicMax(&first[tri.y], ~(t * 3u + 1u));
    atomicMax(&first[tri.z], ~(t * 3u + 2u));
}
"#,
    )
}

/// Each vertex's cell at its first appearance, and a flag there to scan
/// into its number.
pub fn nets_order() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> first: array<u32>;
@group(0) @binding(1) var<storage, read_write> flags: array<u32>;
@group(0) @binding(2) var<storage, read_write> cell_at: array<u32>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(3) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let c = item(id, groups);
    if (c >= params.count || first[c] == 0u) {
        return;
    }
    let key = ~first[c];
    flags[key] = 1u;
    cell_at[key] = c;
}
"#,
    )
}

/// The scalp's vertices in the host's order: position, normal (the field's
/// gradient, as `Head::gradient`), UV (`Head::scalp_uv`); each cell's
/// vertex number; and the box they are in.
pub fn nets_vertices() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> cells: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> flags: array<u32>;
@group(0) @binding(3) var<storage, read> cell_at: array<u32>;
@group(0) @binding(4) var<storage, read_write> vertex_of: array<u32>;
@group(0) @binding(5) var<storage, read_write> scalp_points: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> scalp_normals: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> scalp_uvs: array<vec4<f32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(8) var<uniform> params: Params;
"#,
        &[MATH, HEAD],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let key = item(id, groups);
    if (key >= params.count || flags[key + 1u] == flags[key]) {
        return;
    }
    let v = flags[key];
    let cell = cell_at[key];
    let p = cells[cell].xyz;
    vertex_of[cell] = v;
    scalp_points[v] = vec4<f32>(p, 0.0);
    scalp_normals[v] = vec4<f32>(head_gradient(p), 0.0);
    // The polar projection about the crown.
    let q = p - head_shape[0].xyz;
    let r = acos(clamp(normalize(q).y, -1.0, 1.0)) / 2.6;
    let a = atan2(q.x, q.z);
    scalp_uvs[v] = vec4<f32>(0.5 + 0.5 * r * sin(a), 0.5 + 0.5 * r * cos(a), 0.0, 0.0);
}
"#,
    )
}

/// The triangles by vertex number, their areas, their edges as sort keys
/// (the lesser vertex's number above the greater's), and the box the
/// vertices are in, as ordered words ([`super::lod::bounds`]).
pub fn nets_triangles() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> triangles: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read> vertex_of: array<u32>;
@group(0) @binding(2) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> tri_index: array<vec4<u32>>;
@group(0) @binding(4) var<storage, read_write> areas: array<f32>;
@group(0) @binding(5) var<storage, read_write> edge_keys: array<u32>;
@group(0) @binding(6) var<storage, read_write> edge_ids: array<u32>;
@group(0) @binding(7) var<storage, read_write> box_words: array<atomic<u32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(8) var<uniform> params: Params;
"#,
        &[],
        r#"
fn ordered(x: f32) -> u32 {
    let bits = bitcast<u32>(x);
    return select(bits | 0x80000000u, ~bits, (bits & 0x80000000u) != 0u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let t = item(id, groups);
    if (t >= params.count) {
        return;
    }
    let cells = triangles[t];
    let v = vec3<u32>(vertex_of[cells.x], vertex_of[cells.y], vertex_of[cells.z]);
    tri_index[t] = vec4<u32>(v, 0u);
    let a = scalp_points[v.x].xyz;
    let b = scalp_points[v.y].xyz;
    let c = scalp_points[v.z].xyz;
    areas[t] = length(cross(b - a, c - a)) * 0.5;
    // The box: every vertex is some triangle's.
    for (var e = 0u; e < 3u; e++) {
        let p = scalp_points[v[e]].xyz;
        for (var k = 0u; k < 3u; k++) {
            let key = ordered(p[k]);
            atomicMax(&box_words[k], ~key);
            atomicMax(&box_words[3u + k], key);
        }
    }
    for (var e = 0u; e < 3u; e++) {
        let p = v[e];
        let q = v[(e + 1u) % 3u];
        edge_keys[t * 3u + e] = (min(p, q) << 16u) | max(p, q);
        edge_ids[t * 3u + e] = t * 3u + e;
    }
}
"#,
    )
}

/// The running area, one invocation adding in the host's order: which
/// triangle a share of the area falls in depends on every rounding.
pub fn nets_cdf() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> areas: array<f32>;
@group(0) @binding(1) var<storage, read_write> tri_cdf: array<f32>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(2) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(1)
fn main() {
    var total = 0.0;
    for (var t = 0u; t < params.count; t++) {
        total += areas[t];
        tri_cdf[t] = total;
    }
}
"#,
    )
}

/// Along the sorted edges, which are open (used by one triangle only) and
/// how many points each gets at two spacings (`steps` then `fine`), for
/// the scans to place them.
pub fn nets_open() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> edge_keys: array<u32>;
@group(0) @binding(1) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> steps: array<u32>;
@group(0) @binding(3) var<storage, read_write> fine: array<u32>;
struct Params {
    count: u32,
    pad0: u32,
    step: f32,
    fine_step: f32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count) {
        return;
    }
    let key = edge_keys[k];
    let open = (k == 0u || edge_keys[k - 1u] != key) && (k + 1u >= params.count || edge_keys[k + 1u] != key);
    steps[k] = 0u;
    fine[k] = 0u;
    if (!open) {
        return;
    }
    let length_ab = distance(scalp_points[key >> 16u].xyz, scalp_points[key & 0xFFFFu].xyz);
    steps[k] = u32(max(ceil(length_ab / max(params.step, 1e-5)), 1.0));
    fine[k] = u32(max(ceil(length_ab / max(params.fine_step, 1e-5)), 1.0));
}
"#,
    )
}

/// The points along the open edges, each edge's from its lesser vertex.
pub fn nets_boundary() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> edge_keys: array<u32>;
@group(0) @binding(1) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> boundary: array<vec4<f32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[MATH],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count) {
        return;
    }
    let first = offsets[k];
    let steps = offsets[k + 1u] - first;
    let key = edge_keys[k];
    let a = scalp_points[key >> 16u].xyz;
    let b = scalp_points[key & 0xFFFFu].xyz;
    for (var i = 0u; i < steps; i++) {
        boundary[first + i] = vec4<f32>(lerp3(a, b, f32(i) / f32(steps)), 0.0);
    }
}
"#,
    )
}
