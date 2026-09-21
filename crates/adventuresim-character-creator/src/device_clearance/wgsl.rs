//! The clearance fit in WGSL: which skin supports a plate, each station's
//! convex section of that skin, and every carrier moved to clear the section
//! blended from its two nearest stations.

use adventuresim_armor_model::gpu::wgsl;

use super::{SECTION_WORDS, STATION_CAPACITY, STATIONS};

pub(super) fn support_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(2) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(3) var<storage, read> primary: array<u32>;
@group(0) @binding(4) var<storage, read> extra: array<u32>;
@group(0) @binding(5) var<storage, read> frames: array<f32>;
@group(0) @binding(6) var<storage, read_write> support: array<u32>;
struct Params {{
    count: u32,
    filtered: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(7) var<uniform> params: Params;
{math}
{frame}
{positions}
{support_threshold}

fn weight(vertex: u32, owned: u32) -> f32 {{
    var sum = 0.0;
    for (var k = 0u; k < 8u; k = k + 1u) {{
        let joint = joint_indices[vertex * 8u + k];
        var owns = primary[joint];
        if (owned == 1u) {{
            owns = extra[joint];
        }}
        if (owns != 0u) {{
            sum = sum + joint_weights[vertex * 8u + k];
        }}
    }}
    return sum;
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.count) {{
        return;
    }}
    var lists = 0u;
    if (weight(i, 0u) >= SKIN_SUPPORT_THRESHOLD) {{
        lists = lists | 1u;
    }}
    if (weight(i, 1u) >= SKIN_SUPPORT_THRESHOLD) {{
        // Foot skin joins a greave's ankle only below the calf's depth.
        let fit = frame_at(0u);
        if (params.filtered == 0u || frame_local(fit, positions_at(i)).z <= fit.half_extents.z) {{
            lists = lists | 2u;
        }}
    }}
    support[i] = lists;
}}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        positions = wgsl::read_points("positions"),
        support_threshold = crate::armor_frames::skin_support_wgsl(),
    )
}

pub(super) fn fit_source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> support: array<u32>;
@group(0) @binding(2) var<storage, read> frames: array<f32>;
@group(0) @binding(3) var<storage, read_write> sections: array<f32>;
@group(0) @binding(4) var<storage, read_write> samples: array<f32>;
@group(0) @binding(5) var<storage, read_write> carriers: array<f32>;
@group(0) @binding(6) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    carriers_count: u32,
    cuff: u32,
    style: u32,
    hand: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    half_width: f32,
    gap: f32,
    pad3: f32,
    pad4: f32,
    style0: f32,
    style1: f32,
    style2: f32,
    style3: f32,
    style4: f32,
    style5: f32,
    style6: f32,
    style7: f32,
}};
@group(0) @binding(7) var<uniform> params: Params;
{math}
{frame}
{status_code}
{positions}
{carriers}

const STATIONS: u32 = {stations}u;
const CAPACITY: u32 = {capacity}u;
const SECTION_WORDS: u32 = {section_words}u;
const MINIMUM_SAMPLES: u32 = 16u;
const MINIMUM_RADIUS_M: f32 = 0.012;

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {{
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}}

fn local(f: Frame, p: vec3<f32>) -> vec3<f32> {{
    let d = p - f.origin;
    return vec3<f32>(host_dot(d, f.x), host_dot(d, f.y), host_dot(d, f.z));
}}

fn axial_span() -> vec2<f32> {{
    if (params.style == 0u) {{
        // A greave reaches below the ankle by its extension.
        return vec2<f32>(-1.0 - params.style5 / frame_at(0u).half_extents.y, 1.0);
    }}
    return vec2<f32>(params.style1, params.style2);
}}

fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 {{
    return a.x * b.y - a.y * b.x;
}}

{entry}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        status_code = wgsl::STATUS,
        positions = wgsl::read_points("positions"),
        carriers = wgsl::points("carriers"),
        stations = STATIONS,
        capacity = STATION_CAPACITY,
        section_words = SECTION_WORDS,
    )
}

/// One invocation per station: gather, sort, hull, then the section's radii.
pub(super) const SECTIONS: &str = r#"
fn sample(station: u32, i: u32) -> vec3<f32> {
    let at = (station * CAPACITY + i) * 3u;
    return vec3<f32>(samples[at], samples[at + 1u], samples[at + 2u]);
}

fn set_sample(station: u32, i: u32, p: vec3<f32>) {
    let at = (station * CAPACITY + i) * 3u;
    samples[at] = p.x;
    samples[at + 1u] = p.y;
    samples[at + 2u] = p.z;
}

