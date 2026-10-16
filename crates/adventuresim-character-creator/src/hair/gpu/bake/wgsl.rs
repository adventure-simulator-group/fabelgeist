//! The bake's WGSL: each strand drawn into the texture, the bare scalp
//! found and blurred, the textures written, and each mip level averaged
//! from the one before; and the longest strand measured.

/// A topmost strand's packing: its height, direction and shade in one word,
/// height first, so the greatest is the topmost. 0 is no strand.
pub(super) const PACKING: &str = r#"
const HEIGHT_BITS: u32 = 12u;
const DIRECTION_BITS: u32 = 12u;
const SHADE_BITS: u32 = 8u;
const TURN: f32 = 6.283185307179586;

fn quantize(value: f32, bits: u32) -> u32 {
    return u32(round(clamp(value, 0.0, 1.0) * f32((1u << bits) - 1u)));
}

fn field_of(packed: u32, shift: u32, bits: u32) -> f32 {
    let most = (1u << bits) - 1u;
    return f32((packed >> shift) & most) / f32(most);
}

fn pack(height: f32, direction: f32, shade: f32) -> u32 {
    let turn = (direction - TURN * floor(direction / TURN)) / TURN;
    return max(
        (quantize(height, HEIGHT_BITS) << (DIRECTION_BITS + SHADE_BITS))
            | (quantize(turn, DIRECTION_BITS) << SHADE_BITS)
            | quantize(shade, SHADE_BITS),
        1u,
    );
}

fn item(id: vec3<u32>, groups: vec3<u32>) -> u32 {
    return id.y * groups.x * 64u + id.x;
}
"#;

