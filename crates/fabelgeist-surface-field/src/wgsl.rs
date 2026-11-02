//! The field in WGSL: the shapes, the walk over a point's candidates, the
//! smooth union, and queries made of them.
//!
//! [`module`] is everything, to paste into a shader. The including shader
//! binds the index as read-only storage --
//!
//! ```wgsl
//! @group(0) @binding(5) var<storage, read> surface_shapes: array<SurfaceShape>;
//! @group(0) @binding(6) var<storage, read> surface_starts: array<u32>;
//! @group(0) @binding(7) var<storage, read> surface_entries: array<u32>;
//! ```
//!
//! -- and holds a `SurfaceParams` in a uniform. Every query is called as
//! `name(p, params, among, ...)`, `among` a `SurfaceFilter`: an object to
//! leave out (`SURFACE_NO_OBJECT` for none) and a layer mask. (`filter` is
//! a reserved word in WGSL.)
//!
//! * `surface_nearest(p, params, among, max) -> SurfaceHit`
//! * `surface_nearest4(p, params, among, max) -> SurfaceHits`
//! * `surface_union(p, params, among, k) -> f32`
//! * `surface_inside(p, params, among) -> SurfaceHit`
//!
//! and, on a hit, `surface_gradient(p, hit.shape)`, `surface_mix(own, hit, k)`
//! and `surface_lift(hit, k)`. Queries of one's own are written by [`query`].

/// The shapes, the candidate walk's pieces and the smooth union.
pub const CORE: &str = r#"
struct SurfaceShape {
    p0: vec4<f32>,
    p1: vec4<f32>,
    p2: vec4<f32>,
    object: u32,
    payload: u32,
    reach: f32,
    // The kind in the low byte, the layers above.
    mask: u32,
};

struct SurfaceParams {
    inverse_cell: f32,
    table_size: u32,
    pad0: u32,
    pad1: u32,
};

struct SurfaceFilter {
    exclude: u32,
    layers: u32,
};

struct SurfaceHit {
    // Signed distance; 1e30 when nothing was found.
    distance: f32,
    shape: u32,
    reach: f32,
    payload: u32,
};

struct SurfaceHits {
    hits: array<SurfaceHit, 4>,
    count: u32,
};

const SURFACE_EMPTY: u32 = 0xFFFFFFFFu;
const SURFACE_NO_OBJECT: u32 = 0xFFFFFFFFu;
const SURFACE_ALL_LAYERS: u32 = 0x00FFFFFFu;
const SURFACE_FAR: f32 = 1e30;

fn surface_filter_all() -> SurfaceFilter {
    return SurfaceFilter(SURFACE_NO_OBJECT, SURFACE_ALL_LAYERS);
}

fn surface_no_hit() -> SurfaceHit {
    return SurfaceHit(SURFACE_FAR, SURFACE_EMPTY, 0.0, 0u);
}

fn surface_found(hit: SurfaceHit) -> bool {
    return hit.shape != SURFACE_EMPTY;
}

fn surface_bucket(cell: vec3<i32>, table_size: u32) -> u32 {
    let h = (u32(cell.x) * 73856093u) ^ (u32(cell.y) * 19349663u) ^ (u32(cell.z) * 83492791u);
    return h & (table_size - 1u);
}

// The shapes that may be found from `p`: [begin, end) into
// `surface_entries`, which may hold SURFACE_EMPTY slots to skip.
fn surface_candidates(p: vec3<f32>, params: SurfaceParams) -> vec2<u32> {
    if (params.table_size == 0u) {
        return vec2<u32>(0u, 0u);
    }
    let bucket = surface_bucket(vec3<i32>(floor(p * params.inverse_cell)), params.table_size);
    return vec2<u32>(surface_starts[bucket], surface_starts[bucket + 1u]);
}

fn surface_passes(shape: SurfaceShape, among: SurfaceFilter) -> bool {
    return shape.object != among.exclude && ((shape.mask >> 8u) & among.layers) != 0u;
}

// --- Distances ---------------------------------------------------------

