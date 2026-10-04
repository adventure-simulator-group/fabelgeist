//! The chart kernel in WGSL: its bindings, the placement and relief helpers
//! every chart shares, and the pass that evaluates every carrier point.

use super::super::part::SHELL_MIRRORED;
use super::super::wgsl;
use super::{BOUNDARY_APEX, BOUNDARY_CAPPED, TIP_RINGS};

/// The chart kernel's bindings and parameters.
const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> columns: array<f32>;
@group(0) @binding(1) var<storage, read> flute: array<f32>;
@group(0) @binding(2) var<storage, read> design: array<f32>;
@group(0) @binding(3) var<storage, read> frames: array<f32>;
@group(0) @binding(4) var<storage, read_write> carriers: array<f32>;
@group(0) @binding(5) var<storage, read_write> heights: array<f32>;
@group(0) @binding(6) var<storage, read_write> shells: array<f32>;
struct Params {
    first: u32,
    shell: u32,
    stride: u32,
    rows: u32,
    boundary: u32,
    grid: u32,
    total: u32,
    fluted: u32,
    frame: u32,
    mirrored: u32,
    pad1: u32,
    pad2: u32,
    span0: f32,
    span1: f32,
    tip_length: f32,
    tip_per_half_height: f32,
    value0: f32,
    value1: f32,
    value2: f32,
    value3: f32,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    pad4: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    pad5: f32,
};
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(8) var<storage, read_write> status: array<atomic<u32>>;

fn invalid_chart_point() -> vec3<f32> {
    atomicOr(&status[0], 1u);
    return vec3<f32>(0.0);
}
"#;

/// Placing a chart's local points through its reflection and frame.
const HELPERS: &str = r#"
var<private> fit: Frame;

// A local point placed through the chart's own reflection, then the frame.
fn placed(local: vec3<f32>) -> vec3<f32> {
    var p = local;
    if (params.mirrored != 0u) {
        p.x = -p.x;
    }
    return frame_point(fit, p);
}

fn placed_vector(local: vec3<f32>) -> vec3<f32> {
    var v = local;
    if (params.mirrored != 0u) {
        v.x = -v.x;
    }
    return frame_vector(fit, v);
}

"#;

/// The fanned flute coordinate and the relief height of a chart point.
const RELIEF: &str = r#"
fn fanned(u: f32, axial: f32) -> f32 {
    if (params.fluted != 0u) {
        return flute_fan(u, axial);
    }
    return u;
}

fn height(u: f32, axial: f32) -> f32 {
    var relief = 0.0;
    if (params.fluted != 0u) {
        relief = flute_relief(u, axial);
    }
    return chart_offset(u, axial) + relief;
}

"#;

/// A shape whose extrusion origin is the one its host description gives.
pub(crate) const AUTHORED_ORIGIN: &str = r#"
fn chart_origin() -> vec3<f32> {
    return vec3<f32>(params.origin_x, params.origin_y, params.origin_z);
}
"#;

/// `PlateFluting::fan_coordinate` and `PlateFluting::relief`, reading the flute words.
pub(crate) const FLUTING: &str = r#"
fn flute_fan(u: f32, v: f32) -> f32 {
    let lateral = 2.0 * u - 1.0;
    let span = flute[3];
    let fan = flute[4] + (1.0 - flute[4]) * smoothstep_clamped(v);
    let absolute = abs(lateral);
    var mapped: f32;
    if (absolute <= span) {
        mapped = absolute * fan;
    } else {
        mapped = span * fan + (absolute - span) * (1.0 - span * fan) / (1.0 - span);
    }
    return (1.0 + sign(lateral) * mapped) * 0.5;
}

fn flute_relief(u: f32, v: f32) -> f32 {
    let count = flute[0];
    let pitch = flute[3] / count;
    let start = (1.0 - flute[3]) * 0.5;
    let slot = floor((u - start) / pitch);
    if (slot < 0.0 || slot >= count) {
        return 0.0;
    }
    let center = start + (slot + 0.5) * pitch;
    let distance = abs(u - center) / (pitch * flute[1] * 0.5);
    if (distance >= 1.0) {
        return 0.0;
    }
    let fade = smoothstep_clamped((v - flute[5]) / flute[7])
        * smoothstep_clamped((flute[6] - v) / flute[7]);
    return flute[2] * fade * (1.0 + cos(PI * distance)) * 0.5;
}
"#;

/// The chart kernel: every carrier point of one chart, evaluated by `shape`.
pub(super) fn source(shape: &str) -> String {
    format!(
        r#"
{BINDINGS}{math}
{frame}
{fluting}
{carriers}
const BOUNDARY_CAPPED: u32 = {capped}u;
const BOUNDARY_APEX: u32 = {apex}u;

{HELPERS}{shape}

{RELIEF}@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.total) {{
        return;
    }}
    fit = frame_at(params.frame);
    var point: vec3<f32>;
    var h: f32;
    if (i < params.grid) {{
        let row = i / params.stride;
        let v = f32(row) / f32(params.rows);
        let axial = lerp(params.span0, params.span1, v);
        let u = columns[i % params.stride];
        point = chart_point(fanned(u, axial), v);
        h = height(u, axial);
    }} else if (params.boundary == BOUNDARY_APEX) {{
        point = chart_point(0.5, 1.0);
        h = chart_offset(0.5, params.span1);
    }} else {{
        // A capped tip: rings below the first row, then the tip itself.
        let tip_length = fit.half_extents.y * params.tip_per_half_height + params.tip_length;
        let root_y = chart_point(fanned(columns[0], params.span0), 0.0).y;
        let j = i - params.grid;
        let rings = {rings}u - 1u;
        if (j < rings * params.stride) {{
            let ring = j / params.stride + 1u;
            let column = j % params.stride;
            let latitude = f32(ring) / {rings}.0 * FRAC_PI_2;
            let u = columns[column];
            let base = chart_point(fanned(u, params.span0), 0.0);
            point = vec3<f32>(
                base.x * cos(latitude),
                root_y - tip_length * sin(latitude),
                base.z * cos(latitude),
            );
            h = height(u, params.span0) * pow2(cos(latitude));
        }} else {{
            point = vec3<f32>(0.0, root_y - tip_length, 0.0);
            h = 0.0;
        }}
    }}
    carriers_set(params.first + i, placed(point));
    heights[params.first + i] = h;
    if (i == 0u) {{
        var origin = chart_origin();
        if (params.boundary == BOUNDARY_CAPPED) {{
            origin = vec3<f32>(0.0, chart_point(0.0, 0.0).y, 0.0);
        }}
        let axis = vec3<f32>(params.axis_x, params.axis_y, params.axis_z);
        let world_origin = placed(origin);
        let world_axis = placed_vector(axis);
        for (var k = 0u; k < 3u; k = k + 1u) {{
            shells[params.shell + 4u + k] = world_origin[k];
            shells[params.shell + 8u + k] = world_axis[k];
        }}
        // A shell's triangles are rewound when its frame reflects, and a local
        // mirror reflects once more.
        let reflects = dot(cross(fit.x, fit.y), fit.z) < 0.0;
        let mirrored = reflects != (params.mirrored != 0u);
        shells[params.shell + {mirrored_word}u] = bitcast<f32>(select(0u, 1u, mirrored));
    }}
}}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        fluting = FLUTING,
        carriers = wgsl::points("carriers"),
        capped = BOUNDARY_CAPPED,
        apex = BOUNDARY_APEX,
        rings = TIP_RINGS,
        mirrored_word = SHELL_MIRRORED,
    )
}
