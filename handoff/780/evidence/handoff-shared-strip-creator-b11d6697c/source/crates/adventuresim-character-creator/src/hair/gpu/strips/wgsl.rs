//! Native strip source rows, attributes and skin-clearance program.

pub(super) const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> normals: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> tangents: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> texcoords: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> lifts: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> head_shape: array<vec4<f32>>;

// One edge point of a strip's row.
struct StripPoint {
    position: vec3<f32>,
    normal: vec3<f32>,
    tangent: vec3<f32>,
    texcoord: vec2<f32>,
    occlusion: f32,
    random: f32,
};

fn lerp_point(a: StripPoint, b: StripPoint, t: f32) -> StripPoint {
    var p: StripPoint;
    p.position = a.position + (b.position - a.position) * t;
    p.normal = safe_normalize(mix(a.normal, b.normal, t), a.normal);
    p.tangent = safe_normalize(mix(a.tangent, b.tangent, t), a.tangent);
    p.texcoord = a.texcoord + (b.texcoord - a.texcoord) * t;
    p.occlusion = a.occlusion + (b.occlusion - a.occlusion) * t;
    p.random = a.random;
    return p;
}
"#;

/// Cards: the hair crate's card vertices, four entries each, a card's rows
/// in order, both edges of a row side by side.
pub(super) const CARD_ROWS: &str = r#"
@group(0) @binding(6) var<storage, read> card_vertices: array<vec4<f32>>;
struct Params {
    strips: u32,
    rows: u32,
    pieces: u32,
    spread: u32,
    clearance: f32,
    tightest_curve: f32,
    lie_flat_reach: f32,
    source_rows: u32,
    variants: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(7) var<uniform> params: Params;

fn source_point(strip: u32, row: u32, edge: u32) -> StripPoint {
    let v = ((strip * params.source_rows + row) * 2u + edge) * 4u;
    var p: StripPoint;
    p.position = card_vertices[v].xyz;
    p.tangent = card_vertices[v + 1u].xyz;
    p.normal = card_vertices[v + 2u].xyz;
    let variant = card_vertices[v + 2u].w;
    p.texcoord = vec2<f32>((variant + card_vertices[v].w) / params.variants, card_vertices[v + 1u].w);
    p.random = card_vertices[v + 3u].x;
    p.occlusion = card_vertices[v + 3u].y;
    return p;
}
"#;

/// Ribbons: a strand level's points, position and sky visibility, and each
/// strand's radius among its infos.
pub(super) const RIBBON_ROWS: &str = r#"
@group(0) @binding(6) var<storage, read> level_positions: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read> level_infos: array<vec4<f32>>;
@group(0) @binding(8) var<storage, read> taper: array<f32>;
struct Params {
    strips: u32,
    rows: u32,
    pieces: u32,
    spread: u32,
    clearance: f32,
    tightest_curve: f32,
    lie_flat_reach: f32,
    source_rows: u32,
    widen: f32,
    min_width: f32,
    taper_samples: u32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(9) var<uniform> params: Params;

fn taper_at(along: f32) -> f32 {
    let x = clamp(along, 0.0, 1.0) * f32(params.taper_samples - 1u);
    let i = min(u32(floor(x)), params.taper_samples - 2u);
    return mix(taper[i], taper[i + 1u], x - f32(i));
}

fn level_point(strip: u32, i: u32) -> vec3<f32> {
    return level_positions[strip * params.source_rows + i].xyz;
}

fn source_point(strip: u32, row: u32, edge: u32) -> StripPoint {
    let points = params.source_rows;
    let point = level_point(strip, row);
    let tangent = safe_normalize(
        level_point(strip, min(row + 1u, points - 1u)) - level_point(strip, select(row - 1u, 0u, row == 0u)),
        vec3<f32>(0.0, -1.0, 0.0),
    );
    let center = vec3<f32>(params.center_x, params.center_y, params.center_z);
    let side = safe_normalize(cross(tangent, point - center), vec3<f32>(1.0, 0.0, 0.0));
    let normal = safe_normalize(cross(side, tangent), vec3<f32>(0.0, 0.0, 1.0));
    let along = f32(row) / f32(max(points - 1u, 1u));
    let radius = level_infos[strip].x;
    let width = max(2.0 * radius * params.widen, params.min_width);
    let half = 0.5 * width * taper_at(along);
    var p: StripPoint;
    p.position = point + side * select(-half, half, edge == 1u);
    p.normal = normal;
    p.tangent = tangent;
    p.texcoord = vec2<f32>(f32(edge), along);
    p.occlusion = level_positions[strip * points + row].w;
    p.random = 0.0;
    return p;
}
"#;

/// The strip itself: cut, laid flat near the skin, written, then lifted off
/// it with the lift spread along the strip.
pub(super) const WRAP: &str = r#"
fn smooth01(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return c * c * (3.0 - 2.0 * c);
}

// Row `row` of the cut strip: between two of its source rows.
fn cut_point(strip: u32, row: u32, edge: u32) -> StripPoint {
    let segment = min(row / params.pieces, params.source_rows - 2u);
    let t = f32(row - segment * params.pieces) / f32(params.pieces);
    return lerp_point(source_point(strip, segment, edge), source_point(strip, segment + 1u, edge), t);
}

fn out_index(strip: u32, row: u32, edge: u32) -> u32 {
    return (strip * params.rows + row) * 2u + edge;
}

fn write_point(index: u32, p: StripPoint) {
    positions[index] = vec4<f32>(p.position, p.occlusion);
    normals[index] = vec4<f32>(p.normal, p.random);
    tangents[index] = vec4<f32>(p.tangent, 0.0);
    texcoords[index] = vec4<f32>(p.texcoord, 0.0, 0.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let strip = id.y * groups.x * 64u + id.x;
    if (strip >= params.strips) {
        return;
    }
    for (var row = 0u; row < params.rows; row++) {
        var left = cut_point(strip, row, 0u);
        var right = cut_point(strip, row, 1u);
        // Turned about its middle to lie flat on the skin near it: its width
        // square to both the strands and the skin's normal.
        let centre = (left.position + right.position) * 0.5;
        let weight = smooth01(1.0 - head_distance(centre) / params.lie_flat_reach);
        let across = right.position - left.position;
        let width = length(across);
        let flat_raw = cross(left.tangent, head_gradient(centre));
        if (weight > 0.0 && width > 1e-6 && length(flat_raw) > 1e-6) {
            let side = across / width;
            var flat = normalize(flat_raw);
            if (dot(flat, side) < 0.0) {
                flat = -flat;
            }
            let turned = normalize(mix(side, flat, weight));
            var face = normalize(cross(turned, left.tangent));
            if (dot(face, left.normal) < 0.0) {
                face = -face;
            }
            left.position = centre - turned * (0.5 * width);
            right.position = centre + turned * (0.5 * width);
            left.normal = face;
            right.normal = face;
        }
        write_point(out_index(strip, row, 0u), left);
        write_point(out_index(strip, row, 1u), right);
        // How far the row must move off the skin: whole, as far as its
        // deepest point must, by its clearance and its sag across its
        // width.
        let clearance = params.clearance + width * width / (8.0 * params.tightest_curve);
        var deepest = centre;
        var depth = head_distance(centre);
        for (var e = 0u; e < 2u; e++) {
            let point = select(left.position, right.position, e == 1u);
            let d = head_distance(point);
            if (d < depth) {
                depth = d;
                deepest = point;
            }
        }
        var need = vec3<f32>(0.0);
        if (depth < clearance) {
            need = push_out(deepest, clearance) - deepest;
        }
        lifts[strip * params.rows + row] = vec4<f32>(need, 0.0);
    }
    // Every row moves as far as the most any row near it asks, scaled by
    // how near.
    for (var row = 0u; row < params.rows; row++) {
        let first = select(row - params.spread, 0u, row < params.spread);
        let last = min(row + params.spread, params.rows - 1u);
        var lift = vec3<f32>(0.0);
        for (var other = first; other <= last; other++) {
            let apart = select(row - other, other - row, other > row);
            let taper = 1.0 - f32(apart) / f32(params.spread + 1u);
            let asked = lifts[strip * params.rows + other].xyz * taper;
            if (dot(asked, asked) > dot(lift, lift)) {
                lift = asked;
            }
        }
        for (var e = 0u; e < 2u; e++) {
            let index = out_index(strip, row, e);
            positions[index] = vec4<f32>(positions[index].xyz + lift, positions[index].w);
        }
    }
}
"#;
