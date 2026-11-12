//! The WGSL the GPU hierarchy is built from, plus the traversal source other
//! crates paste into their own kernels.
//!
//! A BVH is only useful to a solver if the solver can walk it *inside* its own
//! kernel -- a contact pass wants to find candidate triangles and resolve them
//! in one dispatch, not read a candidate list back and forth. So the traversal
//! is published as source rather than as a kernel: call [`traversal_source`],
//! paste the result into your kernel, and call `bvh_query_aabb` from it.
//!
//! # Buffer layout
//!
//! * `nodes: array<vec4<f32>>` -- two entries per node. `[0].xyz` is the lower
//!   corner and `[0].w` (bitcast to `u32`) is the left child; `[1].xyz` is the
//!   upper corner and `[1].w` is the primitive count, which is zero for an
//!   internal node and one for a leaf.
//! * `right_children: array<u32>` -- the right child of each internal node.
//!   It lives outside the node because `[1].w` has to stay zero to mark the
//!   node internal.
//! * `indices: array<u32>` -- primitives in Morton order. Leaf `i` (node
//!   `count - 1 + i`) holds primitive `indices[i]`.
//! * `primitive_bounds: array<vec4<f32>>` -- two entries per primitive, lower
//!   then upper.

/// Encode/decode between `f32` and a `u32` that compares the same way, so that
/// `atomicMin`/`atomicMax` can reduce floats. WGSL has no atomic float.
///
/// A non-negative float already compares correctly as an integer once the sign
/// bit is set; a negative one needs every bit flipped, which turns its
/// descending integer order into an ascending one.
pub const ORDERED_FLOAT: &str = r#"
fn ordered_from_float(value: f32) -> u32 {
    let bits = bitcast<u32>(value);
    if ((bits & 0x80000000u) != 0u) {
        return ~bits;
    }
    return bits | 0x80000000u;
}

fn float_from_ordered(bits: u32) -> f32 {
    if ((bits & 0x80000000u) != 0u) {
        return bitcast<f32>(bits & 0x7FFFFFFFu);
    }
    return bitcast<f32>(~bits);
}
"#;

/// Morton encoding, matching [`crate::morton`] bit for bit.
pub const MORTON: &str = r#"
fn bvh_expand_bits(value: u32) -> u32 {
    var v = value & 0x3FFu;
    v = (v | (v << 16u)) & 0x030000FFu;
    v = (v | (v << 8u))  & 0x0300F00Fu;
    v = (v | (v << 4u))  & 0x030C30C3u;
    v = (v | (v << 2u))  & 0x09249249u;
    return v;
}

fn bvh_morton(unit: vec3<f32>) -> u32 {
    let q = vec3<u32>(clamp(unit * 1024.0, vec3<f32>(0.0), vec3<f32>(1023.0)));
    return (bvh_expand_bits(q.x) << 2u) | (bvh_expand_bits(q.y) << 1u) | bvh_expand_bits(q.z);
}
"#;

const PARAMS: &str = r#"
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
"#;

/// Reduce every primitive's box into the six words of the scene bounds.
pub fn bounds_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> primitive_bounds: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> scene_bounds: array<atomic<u32>>;
{PARAMS}
@group(0) @binding(2) var<uniform> params: Params;
{ORDERED_FLOAT}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;
    if (index >= params.count) {{
        return;
    }}
    let lower = primitive_bounds[index * 2u].xyz;
    let upper = primitive_bounds[index * 2u + 1u].xyz;

    atomicMin(&scene_bounds[0], ordered_from_float(lower.x));
    atomicMin(&scene_bounds[1], ordered_from_float(lower.y));
    atomicMin(&scene_bounds[2], ordered_from_float(lower.z));
    atomicMax(&scene_bounds[3], ordered_from_float(upper.x));
    atomicMax(&scene_bounds[4], ordered_from_float(upper.y));
    atomicMax(&scene_bounds[5], ordered_from_float(upper.z));
}}
"#
    )
}

