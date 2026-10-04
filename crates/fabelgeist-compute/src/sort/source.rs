//! Exact native shader generation for the three radix stages.
use super::{RADIX, TILE};
use crate::prelude::*;
pub(super) fn histogram_code() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> histogram: array<u32>;

struct Params {{
    count: u32,
    shift: u32,
    tiles: u32,
    pad: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;

var<workgroup> local_histogram: array<atomic<u32>, {RADIX}u>;

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {{
    atomicStore(&local_histogram[local_id.x], 0u);
    workgroupBarrier();

    let index = global_id.x;
    if (index < params.count) {{
        let digit = (keys[index] >> params.shift) & {mask}u;
        atomicAdd(&local_histogram[digit], 1u);
    }}
    workgroupBarrier();

    // Digit-major: every tile's count for one digit is contiguous, which is
    // what lets the scan pass hand one whole row to one thread.
    histogram[local_id.x * params.tiles + group_id.x] =
        atomicLoad(&local_histogram[local_id.x]);
}}
"#,
        mask = RADIX - 1
    ))
}

pub(super) fn scan_code() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read_write> histogram: array<u32>;

struct Params {{
    count: u32,
    shift: u32,
    tiles: u32,
    pad: u32,
}};
@group(0) @binding(1) var<uniform> params: Params;

var<workgroup> digit_totals: array<u32, {RADIX}u>;
var<workgroup> scratch: array<u32, {RADIX}u>;

@compute @workgroup_size({RADIX}u)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {{
    let digit = local_id.x;
    let row = digit * params.tiles;

    // Exclusive scan along this digit's row, leaving each tile's offset
    // *within* the digit in place, and the digit's total in hand.
    var running = 0u;
    for (var tile = 0u; tile < params.tiles; tile = tile + 1u) {{
        let value = histogram[row + tile];
        histogram[row + tile] = running;
        running = running + value;
    }}
    digit_totals[digit] = running;
    workgroupBarrier();

    // Exclusive scan across the 256 digit totals: where each digit's block
    // starts in the output. Hillis-Steele, double-buffered through `scratch`
    // so no thread reads a slot another has already overwritten.
    var value = digit_totals[digit];
    for (var offset = 1u; offset < {RADIX}u; offset = offset * 2u) {{
        scratch[digit] = value;
        workgroupBarrier();
        if (digit >= offset) {{
            value = value + scratch[digit - offset];
        }}
        workgroupBarrier();
    }}
    // `value` is now the inclusive sum; the exclusive one drops this digit.
    let digit_base = value - digit_totals[digit];

    for (var tile = 0u; tile < params.tiles; tile = tile + 1u) {{
        histogram[row + tile] = histogram[row + tile] + digit_base;
    }}
}}
"#
    ))
}

pub(super) fn scatter_code() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> keys_in: array<u32>;
@group(0) @binding(1) var<storage, read> values_in: array<u32>;
@group(0) @binding(2) var<storage, read_write> keys_out: array<u32>;
@group(0) @binding(3) var<storage, read_write> values_out: array<u32>;
@group(0) @binding(4) var<storage, read> histogram: array<u32>;

struct Params {{
    count: u32,
    shift: u32,
    tiles: u32,
    pad: u32,
}};
@group(0) @binding(5) var<uniform> params: Params;

var<workgroup> tile_digits: array<u32, {TILE}u>;

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {{
    let index = global_id.x;
    let in_range = index < params.count;

    // Out-of-range lanes park on a digit no live lane can hold. They sit past
    // the end of the tile, so a live lane would never count them anyway; this
    // just keeps the shared array from carrying a digit that means something.
    var digit = {RADIX}u;
    var key = 0u;
    if (in_range) {{
        key = keys_in[index];
        digit = (key >> params.shift) & {mask}u;
    }}
    tile_digits[local_id.x] = digit;
    workgroupBarrier();

    if (in_range) {{
        // Rank among the same-digit elements earlier in this tile. Counting
        // rather than scanning keeps the sort stable, which is what makes the
        // whole least-significant-digit scheme work.
        var rank = 0u;
        for (var i = 0u; i < local_id.x; i = i + 1u) {{
            if (tile_digits[i] == digit) {{
                rank = rank + 1u;
            }}
        }}
        let destination = histogram[digit * params.tiles + group_id.x] + rank;
        keys_out[destination] = key;
        values_out[destination] = values_in[index];
    }}
}}
"#,
        mask = RADIX - 1
    ))
}
