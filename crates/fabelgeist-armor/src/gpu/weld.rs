//! The weld, as a deterministic device dedup.
//!
//! Vertices whose position, normal and texture coordinate agree to a
//! micrometre merge, numbered in order of first appearance, and the
//! triangles that collapse are dropped. Every unwelded vertex gets that
//! quantized key, prefixed by its part so parts never
//! merge; a hash of the key is sorted stably, and within a run of equal
//! hashes each vertex compares full keys to find the first vertex (lowest
//! slot) with its key -- its representative. A scan over the representatives
//! in slot order numbers them in first-appearance order, and a second scan
//! compacts the surviving triangles in order.

use super::geometry::{VALID_TRIANGLE, VERTEX_FLOATS};
use super::wgsl;

/// Words of a vertex key: its part, then eight quantized attributes.
pub(super) const KEY_WORDS: usize = 1 + VERTEX_FLOATS;

/// What every weld kernel knows of a triangle's info word and a key.
fn common() -> String {
    format!(
        r#"
const VALID: u32 = {valid}u;
const KEY_WORDS: u32 = {key_words}u;

fn is_valid(info: u32) -> bool {{
    return (info & VALID) != 0u;
}}
"#,
        valid = VALID_TRIANGLE,
        key_words = KEY_WORDS,
    )
}

/// Quantize every vertex and hash its key.
pub(super) fn keys_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> vertices: array<f32>;
@group(0) @binding(1) var<storage, read> triangle_info: array<u32>;
@group(0) @binding(2) var<storage, read_write> keys: array<u32>;
@group(0) @binding(3) var<storage, read_write> hashes: array<u32>;
@group(0) @binding(4) var<storage, read_write> slots: array<u32>;
{counted}
@group(0) @binding(5) var<uniform> params: Params;
{common}
const VERTEX_FLOATS: u32 = {vertex_floats}u;
// The weld resolution: attributes agreeing to a millionth merge.
const WELD_SCALE: f32 = 1000000.0;
// Hash of the slots that are not vertices; no vertex hashes to it.
const NO_VERTEX: u32 = 0xffffffffu;

// `(v * 1e6).round()`: halves round away from zero.
fn quantize(v: f32) -> u32 {{
    let scaled = v * WELD_SCALE;
    let whole = trunc(scaled);
    var rounded = whole;
    if (abs(scaled - whole) >= 0.5) {{
        rounded = whole + sign(scaled);
    }}
    return bitcast<u32>(i32(rounded));
}}

fn rotate(x: u32, bits: u32) -> u32 {{
    return (x << bits) | (x >> (32u - bits));
}}

// MurmurHash3's word step and finalizer.
fn hash_word(hash: u32, word: u32) -> u32 {{
    let k = rotate(word * 0xcc9e2d51u, 15u) * 0x1b873593u;
    return rotate(hash ^ k, 13u) * 5u + 0xe6546b64u;
}}

fn hash_finish(hash: u32) -> u32 {{
    var h = hash ^ (hash >> 16u);
    h = h * 0x85ebca6bu;
    h = h ^ (h >> 13u);
    h = h * 0xc2b2ae35u;
    return h ^ (h >> 16u);
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    if (slot >= params.count) {{
        return;
    }}
    slots[slot] = slot;
    let info = triangle_info[slot / 3u];
    if (!is_valid(info)) {{
        hashes[slot] = NO_VERTEX;
        return;
    }}
    let part = info & ~VALID;
    keys[slot * KEY_WORDS] = part;
    var hash = hash_word(0u, part);
    for (var k = 0u; k < VERTEX_FLOATS; k = k + 1u) {{
        let word = quantize(vertices[slot * VERTEX_FLOATS + k]);
        keys[slot * KEY_WORDS + 1u + k] = word;
        hash = hash_word(hash, word);
    }}
    hashes[slot] = min(hash_finish(hash), NO_VERTEX - 1u);
}}
"#,
        counted = wgsl::COUNTED,
        common = common(),
        vertex_floats = VERTEX_FLOATS,
    )
}

/// Each vertex's representative: the lowest slot with its key.
pub(super) fn representatives_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> hashes: array<u32>;
@group(0) @binding(1) var<storage, read> slots: array<u32>;
@group(0) @binding(2) var<storage, read> keys: array<u32>;
@group(0) @binding(3) var<storage, read> triangle_info: array<u32>;
@group(0) @binding(4) var<storage, read_write> representative: array<u32>;
@group(0) @binding(5) var<storage, read_write> first: array<u32>;
{counted}
@group(0) @binding(6) var<uniform> params: Params;
{common}