/// A Morton code per primitive, from its centroid's position in the scene box.
pub fn codes_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> primitive_bounds: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> scene_bounds: array<u32>;
@group(0) @binding(2) var<storage, read_write> codes: array<u32>;
@group(0) @binding(3) var<storage, read_write> indices: array<u32>;
{PARAMS}
@group(0) @binding(4) var<uniform> params: Params;
{ORDERED_FLOAT}
{MORTON}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;
    if (index >= params.count) {{
        return;
    }}

    let scene_min = vec3<f32>(
        float_from_ordered(scene_bounds[0]),
        float_from_ordered(scene_bounds[1]),
        float_from_ordered(scene_bounds[2]),
    );
    let scene_max = vec3<f32>(
        float_from_ordered(scene_bounds[3]),
        float_from_ordered(scene_bounds[4]),
        float_from_ordered(scene_bounds[5]),
    );

    let centroid = (primitive_bounds[index * 2u].xyz + primitive_bounds[index * 2u + 1u].xyz) * 0.5;
    let extent = scene_max - scene_min;
    // A flat scene has a zero extent on one axis. Sending everything on that
    // axis to the middle of the range beats dividing by zero.
    let unit = select(
        vec3<f32>(0.5),
        (centroid - scene_min) / max(extent, vec3<f32>(1e-20)),
        extent > vec3<f32>(0.0),
    );

    codes[index] = bvh_morton(unit);
    indices[index] = index;
}}
"#
    )
}

/// Karras's hierarchy: `count - 1` internal nodes, each deriving its own range
/// and split from the sorted codes alone, with no communication between them.
///
/// Internal nodes are `0 .. count-1`; leaves are `count-1 .. 2*count-1`, with
/// leaf `i` holding sorted primitive `i`.
pub fn hierarchy_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> codes: array<u32>;
@group(0) @binding(1) var<storage, read_write> nodes: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> parents: array<u32>;
@group(0) @binding(3) var<storage, read_write> right_children: array<u32>;
{PARAMS}
@group(0) @binding(4) var<uniform> params: Params;

// Length of the common prefix of codes `i` and `j`, with ties broken on the
// indices. Without the tie-break, duplicate codes -- and a panel meshed as a
// regular grid produces plenty -- collapse the tree into a list.
fn delta(i: u32, j: i32) -> i32 {{
    if (j < 0 || u32(j) >= params.count) {{
        return -1;
    }}
    let a = codes[i];
    let b = codes[u32(j)];
    if (a == b) {{
        return 32 + i32(countLeadingZeros(i ^ u32(j)));
    }}
    return i32(countLeadingZeros(a ^ b));
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let i = global_id.x;
    if (params.count < 2u || i >= params.count - 1u) {{
        return;
    }}

    // Which way this node's range runs from `i`.
    let direction = select(-1, 1, delta(i, i32(i) + 1) >= delta(i, i32(i) - 1));
    let minimum = delta(i, i32(i) - direction);

    // Grow a power-of-two bound on the range's length, then binary search
    // inside it for the exact end.
    var maximum = 2;
    while (delta(i, i32(i) + maximum * direction) > minimum) {{
        maximum = maximum * 2;
    }}

    var length = 0;
    var step = maximum / 2;
    while (step >= 1) {{
        if (delta(i, i32(i) + (length + step) * direction) > minimum) {{
            length = length + step;
        }}
        step = step / 2;
    }}
    let other = i32(i) + length * direction;

    // The split sits where the prefix shared across the whole range runs out.
    let node_delta = delta(i, other);
    var split = 0;
    var stride = length;
    loop {{
        stride = (stride + 1) / 2;
        if (delta(i, i32(i) + (split + stride) * direction) > node_delta) {{
            split = split + stride;
        }}
        if (stride <= 1) {{
            break;
        }}
    }}
    let gamma = i32(i) + split * direction + min(direction, 0);

    let first = min(i32(i), other);
    let last = max(i32(i), other);

    // A child covering a single primitive is a leaf, which lives in the upper
    // half of the node array.
    var left = u32(gamma);
    if (first == gamma) {{
        left = params.count - 1u + u32(gamma);
    }}
    var right = u32(gamma + 1);
    if (last == gamma + 1) {{
        right = params.count - 1u + u32(gamma + 1);
    }}

    nodes[i * 2u].w = bitcast<f32>(left);
    // `upper.w` must stay zero to mark the node internal, so the right child
    // rides alongside rather than inside.
    nodes[i * 2u + 1u].w = bitcast<f32>(0u);
    right_children[i] = right;
    parents[left] = i;
    parents[right] = i;
}}
"#
    )
}