// Quilez's round cone.
fn surface_round_cone(p: vec3<f32>, a: vec4<f32>, b: vec4<f32>) -> f32 {
    let ba = b.xyz - a.xyz;
    let l2 = dot(ba, ba);
    if (l2 < 1e-12) {
        return length(p - a.xyz) - max(a.w, b.w);
    }
    let rr = a.w - b.w;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    let pa = p - a.xyz;
    let y = dot(pa, ba);
    let z = y - l2;
    let xv = pa * l2 - ba * y;
    let x2 = dot(xv, xv);
    let y2 = y * y * l2;
    let z2 = z * z * l2;
    let k = sign(rr) * rr * rr * x2;
    if (sign(z) * a2 * z2 > k) {
        return sqrt(x2 + z2) * il2 - b.w;
    }
    if (sign(y) * a2 * y2 < k) {
        return sqrt(x2 + y2) * il2 - a.w;
    }
    return (sqrt(x2 * a2 * il2) + y * rr) * il2 - a.w;
}

// A box about `center.xyz`, `half.xyz` in its own frame, turned by the unit
// quaternion `q`, its edges rounded by `center.w`.
fn surface_box(p: vec3<f32>, center: vec4<f32>, half: vec4<f32>, q: vec4<f32>) -> f32 {
    // Into the box's frame: the inverse rotation.
    let d = p - center.xyz;
    let t = 2.0 * cross(-q.xyz, d);
    let local = d + q.w * t + cross(-q.xyz, t);
    let r = center.w;
    let e = abs(local) - (half.xyz - r);
    return length(max(e, vec3<f32>(0.0))) + min(max(e.x, max(e.y, e.z)), 0.0) - r;
}

fn surface_dot2(v: vec3<f32>) -> f32 {
    return dot(v, v);
}

// Quilez's triangle, as a shell `a.w` either side.
fn surface_triangle(p: vec3<f32>, a: vec4<f32>, b: vec4<f32>, c: vec4<f32>) -> f32 {
    let ba = b.xyz - a.xyz;
    let pa = p - a.xyz;
    let cb = c.xyz - b.xyz;
    let pb = p - b.xyz;
    let ac = a.xyz - c.xyz;
    let pc = p - c.xyz;
    let nor = cross(ba, ac);
    let side = sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) + sign(dot(cross(ac, nor), pc));
    var squared: f32;
    if (side < 2.0) {
        squared = min(min(
            surface_dot2(ba * clamp(dot(ba, pa) / surface_dot2(ba), 0.0, 1.0) - pa),
            surface_dot2(cb * clamp(dot(cb, pb) / surface_dot2(cb), 0.0, 1.0) - pb)),
            surface_dot2(ac * clamp(dot(ac, pc) / surface_dot2(ac), 0.0, 1.0) - pc));
    } else {
        squared = dot(nor, pa) * dot(nor, pa) / surface_dot2(nor);
    }
    return sqrt(squared) - a.w;
}

// The ground under a bilinear patch over [x0, x1) x [z0, z1) (`footprint`),
// its corner heights `h`, down to `depth` below the lowest; nothing outside
// the footprint.
fn surface_height_tile(p: vec3<f32>, footprint: vec4<f32>, h: vec4<f32>, depth: f32) -> f32 {
    if (p.x < footprint.x || p.x >= footprint.z || p.z < footprint.y || p.z >= footprint.w) {
        return SURFACE_FAR;
    }
    let w = max(footprint.z - footprint.x, 1e-9);
    let d = max(footprint.w - footprint.y, 1e-9);
    let u = (p.x - footprint.x) / w;
    let v = (p.z - footprint.y) / d;
    let height = h.x * (1.0 - u) * (1.0 - v) + h.y * u * (1.0 - v) + h.z * (1.0 - u) * v + h.w * u * v;
    let gx = ((h.y - h.x) * (1.0 - v) + (h.w - h.z) * v) / w;
    let gz = ((h.z - h.x) * (1.0 - u) + (h.w - h.y) * u) / d;
    let top = (p.y - height) / sqrt(1.0 + gx * gx + gz * gz);
    let bottom = min(min(h.x, h.y), min(h.z, h.w)) - depth;
    return max(top, bottom - p.y);
}