// Lexicographic by x, then z.
fn before(a: vec3<f32>, b: vec3<f32>) -> bool {
    return a.x < b.x || (a.x == b.x && a.z < b.z);
}

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let station = group.x;
    let fit = frame_at(0u);
    let span = axial_span();
    let y = fit.half_extents.y * (span.x + (span.y - span.x) * f32(station) / f32(STATIONS - 1u));
    // Every support point within the band, once per support list naming it;
    // if the band holds too few, the nearest by height in list order.
    var band = 0u;
    for (var v = 0u; v < params.count; v = v + 1u) {
        let lists = support[v];
        if (lists != 0u && abs(local(fit, positions_at(v)).y - y) < params.half_width) {
            band = band + countOneBits(lists);
        }
    }
    var count = 0u;
    if (band >= MINIMUM_SAMPLES) {
        for (var v = 0u; v < params.count; v = v + 1u) {
            if (support[v] == 0u) {
                continue;
            }
            let p = local(fit, positions_at(v));
            if (abs(p.y - y) < params.half_width) {
                if (count >= CAPACITY) {
                    fail(STATUS_INVALID_SURFACE);
                    return;
                }
                set_sample(station, count, p);
                count = count + 1u;
            }
        }
    } else {
        // Take the MINIMUM_SAMPLES smallest (distance, list position) pairs.
        // List order is primary support in vertex order, then the extension
        // in vertex order; a vertex in both appears twice.
        var last_distance = -1.0;
        var last_key = 0u;
        loop {
            if (count >= MINIMUM_SAMPLES) {
                break;
            }
            var best_distance = 3.4e38;
            var best_key = 0xffffffffu;
            var best_point = vec3<f32>(0.0);
            for (var v = 0u; v < params.count; v = v + 1u) {
                let lists = support[v];
                if (lists == 0u) {
                    continue;
                }
                let p = local(fit, positions_at(v));
                let d = abs(p.y - y);
                for (var list = 0u; list < 2u; list = list + 1u) {
                    if ((lists & (1u << list)) == 0u) {
                        continue;
                    }
                    let key = list * params.count + v;
                    let after = d > last_distance || (d == last_distance && key > last_key);
                    let better = d < best_distance || (d == best_distance && key < best_key);
                    if (after && better) {
                        best_distance = d;
                        best_key = key;
                        best_point = p;
                    }
                }
            }
            if (best_key == 0xffffffffu) {
                break;
            }
            set_sample(station, count, best_point);
            count = count + 1u;
            last_distance = best_distance;
            last_key = best_key;
        }
    }
    // Insertion sort by (x, z), then drop exact repeats.
    for (var i = 1u; i < count; i = i + 1u) {
        let p = sample(station, i);
        var j = i;
        loop {
            if (j == 0u || !before(p, sample(station, j - 1u))) {
                break;
            }
            set_sample(station, j, sample(station, j - 1u));
            j = j - 1u;
        }
        set_sample(station, j, p);
    }
    var unique = 0u;
    for (var i = 0u; i < count; i = i + 1u) {
        let p = sample(station, i);
        if (unique > 0u) {
            let q = sample(station, unique - 1u);
            if (p.x == q.x && p.z == q.z) {
                continue;
            }
        }
        set_sample(station, unique, p);
        unique = unique + 1u;
    }
    // Monotone chain over (x, z), both halves. The hull is written after the
    // samples, which it never outgrows.
    var hull = 0u;
    let base = unique;
    for (var half = 0u; half < 2u; half = half + 1u) {
        let start = hull;
        for (var k = 0u; k < unique; k = k + 1u) {
            var index = k;
            if (half == 1u) {
                index = unique - 1u - k;
            }
            let p = sample(station, index).xz;
            loop {
                if (hull < start + 2u) {
                    break;
                }
                let last = sample(station, base + hull - 1u).xz;
                let previous = sample(station, base + hull - 2u).xz;
                if (cross2(last - previous, p - last) > 0.0) {
                    break;
                }
                hull = hull - 1u;
            }
            if (base + hull >= CAPACITY) {
                fail(STATUS_INVALID_SURFACE);
                return;
            }
            set_sample(station, base + hull, vec3<f32>(p.x, 0.0, p.y));
            hull = hull + 1u;
        }
        hull = hull - 1u;
    }
    var center = vec2<f32>(0.0);
    for (var i = 0u; i < hull; i = i + 1u) {
        center = center + sample(station, base + i).xz;
    }
    center = center / f32(hull);
    let out = station * SECTION_WORDS;
    sections[out] = center.x;
    sections[out + 1u] = center.y;
    for (var r = 0u; r < 64u; r = r + 1u) {
        let angle = f32(r) / 64.0 * TAU;
        let direction = vec2<f32>(cos(angle), sin(angle));
        var radius = MINIMUM_RADIUS_M;
        for (var i = 0u; i < hull; i = i + 1u) {
            let a = sample(station, base + i).xz - center;
            let edge = sample(station, base + (i + 1u) % hull).xz - sample(station, base + i).xz;
            let determinant = cross2(direction, edge);
            if (abs(determinant) < 1e-8) {
                continue;
            }
            let distance = cross2(a, edge) / determinant;
            let along = cross2(a, direction) / determinant;
            if (along >= 0.0 && along <= 1.0 && distance > radius) {
                radius = distance;
            }
        }
        sections[out + 2u + r] = radius;
    }
}
"#;

