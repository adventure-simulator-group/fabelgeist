//! A procedural ornament drawn into the engraving's recess.
//!
//! Each texel finds where it falls in its engraving cell, exactly as the
//! image resampling does, then measures its distance to the ornament's
//! strokes and shapes: a curve's distance is its vertical offset over the
//! curve's slope, which is exact enough for the stroke widths drawn. Coverage
//! is antialiased over one texel, so the lines are as sharp as the bake.
//!
//! Every intermediate is fenced (see [`fabelgeist_compute::host_float`]).

use super::wgsl;

pub(super) fn ornament_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> recess: array<f32>;

struct Params {{
    size: u32,
    motif: u32,
    repeats: u32,
    strands: u32,
    tiles: f32,
    sin_rotation: f32,
    cos_rotation: f32,
    line: f32,
    first: f32,
    second: f32,
    fillets: u32,
    zero: u32,
}};
@group(0) @binding(1) var<uniform> params: Params;

{math}

{shapes}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let index = id.x;
    let n = params.size;
    if (index >= n * n) {{
        return;
    }}
    let size = f32(n);
    let px = host_sub(host_div(f32(index % n) + 0.5, size), 0.5);
    let py = host_sub(host_div(f32(index / n) + 0.5, size), 0.5);
    let s = params.sin_rotation;
    let c = params.cos_rotation;
    let qx = host_add(host_mul(c, px), host_mul(s, py));
    let qy = host_add(host_mul(-s, px), host_mul(c, py));
    // Where the texel falls in its cell, as the image resampling finds it.
    let x = fraction(host_add(host_mul(qx, params.tiles), 0.5));
    let y = fraction(host_add(host_mul(qy, params.tiles), 0.5));
    let soft = host_div(params.tiles, size);
    let line = params.line;
    let half = host_mul(line, 0.5);
    // Fillets hug the cell's long sides; the motif keeps to the room between.
    var cut = 0.0;
    var margin = half;
    if (params.fillets != 0u) {{
        let low = stroke(abs(host_sub(y, line)), half, soft);
        let high = stroke(abs(host_sub(y, host_sub(1.0, line))), half, soft);
        cut = max(low, high);
        margin = host_mul(line, 2.5);
    }}
    let room = max(host_sub(host_sub(0.5, margin), half), 0.0);
    var motif = 0.0;
    switch params.motif {{
        case MOTIF_WAVE: {{
            motif = stroke(wave(x, y, host_mul(params.first, room), 0.0), half, soft);
        }}
        case MOTIF_ZIGZAG: {{ motif = zigzag(x, y, room, half, soft); }}
        case MOTIF_GUILLOCHE: {{ motif = guilloche(x, y, room, half, soft); }}
        case MOTIF_ROPE: {{ motif = rope(x, y, room, half, soft); }}
        case MOTIF_BEADS: {{ motif = beads(x, y, room, half, soft); }}
        case MOTIF_VINE: {{ motif = vine(x, y, room, half, soft); }}
        default: {{}}
    }}
    cut = max(cut, motif);
    recess[index] = cut;
}}
"#,
        math = wgsl::math(),
        shapes = SHAPES,
    )
}

/// The motif codes, the strokes and fills they are drawn with, and each
/// motif's coverage.
const SHAPES: &str = r#"
const TAU: f32 = 6.28318530717959;
const MOTIF_WAVE: u32 = 0u;
const MOTIF_ZIGZAG: u32 = 1u;
const MOTIF_GUILLOCHE: u32 = 2u;
const MOTIF_ROPE: u32 = 3u;
const MOTIF_BEADS: u32 = 4u;
const MOTIF_VINE: u32 = 5u;
// How far a rope groove bows, in repeats, between the band's middle and sides.
const ROPE_BOW: f32 = 0.3;

fn fraction(x: f32) -> f32 {
    return host_sub(x, floor(x));
}

// Coverage of a stroke of half width `half` at distance `d`, over `soft`.
fn stroke(d: f32, half: f32, soft: f32) -> f32 {
    return clamp(host_add(host_div(host_sub(half, d), soft), 0.5), 0.0, 1.0);
}

// Coverage of a shape at signed distance `d`, negative inside.
fn fill(d: f32, soft: f32) -> f32 {
    return clamp(host_sub(0.5, host_div(d, soft)), 0.0, 1.0);
}

// Distance to a curve through `value` with `slope` at the texel's x.
fn curve(y: f32, value: f32, slope: f32) -> f32 {
    return host_div(abs(host_sub(y, value)), host_sqrt(host_add(1.0, host_mul(slope, slope))));
}