// A shape's signed distance, negative inside.
fn surface_shape_distance(p: vec3<f32>, shape: SurfaceShape) -> f32 {
    switch (shape.mask & 0xFFu) {
        case 1u: { return surface_box(p, shape.p0, shape.p1, shape.p2); }
        case 2u: { return surface_triangle(p, shape.p0, shape.p1, shape.p2); }
        case 3u: { return surface_height_tile(p, shape.p0, shape.p1, shape.p2.x); }
        default: { return surface_round_cone(p, shape.p0, shape.p1); }
    }
}

fn surface_distance(p: vec3<f32>, index: u32) -> f32 {
    return surface_shape_distance(p, surface_shapes[index]);
}

// The outward normal of a shape's surface nearest `p`, by a tetrahedron of
// differences.
fn surface_gradient(p: vec3<f32>, index: u32) -> vec3<f32> {
    let shape = surface_shapes[index];
    let e = max(1e-4, 1e-3 * max(shape.reach, 1e-3));
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * surface_shape_distance(p + k.xyy * e, shape)
        + k.yyx * surface_shape_distance(p + k.yyx * e, shape)
        + k.yxy * surface_shape_distance(p + k.yxy * e, shape)
        + k.xxx * surface_shape_distance(p + k.xxx * e, shape)
    );
}

// --- The smooth union ----------------------------------------------------

fn surface_smin(a: f32, b: f32, k: f32) -> f32 {
    if (a >= SURFACE_FAR) {
        return b;
    }
    if (b >= SURFACE_FAR) {
        return a;
    }
    let kk = max(k, 1e-9);
    let h = clamp(0.5 + 0.5 * (b - a) / kk, 0.0, 1.0);
    return b + (a - b) * h - kk * h * (1.0 - h);
}

// How much of its own surface a point `own` from it is made of, against the
// hit surface, blending by the smaller of `k` and the hit's reach: 1 away
// from the hit, 1/2 where the two meet, 0 deep in it.
fn surface_mix(own: f32, hit: SurfaceHit, k: f32) -> f32 {
    if (!surface_found(hit)) {
        return 1.0;
    }
    let kk = max(min(hit.reach, k), 1e-9);
    return clamp(0.5 + 0.5 * (hit.distance - own) / kk, 0.0, 1.0);
}

// How far to move a point of a surface out along its normal onto its smooth
// union with the hit surface: the fillet. Deeper in the hit surface than
// the blend radius, a point is hidden and stays.
fn surface_lift(hit: SurfaceHit, k: f32) -> f32 {
    if (!surface_found(hit)) {
        return 0.0;
    }
    let kk = max(min(hit.reach, k), 1e-9);
    if (hit.distance <= -kk || hit.distance >= kk) {
        return 0.0;
    }
    return -surface_smin(0.0, hit.distance, kk);
}
"#;

/// A query of the caller's own: WGSL has no closures, so this writes the
/// walk over a point's candidates around a snippet.
///
/// The generated function is
/// `fn {name}(p: vec3<f32>, params: SurfaceParams, among: SurfaceFilter{parameters}) -> {result}`.
/// It starts `var acc: {result} = {init};`, then for each candidate the
/// filter passes and that is within its reach of `p` runs `visit`, which
/// sees `p`, `params`, `filter`, the extra parameters, `acc`, the shape's
/// index `i`, the `shape` itself and its `distance`; then returns `acc`.
#[derive(Clone, Copy, Debug)]
pub struct Query<'a> {
    pub name: &'a str,
    /// Extra parameters after the filter, e.g. `max: f32`; empty for none.
    pub parameters: &'a str,
    /// The accumulator's type, which is also the result's.
    pub result: &'a str,
    /// The accumulator's starting value.
    pub init: &'a str,
    /// Statements run per candidate.
    pub visit: &'a str,
}

/// The WGSL of a [`Query`].
pub fn query(q: &Query) -> String {
    let parameters = if q.parameters.trim().is_empty() {
        String::new()
    } else {
        format!(", {}", q.parameters)
    };
    format!(
        r#"
fn {name}(p: vec3<f32>, params: SurfaceParams, among: SurfaceFilter{parameters}) -> {result} {{
    var acc: {result} = {init};
    let range = surface_candidates(p, params);
    for (var entry = range.x; entry < range.y; entry++) {{
        let i = surface_entries[entry];
        if (i == SURFACE_EMPTY) {{
            continue;
        }}
        let shape = surface_shapes[i];
        if (!surface_passes(shape, among)) {{
            continue;
        }}
        let distance = surface_shape_distance(p, shape);
        if (distance >= shape.reach) {{
            continue;
        }}
        {visit}
    }}
    return acc;
}}
"#,
        name = q.name,
        result = q.result,
        init = q.init,
        visit = q.visit,
    )
}