pub(super) const DRAW: &str = r#"
struct StrandGpu {
    normal: vec3<f32>,
    id: u32,
    uv: vec2<f32>,
    group: i32,
    radius: f32,
};
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> topmost: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> shadowed: array<atomic<u32>>;
struct Params {
    center_x: f32,
    center_y: f32,
    center_z: f32,
    rim: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    size: u32,
    forward_x: f32,
    forward_y: f32,
    forward_z: f32,
    count: u32,
    side_x: f32,
    side_y: f32,
    side_z: f32,
    points: u32,
    step: f32,
    reach: f32,
    shadow_reach: f32,
    at_skin: f32,
    at_reach: f32,
    lock_spread: f32,
    strand_spread: f32,
    lock_width: f32,
    block_side: u32,
    blocks: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(5) var<uniform> params: Params;

// A well-mixed 32-bit hash (the "lowbias32" of Chris Wellons).
fn mix_hash(value: u32) -> u32 {
    var x = value + 0x9e3779b9u;
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    x *= 0x846ca68bu;
    x ^= x >> 16u;
    return x;
}

fn unit_of(value: u32) -> f32 {
    return f32(mix_hash(value)) / 4294967295.0;
}

// Smooth noise, 0 to 1, varying over about a unit of `p`.
fn lock_noise(p: vec3<f32>) -> f32 {
    let base = floor(p);
    let t = p - base;
    let fraction = t * t * (vec3<f32>(3.0) - 2.0 * t);
    var sum = 0.0;
    for (var corner = 0u; corner < 8u; corner++) {
        let offset = vec3<u32>(corner & 1u, (corner >> 1u) & 1u, (corner >> 2u) & 1u);
        var key = 0u;
        for (var axis = 0u; axis < 3u; axis++) {
            key = mix_hash(key ^ bitcast<u32>(i32(base[axis]) + i32(offset[axis])));
        }
        var weight = 1.0;
        for (var axis = 0u; axis < 3u; axis++) {
            weight *= select(1.0 - fraction[axis], fraction[axis], offset[axis] == 1u);
        }
        sum += weight * unit_of(key);
    }
    return sum;
}

// A point of the unit disk on the square from -1 to 1 each way: Shirley and
// Chiu's concentric mapping, inverted.
fn square(reach: f32, azimuth_in: f32) -> vec2<f32> {
    let quarter = 0.7853981633974483;
    let shifted = azimuth_in + quarter;
    let azimuth = shifted - TURN * floor(shifted / TURN) - quarter;
    if (azimuth < quarter) {
        return vec2<f32>(reach, azimuth / quarter * reach);
    }
    if (azimuth < 3.0 * quarter) {
        return vec2<f32>(-(azimuth - 2.0 * quarter) / quarter * reach, reach);
    }
    if (azimuth < 5.0 * quarter) {
        return vec2<f32>(-reach, -(azimuth - 4.0 * quarter) / quarter * reach);
    }
    return vec2<f32>((azimuth - 6.0 * quarter) / quarter * reach, -reach);
}

// Where `p` lies in the texture, texels, and whether it lies in it.
fn map_texel(p: vec3<f32>) -> vec3<f32> {
    let offset = p - vec3<f32>(params.center_x, params.center_y, params.center_z);
    let length_of = length(offset);
    if (length_of <= 1e-7) {
        return vec3<f32>(0.0);
    }
    let direction = offset / length_of;
    let axis = vec3<f32>(params.axis_x, params.axis_y, params.axis_z);
    let reach = sin(acos(clamp(dot(direction, axis), -1.0, 1.0)) * 0.5) / params.rim;
    if (reach > 1.0) {
        return vec3<f32>(0.0);
    }
    let azimuth = atan2(
        dot(direction, vec3<f32>(params.side_x, params.side_y, params.side_z)),
        dot(direction, vec3<f32>(params.forward_x, params.forward_y, params.forward_z)),
    );
    let ab = square(reach, azimuth);
    return vec3<f32>((ab + vec2<f32>(1.0)) * 0.5 * f32(params.size), 1.0);
}

fn draw(begin: vec2<f32>, end: vec2<f32>, begin_height: f32, end_height: f32, tone: f32) {
    let direction = atan2(end.y - begin.y, end.x - begin.x);
    let steps = max(u32(ceil(max(abs(end.x - begin.x), abs(end.y - begin.y)))), 1u);
    for (var i = 0u; i <= steps; i++) {
        let t = f32(i) / f32(steps);
        let at = begin + (end - begin) * t;
        let column = u32(at.x);
        let row = u32(at.y);
        if (column < params.size && row < params.size) {
            let height = min((begin_height + (end_height - begin_height) * t) / params.reach, 1.0);
            let shade = (params.at_skin + (params.at_reach - params.at_skin) * height) * tone;
            atomicMax(&topmost[row * params.size + column], pack(height, direction, shade));
        }
    }
}

fn shade_blocks(begin: vec2<f32>, end: vec2<f32>) {
    let steps = max(u32(ceil(max(abs(end.x - begin.x), abs(end.y - begin.y)))), 1u);
    for (var i = 0u; i <= steps; i++) {
        let at = begin + (end - begin) * (f32(i) / f32(steps));
        let column = u32(at.x);
        let row = u32(at.y);
        if (column < params.size && row < params.size) {
            let block = (row / params.block_side) * params.blocks + column / params.block_side;
            atomicStore(&shadowed[block], 1u);
        }
    }
}

// One point along a strand: drawn from the last if both lie near enough
// the skin, or their stretch marked as shadowing the scalp below.
fn visit(
    q: vec3<f32>,
    tone: f32,
    last_seen: ptr<function, bool>,
    last_at: ptr<function, vec2<f32>>,
    last_height: ptr<function, f32>,
) {
    let height = max(head_distance(q), 0.0);
    let mapped = map_texel(q);
    let seen = mapped.z > 0.5 && height < params.shadow_reach;
    if (seen && *last_seen) {
        if (min(*last_height, height) < params.reach) {
            draw(*last_at, mapped.xy, *last_height, height, tone);
        } else {
            shade_blocks(*last_at, mapped.xy);
        }
    }
    *last_seen = seen;
    *last_at = mapped.xy;
    *last_height = height;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let base = s * params.points;
    let lock = lock_noise(positions[base].xyz / params.lock_width);
    let tone = 1.0 - params.lock_spread * lock - params.strand_spread * unit_of(strands[s].id);
    var last_seen = false;
    var last_at = vec2<f32>(0.0);
    var last_height = 0.0;
    var last = positions[base].xyz;
    visit(last, tone, &last_seen, &last_at, &last_height);
    for (var j = 1u; j < params.points; j++) {
        let next = positions[base + j].xyz;
        let pieces = max(u32(ceil(length(next - last) / params.step)), 1u);
        for (var piece = 1u; piece <= pieces; piece++) {
            visit(mix(last, next, f32(piece) / f32(pieces)), tone, &last_seen, &last_at, &last_height);
        }
        last = next;
    }
}
"#;

pub(super) const BARE: &str = r#"
@group(0) @binding(0) var<storage, read> topmost: array<u32>;
@group(0) @binding(1) var<storage, read> shadowed: array<u32>;
@group(0) @binding(2) var<storage, read_write> bare: array<f32>;
struct Params {
    size: u32,
    block_side: u32,
    blocks: u32,
    pad0: u32,
};
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = id.y * groups.x * 64u + id.x;
    if (k >= params.size * params.size) {
        return;
    }
    let row = k / params.size;
    let column = k % params.size;
    let block = (row / params.block_side) * params.blocks + column / params.block_side;
    bare[k] = select(0.0, 1.0, topmost[k] == 0u && shadowed[block] == 0u);
}
"#;