/// Reset every node's bounds to the empty box, ready for the push-up.
pub fn clear_bounds_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> node_bounds: array<atomic<u32>>;
{PARAMS}
@group(0) @binding(1) var<uniform> params: Params;
{ORDERED_FLOAT}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    // Six words per node: three for the lower corner, three for the upper.
    let index = global_id.x;
    let nodes = 2u * params.count - 1u;
    if (index >= nodes) {{
        return;
    }}
    let empty_low = ordered_from_float(0x1p+127f);
    let empty_high = ordered_from_float(-0x1p+127f);
    atomicStore(&node_bounds[index * 6u + 0u], empty_low);
    atomicStore(&node_bounds[index * 6u + 1u], empty_low);
    atomicStore(&node_bounds[index * 6u + 2u], empty_low);
    atomicStore(&node_bounds[index * 6u + 3u], empty_high);
    atomicStore(&node_bounds[index * 6u + 4u], empty_high);
    atomicStore(&node_bounds[index * 6u + 5u], empty_high);
}}
"#
    )
}

/// Bottom-up refit, done by having each leaf push its own box into every
/// ancestor with `atomicMin`/`atomicMax`.
///
/// The textbook version instead has one thread per node continue upwards, with
/// an atomic counter deciding which of two children carries on -- half the
/// atomic traffic. It also depends on one workgroup seeing another workgroup's
/// plain stores, and WGSL guarantees no such thing: `storageBarrier` is scoped
/// to a workgroup, and a relaxed atomic orders nothing around it. Pushing
/// through atomics needs no ordering at all -- min and max do not care what
/// order they arrive in -- so it is correct by construction. The cost is
/// `leaves x depth` atomic operations, which for the tens of thousands of
/// primitives here is nothing next to a shader launch.
pub fn refit_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> primitive_bounds: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> indices: array<u32>;
@group(0) @binding(2) var<storage, read> parents: array<u32>;
@group(0) @binding(3) var<storage, read_write> node_bounds: array<atomic<u32>>;
{PARAMS}
@group(0) @binding(4) var<uniform> params: Params;
{ORDERED_FLOAT}

fn push(node: u32, lower: vec3<f32>, upper: vec3<f32>) {{
    atomicMin(&node_bounds[node * 6u + 0u], ordered_from_float(lower.x));
    atomicMin(&node_bounds[node * 6u + 1u], ordered_from_float(lower.y));
    atomicMin(&node_bounds[node * 6u + 2u], ordered_from_float(lower.z));
    atomicMax(&node_bounds[node * 6u + 3u], ordered_from_float(upper.x));
    atomicMax(&node_bounds[node * 6u + 4u], ordered_from_float(upper.y));
    atomicMax(&node_bounds[node * 6u + 5u], ordered_from_float(upper.z));
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let leaf = global_id.x;
    if (leaf >= params.count) {{
        return;
    }}

    let primitive = indices[leaf];
    let lower = primitive_bounds[primitive * 2u].xyz;
    let upper = primitive_bounds[primitive * 2u + 1u].xyz;

    var node = params.count - 1u + leaf;
    push(node, lower, upper);

    // A single-primitive tree is its own root, and has no parents array.
    if (params.count < 2u) {{
        return;
    }}

    // Walk to the root. The bound is the tallest tree the node count admits,
    // so a corrupt `parents` array cannot spin here forever.
    for (var step = 0u; step < 2u * params.count; step = step + 1u) {{
        node = parents[node];
        push(node, lower, upper);
        if (node == 0u) {{
            return;
        }}
    }}
}}
"#
    )
}