/// One invocation per carrier point: clear the blended section.
pub(super) const FIT: &str = r#"
fn section_radius(station: u32, direction: vec2<f32>) -> f32 {
    var turns = atan2(direction.y, direction.x) % TAU;
    if (turns < 0.0) {
        turns = turns + TAU;
    }
    let coordinate = turns / TAU * 64.0;
    let index = i32(floor(coordinate));
    let t = coordinate - floor(coordinate);
    let weights = array<f32, 4>(
        pow3(1.0 - t),
        3.0 * pow3(t) - 6.0 * t * t + 4.0,
        -3.0 * pow3(t) + 3.0 * t * t + 3.0 * t + 1.0,
        pow3(t),
    );
    var sum = 0.0;
    for (var offset = 0; offset < 4; offset = offset + 1) {
        let slot = u32(((index + offset - 1) % 64 + 64) % 64);
        sum = sum + weights[offset] * sections[station * SECTION_WORDS + 2u + slot] / 6.0;
    }
    return sum;
}

fn allowance(v: f32, direction: vec2<f32>, width: f32) -> f32 {
    switch params.style {
        case 0u: {
            let calf = params.style0;
            let ankle = pow2(max(1.0 - v / calf, 0.0));
            let knee = pow2(max((v - calf) / (1.0 - calf), 0.0));
            let belly = pow2(max(1.0 - abs((v - calf) / 0.35), 0.0));
            return width * (0.30 * (params.style3 - 0.45) * ankle
                + 0.20 * (params.style4 - 0.70) * knee
                + 0.06 * belly);
        }
        case 1u: {
            return width * 0.25 * (params.style0 - 0.55) * pow2(1.0 - v);
        }
        case 2u: {
            return width * (0.25 * (params.style0 - 0.60) * pow2(1.0 - v)
                + 0.35 * (params.style3 - 0.85) * pow2(direction.y));
        }
        default: {
            return 0.0;
        }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.carriers_count) {
        return;
    }
    let fit = frame_at(0u);
    var gap = params.gap;
    if (params.style == 3u && i < params.cuff) {
        gap = gap + params.style0;
    }
    var p = local(fit, carriers_at(i));
    let span = axial_span();
    let station = clamp(
        (p.y / fit.half_extents.y - span.x) / (span.y - span.x) * f32(STATIONS - 1u),
        0.0,
        f32(STATIONS - 1u),
    );
    let first = min(u32(floor(station)), STATIONS - 2u);
    let t = station - f32(first);
    let blend = t * t * (3.0 - 2.0 * t);
    let a = first * SECTION_WORDS;
    let b = (first + 1u) * SECTION_WORDS;
    let center = vec2<f32>(
        sections[a] + (sections[b] - sections[a]) * blend,
        sections[a + 1u] + (sections[b + 1u] - sections[a + 1u]) * blend,
    );
    // Keep the authored angular correspondence when the ankle section moves
    // toward the heel: re-centred rays can reverse adjacent columns.
    var radial = vec2<f32>(p.x - center.x, p.z - center.y);
    if (params.style == 0u) {
        radial = vec2<f32>(p.x, p.z);
    }
    let distance = sqrt(radial.x * radial.x + radial.y * radial.y);
    var direction = vec2<f32>(0.0, 1.0);
    if (distance > 1.1920929e-7) {
        direction = radial / distance;
    }
    let radius = section_radius(first, direction) * (1.0 - blend)
        + section_radius(first + 1u, direction) * blend + gap;
    let rho = distance / radius;
    let v = clamp((p.y / fit.half_extents.y + 1.0) * 0.5, 0.0, 1.0);
    var fitted = 1.0 + allowance(v, direction, fit.half_extents.x) / radius;
    if (params.style == 3u) {
        fitted = max(rho, 1.0);
    }
    if (rho > 1.1920929e-7) {
        if (params.hand != 0u) {
            let fingertip_end = 1.1;
            let strength = clamp(
                (p.y / fit.half_extents.y + fingertip_end) / (fingertip_end - 1.0),
                0.0,
                1.0,
            );
            let factor = 1.0 + (fitted / rho - 1.0) * strength;
            p.x = center.x + radial.x * factor;
            p.z = center.y + radial.y * factor;
        } else {
            p.x = center.x + radial.x * fitted / rho;
            p.z = center.y + radial.y * fitted / rho;
        }
    }
    carriers_set(i, frame_point(fit, p));
}
"#;