// Distance to the wave `0.5 + amplitude sin(2 pi t)`, t = repeats x.
fn wave(x: f32, y: f32, amplitude: f32, phase: f32) -> f32 {
    let angle = host_mul(TAU, host_add(host_mul(x, f32(params.repeats)), phase));
    let turn = host_sin_cos(angle);
    let value = host_add(0.5, host_mul(amplitude, turn.x));
    let slope = host_mul(host_mul(host_mul(amplitude, TAU), f32(params.repeats)), turn.y);
    return curve(y, value, slope);
}

// Each motif's coverage at cell point (x, y), within `room` of the middle.
fn zigzag(x: f32, y: f32, room: f32, half: f32, soft: f32) -> f32 {
    let repeats = f32(params.repeats);
    let amplitude = host_mul(params.first, room);
    let u = fraction(host_add(host_mul(x, repeats), 0.25));
    let value = host_add(0.5, host_mul(amplitude, host_sub(host_mul(4.0, abs(host_sub(u, 0.5))), 1.0)));
    let slope = host_mul(host_mul(4.0, amplitude), repeats);
    return stroke(curve(y, value, slope), half, soft);
}

fn guilloche(x: f32, y: f32, room: f32, half: f32, soft: f32) -> f32 {
    let amplitude = host_mul(params.first, room);
    var nearest = 1.0;
    for (var k = 0u; k < params.strands; k = k + 1u) {
        let phase = host_div(f32(k), f32(params.strands));
        nearest = min(nearest, wave(x, y, amplitude, phase));
    }
    return stroke(nearest, half, soft);
}

// Grooves shift by `slant` repeats across the room's height, and bow like
// the lay of twisted strands.
fn rope(x: f32, y: f32, room: f32, half: f32, soft: f32) -> f32 {
    let period = host_div(1.0, f32(params.repeats));
    let height = max(host_mul(2.0, room), soft);
    let shift = host_div(params.first, height);
    let across = host_div(host_sub(y, 0.5), max(room, soft));
    let bow = host_mul(ROPE_BOW, host_sub(1.0, host_mul(across, across)));
    let t = host_mul(x, f32(params.repeats));
    let along = host_add(host_add(t, host_mul(shift, host_sub(y, 0.5))), bow);
    let offset = abs(host_sub(fraction(host_add(along, 0.5)), 0.5));
    // The groove's lean: its shift plus the bow's slope.
    let bend = host_div(host_mul(host_mul(-2.0, ROPE_BOW), across), max(room, soft));
    let lean = host_mul(host_add(shift, bend), period);
    let d = host_div(host_mul(offset, period), host_sqrt(host_add(1.0, host_mul(lean, lean))));
    let inside = fill(host_sub(abs(host_sub(y, 0.5)), host_add(room, half)), soft);
    return host_mul(stroke(d, half, soft), inside);
}

fn beads(x: f32, y: f32, room: f32, half: f32, soft: f32) -> f32 {
    let period = host_div(1.0, f32(params.repeats));
    let centre = host_mul(host_add(floor(host_mul(x, f32(params.repeats))), 0.5), period);
    let radius = host_mul(params.first, min(host_mul(0.5, period), host_add(room, half)));
    let dx = host_sub(x, centre);
    let dy = host_sub(y, 0.5);
    return fill(host_sub(host_sqrt(host_add(host_mul(dx, dx), host_mul(dy, dy))), radius), soft);
}

// A wave with a narrow upright leaf in each bay: beneath a crest, above a
// trough.
fn vine(x: f32, y: f32, room: f32, half: f32, soft: f32) -> f32 {
    let period = host_div(1.0, f32(params.repeats));
    let amplitude = host_mul(params.first, room);
    let stem = stroke(wave(x, y, amplitude, 0.0), half, soft);
    let bay = floor(host_mul(host_mul(x, f32(params.repeats)), 2.0));
    let centre = host_mul(host_mul(host_add(bay, 0.5), 0.5), period);
    var side = 1.0;
    if (fraction(host_mul(bay, 0.5)) > 0.25) {
        side = -1.0;
    }
    let across = max(host_mul(params.second, host_mul(0.12, period)), soft);
    let up = max(host_mul(params.second, host_mul(0.55, room)), soft);
    let leaf_y = host_sub(0.5, host_mul(side, host_mul(amplitude, 0.3)));
    let ex = host_div(host_sub(x, centre), across);
    let ey = host_div(host_sub(y, leaf_y), up);
    let reach = host_sub(host_sqrt(host_add(host_mul(ex, ex), host_mul(ey, ey))), 1.0);
    return max(stem, fill(host_mul(reach, min(across, up)), soft));
}
"#;
