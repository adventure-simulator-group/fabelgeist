//! The surface finish of worked plate, baked per texel on the device.
//!
//! Hand-raised armor is never optically flat. Planishing leaves a gentle
//! undulation a few centimetres across, which makes reflections wobble;
//! polishing leaves a dense grain of hairlines along one direction, which
//! streaks highlights; and handling leaves the gloss uneven. Each is periodic
//! value noise on a lattice that divides the tile, so the maps still tile:
//! the undulation and grain become slopes, the smudges a roughness offset.
//!
//! Every lattice value is a draw of the metal's seeded sequence, and every
//! intermediate is fenced, so the result is the same on any device.

use super::wgsl;
use crate::material::Metal;
use fabelgeist_gpu::prelude::ShaderSource;

/// Floats written per texel: the slope along x and y, and a roughness offset.
pub(super) const FINISH_WORDS: u32 = 3;

/// Bake the finish of every texel.
pub(super) fn finish_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read_write> finish: array<f32>;

struct Params {{
    size: u32,
    seed: u32,
    zero: u32,
    pad0: u32,
    waviness: f32,
    grain: f32,
    smudge: f32,
    pad1: f32,
}};
@group(0) @binding(1) var<uniform> params: Params;

{math}
{random}

const TILES_PER_METRE: f32 = {tiles_per_metre:?};
// Undulation lattices per tile: planishing blows a few centimetres across.
const WAVE_CELLS: u32 = 4u;
const RIPPLE_CELLS: u32 = 9u;
// Polishing grain: long along the tile's x, a fraction of a millimetre across.
const GRAIN_ALONG: u32 = 3u;
const GRAIN_ACROSS: u32 = 400u;
// Slope and roughness of the grain at full strength.
const GRAIN_SLOPE: f32 = 0.03;
const GRAIN_ROUGHNESS: f32 = 0.12;
// Smudge lattices per tile.
const SMUDGE_CELLS: u32 = 3u;
const SPOT_CELLS: u32 = 7u;
// Keeps each layer's lattice values apart in the sequence.
const LAYER_STRIDE: u32 = 1048576u;

// A lattice value in [0, 1) of `layer`.
fn lattice(layer: u32, cells_x: u32, i: u32, j: u32) -> f32 {{
    return draw(lcg_skip(params.seed, layer * LAYER_STRIDE + j * cells_x + i + 1u));
}}

// Smooth value noise on a lattice of `cells` per tile, at tile coordinates
// in [0, 1): the value and its derivative with respect to the coordinates.
fn noise(layer: u32, cells_x: u32, cells_y: u32, p: vec2<f32>) -> vec3<f32> {{
    let gx = host_mul(p.x, f32(cells_x));
    let gy = host_mul(p.y, f32(cells_y));
    let ix = u32(floor(gx));
    let iy = u32(floor(gy));
    let fx = host_fence(gx - floor(gx));
    let fy = host_fence(gy - floor(gy));
    let i1 = (ix + 1u) % cells_x;
    let j1 = (iy + 1u) % cells_y;
    let a = lattice(layer, cells_x, ix % cells_x, iy % cells_y);
    let b = lattice(layer, cells_x, i1, iy % cells_y);
    let c = lattice(layer, cells_x, ix % cells_x, j1);
    let d = lattice(layer, cells_x, i1, j1);
    // Smoothstep weights and their derivatives.
    let sx = host_mul(host_mul(fx, fx), host_fence(3.0 - host_mul(2.0, fx)));
    let sy = host_mul(host_mul(fy, fy), host_fence(3.0 - host_mul(2.0, fy)));
    let dsx = host_mul(host_mul(6.0, fx), host_fence(1.0 - fx));
    let dsy = host_mul(host_mul(6.0, fy), host_fence(1.0 - fy));
    let ab = host_fence(b - a);
    let cd = host_fence(d - c);
    let top = host_fence(a + host_mul(ab, sx));
    let bottom = host_fence(c + host_mul(cd, sx));
    let value = host_fence(top + host_mul(host_fence(bottom - top), sy));
    let twist = host_fence(cd - ab);
    let along_x = host_mul(host_fence(ab + host_mul(twist, sy)), dsx);
    let along_y = host_mul(host_fence(bottom - top), dsy);
    return vec3<f32>(
        value,
        host_mul(along_x, f32(cells_x)),
        host_mul(along_y, f32(cells_y)),
    );
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let index = id.x;
    let n = params.size;
    if (index >= n * n) {{
        return;
    }}
    let size = f32(n);
    let p = vec2<f32>(
        host_div(f32(index % n) + 0.5, size),
        host_div(f32(index / n) + 0.5, size),
    );
    // Undulation height in metres; its slope per metre of surface is its
    // derivative per tile times the tiles per metre.
    let wave = noise(0u, WAVE_CELLS, WAVE_CELLS, p);
    let ripple = noise(1u, RIPPLE_CELLS, RIPPLE_CELLS, p);
    let per_metre = host_mul(params.waviness, TILES_PER_METRE);
    var dx = host_mul(host_fence(wave.y + host_mul(0.35, ripple.y)), per_metre);
    var dy = host_mul(host_fence(wave.z + host_mul(0.35, ripple.z)), per_metre);
    // The grain's slope is across its lines, relative to their pitch.
    let grain = noise(2u, GRAIN_ALONG, GRAIN_ACROSS, p);
    let grain_slope = host_mul(params.grain, GRAIN_SLOPE);
    dy = host_fence(dy + host_mul(host_div(grain.z, f32(GRAIN_ACROSS)), grain_slope));
    dx = host_fence(dx + host_mul(host_div(grain.y, f32(GRAIN_ACROSS)), grain_slope));
    let smudge = noise(3u, SMUDGE_CELLS, SMUDGE_CELLS, p);
    let spot = noise(4u, SPOT_CELLS, SPOT_CELLS, p);
    let blotch = host_fence(host_fence(smudge.x - 0.5) + host_mul(0.5, host_fence(spot.x - 0.5)));
    var roughness = host_mul(params.smudge, blotch);
    roughness = host_fence(roughness + host_mul(host_mul(params.grain, GRAIN_ROUGHNESS), host_fence(grain.x - 0.5)));
    finish[index * 3u] = dx;
    finish[index * 3u + 1u] = dy;
    finish[index * 3u + 2u] = roughness;
}}
"#,
        math = wgsl::math(),
        random = wgsl::RANDOM,
        tiles_per_metre = Metal::TILES_PER_METRE,
    ))
}