pub(super) const BLUR: &str = r#"
@group(0) @binding(0) var<storage, read> blur_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> blur_out: array<f32>;
struct Params {
    size: u32,
    stride: u32,
    reach: u32,
    pad0: u32,
};
@group(0) @binding(2) var<uniform> params: Params;

// The mean within `reach` texels along a row (`stride` 1) or a column.
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = id.y * groups.x * 64u + id.x;
    if (k >= params.size * params.size) {
        return;
    }
    let position = (k / params.stride) % params.size;
    let start = k - position * params.stride;
    let first = select(position - params.reach, 0u, position < params.reach);
    let last = min(position + params.reach, params.size - 1u);
    var sum = 0.0;
    for (var other = first; other <= last; other++) {
        sum += blur_in[start + other * params.stride];
    }
    blur_out[k] = sum / f32(last - first + 1u);
}
"#;

pub(super) const FINISH: &str = r#"
@group(0) @binding(0) var<storage, read> topmost: array<u32>;
@group(0) @binding(1) var<storage, read> share: array<f32>;
@group(0) @binding(2) var<storage, read_write> shade: array<u32>;
@group(0) @binding(3) var<storage, read_write> strands: array<u32>;
struct Params {
    size: u32,
    shadowed_shade: f32,
    bare_start: f32,
    bare_full: f32,
    strand_over_skin: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(4) var<uniform> params: Params;

fn to_byte(value: f32) -> u32 {
    return u32(round(clamp(value, 0.0, 1.0) * 255.0));
}

fn texel_of(r: f32, g: f32, b: f32, a: f32) -> u32 {
    return to_byte(r) | (to_byte(g) << 8u) | (to_byte(b) << 16u) | (to_byte(a) << 24u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.size * params.size) {
        return;
    }
    let packed = topmost[k];
    let t = clamp((share[k] - params.bare_start) / (params.bare_full - params.bare_start), 0.0, 1.0);
    let skin = t * t * (3.0 - 2.0 * t);
    var tone = params.shadowed_shade;
    var along = vec2<f32>(0.0);
    var clear = 1.0;
    if (packed != 0u) {
        tone = field_of(packed, 0u, SHADE_BITS);
        let direction = field_of(packed, SHADE_BITS, DIRECTION_BITS) * TURN;
        along = vec2<f32>(cos(direction), sin(direction));
        clear = 1.0 - params.strand_over_skin;
    }
    shade[k] = texel_of(tone, tone, tone, 1.0 - skin * clear);
    strands[k] = texel_of(along.x * 0.5 + 0.5, along.y * 0.5 + 0.5, tone, 1.0);
}
"#;

pub(super) const HALVE: &str = r#"
@group(0) @binding(0) var<storage, read_write> chain: array<u32>;
struct Params {
    from_side: u32,
    from_stride: u32,
    from_offset: u32,
    to_side: u32,
    to_stride: u32,
    to_offset: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(1) var<uniform> params: Params;

fn texel_at(x: u32, y: u32) -> vec4<u32> {
    let side = params.from_side;
    let v = chain[params.from_offset + min(y, side - 1u) * params.from_stride + min(x, side - 1u)];
    return vec4<u32>(v & 255u, (v >> 8u) & 255u, (v >> 16u) & 255u, v >> 24u);
}

// Each texel of the next level the mean of four of this one's.
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = id.y * groups.x * 64u + id.x;
    if (k >= params.to_side * params.to_side) {
        return;
    }
    let x = k % params.to_side;
    let y = k / params.to_side;
    let sum = texel_at(2u * x, 2u * y) + texel_at(2u * x + 1u, 2u * y)
        + texel_at(2u * x, 2u * y + 1u) + texel_at(2u * x + 1u, 2u * y + 1u);
    let mean = sum / 4u;
    chain[params.to_offset + y * params.to_stride + x] = mean.x | (mean.y << 8u) | (mean.z << 16u) | (mean.w << 24u);
}
"#;

/// The longest strand's arc length, as the bits of a float: lengths are
/// never negative, so the greatest bits are the greatest length.
pub(super) const LONGEST: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> longest: array<atomic<u32>>;
struct Params {
    count: u32,
    points: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = id.y * groups.x * 64u + id.x;
    if (s >= params.count) {
        return;
    }
    let base = s * params.points;
    var length_of = 0.0;
    for (var j = 1u; j < params.points; j++) {
        length_of += distance(positions[base + j].xyz, positions[base + j - 1u].xyz);
    }
    atomicMax(&longest[0], bitcast<u32>(length_of));
}
"#;