/// The ready-made queries, each written by [`query`].
pub fn queries() -> [Query<'static>; 4] {
    [
        Query {
            name: "surface_nearest",
            parameters: "max: f32",
            result: "SurfaceHit",
            init: "surface_no_hit()",
            visit: "if (distance < max && distance < acc.distance) {
            acc = SurfaceHit(distance, i, shape.reach, shape.payload);
        }",
        },
        Query {
            name: "surface_nearest4",
            parameters: "max: f32",
            result: "SurfaceHits",
            init: "SurfaceHits(array<SurfaceHit, 4>(surface_no_hit(), surface_no_hit(), surface_no_hit(), surface_no_hit()), 0u)",
            // An insertion into four slots kept nearest first, ties by
            // index as the host's are.
            visit: "if (distance < max) {
            var hit = SurfaceHit(distance, i, shape.reach, shape.payload);
            for (var s = 0u; s < 4u; s++) {
                let there = acc.hits[s];
                if (hit.distance < there.distance || (hit.distance == there.distance && hit.shape < there.shape)) {
                    acc.hits[s] = hit;
                    hit = there;
                }
            }
            acc.count = min(acc.count + 1u, 4u);
        }",
        },
        Query {
            name: "surface_union",
            parameters: "k: f32",
            result: "f32",
            init: "SURFACE_FAR",
            visit: "acc = surface_smin(acc, distance, min(k, shape.reach));",
        },
        Query {
            name: "surface_inside",
            parameters: "",
            result: "SurfaceHit",
            init: "surface_no_hit()",
            visit: "if (distance < 0.0 && distance < acc.distance) {
            acc = SurfaceHit(distance, i, shape.reach, shape.payload);
        }",
        },
    ]
}

/// [`CORE`] and every ready-made query.
pub fn module() -> String {
    let mut source = CORE.to_string();
    for q in queries() {
        source.push_str(&query(&q));
    }
    source
}

#[cfg(test)]
mod tests {
    use super::*;

    const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> surface_shapes: array<SurfaceShape>;
@group(0) @binding(1) var<storage, read> surface_starts: array<u32>;
@group(0) @binding(2) var<storage, read> surface_entries: array<u32>;
@group(0) @binding(3) var<uniform> params: SurfaceParams;
@group(0) @binding(4) var<storage, read_write> out: array<f32>;
"#;

    fn validate(source: &str) {
        let module = match naga::front::wgsl::parse_str(source) {
            Ok(module) => module,
            Err(error) => panic!("{}", error.emit_to_string(source)),
        };
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("validates");
    }

    #[test]
    fn every_query_validates() {
        let source = format!(
            "{}{BINDINGS}
@compute @workgroup_size(1)
fn main() {{
    let p = vec3<f32>(0.0);
    let among = surface_filter_all();
    let hit = surface_nearest(p, params, among, 1.0);
    let four = surface_nearest4(p, params, among, 1.0);
    let inside = surface_inside(p, params, among);
    out[0] = surface_lift(hit, 1.0) + surface_mix(0.0, hit, 1.0) + surface_gradient(p, 0u).x
        + surface_union(p, params, among, 0.5) + four.hits[0].distance + inside.distance;
}}",
            module()
        );
        validate(&source);
    }

    #[test]
    fn a_query_of_ones_own_validates() {
        // How many surfaces of a layer lie within a distance.
        let count = query(&Query {
            name: "count_near",
            parameters: "within: f32",
            result: "u32",
            init: "0u",
            visit: "if (distance < within) { acc += 1u; }",
        });
        let source = format!(
            "{CORE}{count}{BINDINGS}
@compute @workgroup_size(1)
fn main() {{
    out[0] = f32(count_near(vec3<f32>(0.0), params, SurfaceFilter(SURFACE_NO_OBJECT, 1u), 0.5));
}}"
        );
        validate(&source);
    }
}