fn same_key(a: u32, b: u32) -> bool {{
    for (var k = 0u; k < KEY_WORDS; k = k + 1u) {{
        if (keys[a * KEY_WORDS + k] != keys[b * KEY_WORDS + k]) {{
            return false;
        }}
    }}
    return true;
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let sorted = id.x;
    if (sorted >= params.count) {{
        return;
    }}
    let slot = slots[sorted];
    if (!is_valid(triangle_info[slot / 3u])) {{
        representative[slot] = slot;
        first[slot] = 0u;
        return;
    }}
    // The sort is stable, so equal keys sit in slot order within their run
    // of equal hashes: the earliest match is the lowest slot.
    let hash = hashes[sorted];
    var found = slot;
    var j = sorted;
    loop {{
        if (j == 0u || hashes[j - 1u] != hash) {{
            break;
        }}
        j = j - 1u;
        let other = slots[j];
        if (same_key(other, slot)) {{
            found = other;
        }}
    }}
    representative[slot] = found;
    first[slot] = select(0u, 1u, found == slot);
}}
"#,
        counted = wgsl::COUNTED,
        common = common(),
    )
}

/// Copy each representative to its welded index.
pub(super) fn vertices_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> vertices: array<f32>;
@group(0) @binding(1) var<storage, read> first: array<u32>;
@group(0) @binding(2) var<storage, read> rank: array<u32>;
@group(0) @binding(3) var<storage, read_write> welded: array<f32>;
{counted}
@group(0) @binding(4) var<uniform> params: Params;
const VERTEX_FLOATS: u32 = {vertex_floats}u;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    if (slot >= params.count || first[slot] == 0u) {{
        return;
    }}
    let destination = rank[slot] - 1u;
    for (var k = 0u; k < VERTEX_FLOATS; k = k + 1u) {{
        welded[destination * VERTEX_FLOATS + k] = vertices[slot * VERTEX_FLOATS + k];
    }}
}}
"#,
        counted = wgsl::COUNTED,
        vertex_floats = VERTEX_FLOATS,
    )
}

/// Each triangle's welded corners, local to its part, and whether it
/// survives.
pub(super) fn faces_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> triangle_info: array<u32>;
@group(0) @binding(1) var<storage, read> part_slots: array<u32>;
@group(0) @binding(2) var<storage, read> representative: array<u32>;
@group(0) @binding(3) var<storage, read> rank: array<u32>;
@group(0) @binding(4) var<storage, read_write> corners: array<u32>;
@group(0) @binding(5) var<storage, read_write> kept: array<u32>;
{counted}
@group(0) @binding(6) var<uniform> params: Params;
{common}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let triangle = id.x;
    if (triangle >= params.count) {{
        return;
    }}
    let info = triangle_info[triangle];
    if (!is_valid(info)) {{
        kept[triangle] = 0u;
        return;
    }}
    // Representatives before the part's first slot belong to earlier parts.
    let first_slot = part_slots[info & ~VALID];
    var base = 0u;
    if (first_slot > 0u) {{
        base = rank[first_slot - 1u];
    }}
    var index: array<u32, 3>;
    for (var k = 0u; k < 3u; k = k + 1u) {{
        index[k] = rank[representative[triangle * 3u + k]] - 1u - base;
        corners[triangle * 3u + k] = index[k];
    }}
    let distinct = index[0] != index[1] && index[1] != index[2] && index[2] != index[0];
    kept[triangle] = select(0u, 1u, distinct);
}}
"#,
        counted = wgsl::COUNTED,
        common = common(),
    )
}

/// Compact the surviving triangles in order.
pub(super) fn compact_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> corners: array<u32>;
@group(0) @binding(1) var<storage, read> kept: array<u32>;
@group(0) @binding(2) var<storage, read> kept_rank: array<u32>;
@group(0) @binding(3) var<storage, read_write> faces: array<u32>;
{counted}
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let triangle = id.x;
    if (triangle >= params.count || kept[triangle] == 0u) {{
        return;
    }}
    let destination = kept_rank[triangle] - 1u;
    for (var k = 0u; k < 3u; k = k + 1u) {{
        faces[destination * 3u + k] = corners[triangle * 3u + k];
    }}
}}
"#,
        counted = wgsl::COUNTED,
    )
}

/// Each part's welded vertex and surviving triangle counts.
pub(super) fn ranges_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> part_triangles: array<u32>;
@group(0) @binding(1) var<storage, read> rank: array<u32>;
@group(0) @binding(2) var<storage, read> kept_rank: array<u32>;
@group(0) @binding(3) var<storage, read_write> counts: array<u32>;
{counted}
@group(0) @binding(4) var<uniform> params: Params;

// Inclusive scan totals before `end`, zero at the start.
fn before(end: u32, scan_is_rank: bool) -> u32 {{
    if (end == 0u) {{
        return 0u;
    }}
    if (scan_is_rank) {{
        return rank[end - 1u];
    }}
    return kept_rank[end - 1u];
}}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let part = id.x;
    if (part >= params.count) {{
        return;
    }}
    let first = part_triangles[part * 2u];
    let end = first + part_triangles[part * 2u + 1u];
    counts[part * 2u] = before(end * 3u, true) - before(first * 3u, true);
    counts[part * 2u + 1u] = before(end, false) - before(first, false);
}}
"#,
        counted = wgsl::COUNTED,
    )
}