/// Copy the atomically-reduced bounds into the node array, leaving the child
/// and count lanes alone.
pub fn gather_bounds_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> node_bounds: array<u32>;
@group(0) @binding(1) var<storage, read_write> nodes: array<vec4<f32>>;
{PARAMS}
@group(0) @binding(2) var<uniform> params: Params;
{ORDERED_FLOAT}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;
    let node_count = 2u * params.count - 1u;
    if (index >= node_count) {{
        return;
    }}

    let lower = vec3<f32>(
        float_from_ordered(node_bounds[index * 6u + 0u]),
        float_from_ordered(node_bounds[index * 6u + 1u]),
        float_from_ordered(node_bounds[index * 6u + 2u]),
    );
    let upper = vec3<f32>(
        float_from_ordered(node_bounds[index * 6u + 3u]),
        float_from_ordered(node_bounds[index * 6u + 4u]),
        float_from_ordered(node_bounds[index * 6u + 5u]),
    );

    // Leaves carry their slot in `indices` and a count of one; internal nodes
    // keep the left child the hierarchy pass wrote and a count of zero.
    if (index >= params.count - 1u) {{
        let leaf = index - (params.count - 1u);
        nodes[index * 2u] = vec4<f32>(lower, bitcast<f32>(leaf));
        nodes[index * 2u + 1u] = vec4<f32>(upper, bitcast<f32>(1u));
    }} else {{
        nodes[index * 2u] = vec4<f32>(lower, nodes[index * 2u].w);
        nodes[index * 2u + 1u] = vec4<f32>(upper, bitcast<f32>(0u));
    }}
}}
"#
    )
}

/// How a generated traversal names the buffers it reads and the function it
/// calls back into.
///
/// The defaults match what [`super::GpuBvh::parameters`] binds, so a kernel
/// that takes the BVH as-is only needs to set `callback`.
#[derive(Clone, Debug)]
pub struct TraversalConfig {
    /// Name of the `fn (primitive: u32)` the traversal calls per hit. It must
    /// be declared before the generated source.
    pub callback: String,
    pub nodes: String,
    pub right_children: String,
    pub indices: String,
    /// Prefix on the generated function names, so that one kernel can
    /// traverse two different hierarchies.
    pub prefix: String,
    /// Traversal stack depth. A Morton hierarchy over duplicate-heavy input
    /// can get deep, so this is generous by default.
    pub stack_size: u32,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            callback: "bvh_hit".into(),
            nodes: "bvh_nodes".into(),
            right_children: "bvh_right_children".into(),
            indices: "bvh_indices".into(),
            prefix: "bvh".into(),
            stack_size: 64,
        }
    }
}

impl TraversalConfig {
    pub fn with_callback(callback: impl Into<String>) -> Self {
        Self {
            callback: callback.into(),
            ..Default::default()
        }
    }
}

/// WGSL for walking the hierarchy from inside another kernel.
///
/// Generates `<prefix>_query_aabb(lower, upper)` and
/// `<prefix>_query_point(point, radius)`, both calling `callback(primitive)`
/// for every primitive whose *box* passes. The caller does the exact test --
/// the BVH only ever promises candidates.
pub fn traversal_source(config: &TraversalConfig) -> String {
    let TraversalConfig {
        callback,
        nodes,
        right_children,
        indices,
        prefix,
        stack_size,
    } = config;

    format!(
        r#"
fn {prefix}_query_aabb(query_lower: vec3<f32>, query_upper: vec3<f32>) {{
    var stack: array<u32, {stack_size}u>;
    var depth = 1u;
    stack[0] = 0u;

    // A full stack drops the deepest branch rather than writing out of
    // bounds, so an over-deep tree loses candidates instead of corrupting
    // memory next door.
    loop {{
        if (depth == 0u) {{
            break;
        }}
        depth = depth - 1u;
        let index = stack[depth];

        let lower = {nodes}[index * 2u];
        let upper = {nodes}[index * 2u + 1u];
        if (any(lower.xyz > query_upper) || any(upper.xyz < query_lower)) {{
            continue;
        }}

        if (bitcast<u32>(upper.w) != 0u) {{
            {callback}({indices}[bitcast<u32>(lower.w)]);
        }} else {{
            if (depth + 2u <= {stack_size}u) {{
                stack[depth] = bitcast<u32>(lower.w);
                stack[depth + 1u] = {right_children}[index];
                depth = depth + 2u;
            }}
        }}
    }}
}}

fn {prefix}_query_point(point: vec3<f32>, radius: f32) {{
    let r = vec3<f32>(radius);
    {prefix}_query_aabb(point - r, point + r);
}}
"#
    )
}
