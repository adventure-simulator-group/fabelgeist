//! The layer stack limited against nearby opposing body surfaces.
//!
//! Every vertex casts its offset ray a short reach and keeps a share of the
//! distance to the nearest facing triangle. Only triangles sharing a grid
//! cell with the ray's bounds are tested: the cells are hashed into buckets,
//! each triangle entered in every cell its bounds cover, and a ray tests
//! exactly the triangles whose cell range meets its own. The nearest hit is a
//! minimum, so the order candidates are found in does not matter.

use adventuresim_armor_model::gpu::device_error;
use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::standoff::LAYER_STACK_ENVELOPE_M;
use super::wgsl;
use super::workspace::Workspace;

/// Hits closer than this are the ray leaving its own surface.
pub const RAY_ROUNDOFF_M: f32 = 1e-6;
/// Share of the distance to an opposing surface one layer stack may take.
pub const GAP_ALLOCATION: f32 = 0.25;
/// Buckets of the cell hash.
pub(super) const BUCKETS: u32 = 1 << 16;
/// Cells one triangle is entered in; larger triangles are tested by every
/// ray instead.
pub(super) const CELLS_PER_FACE: u32 = 8;
/// Key bits of a bucket, including the key of an unused slot.
const BUCKET_BITS: u32 = BUCKETS.trailing_zeros() + 1;

impl Workspace<'_> {
    /// Record the gap limit of every vertex of a realization into the first
    /// half of `rooms`, starting from the full layer stack.
    pub(super) fn record_gap(
        &mut self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        directions: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let kernel = |source: String| {
            gpu.cache()
                .get(gpu.context(), &source)
                .map_err(device_error)
        };
        let pairs = self.face_count * CELLS_PER_FACE;
        batch.clear_buffer(&self.overflow);
        batch.clear_buffer(&self.ranges);
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.face_count);
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad, 0u32);
        }
        parameters.insert("positions", positions.clone());
        parameters.insert("faces", self.faces.clone());
        parameters.insert("keys", self.keys.clone());
        parameters.insert("values", self.values.clone());
        parameters.insert("overflow", self.overflow.clone());
        batch
            .dispatch_items(&*kernel(cells_source())?, &parameters, self.face_count)
            .map_err(device_error)?;
        self.sort
            .record(
                batch,
                &self.keys,
                &self.values,
                &mut self.sort_scratch,
                pairs,
                BUCKET_BITS,
            )
            .map_err(device_error)?;
        let mut parameters = PassParameters::new();
        parameters.insert("count", pairs);
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad, 0u32);
        }
        parameters.insert("keys", self.keys.clone());
        parameters.insert("ranges", self.ranges.clone());
        batch
            .dispatch_items(&*kernel(ranges_source())?, &parameters, pairs)
            .map_err(device_error)?;
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.vertex_count);
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad, 0u32);
        }
        parameters.insert("positions", positions.clone());
        parameters.insert("directions", directions.clone());
        parameters.insert("faces", self.faces.clone());
        parameters.insert("ranges", self.ranges.clone());
        parameters.insert("values", self.values.clone());
        parameters.insert("overflow", self.overflow.clone());
        parameters.insert("rooms", self.rooms.clone());
        parameters.insert("status", self.status.clone());
        batch
            .dispatch_items(&*kernel(query_source())?, &parameters, self.vertex_count)
            .map_err(device_error)?;
        Ok(())
    }
}

/// Grid cells of the gap search, and their bucket.
fn cells() -> String {
    format!(
        r#"
{envelope}
const REACH: f32 = LAYER_STACK_ENVELOPE_M * 2.0;
const BUCKETS: u32 = {buckets}u;

fn point(v: u32) -> vec3<f32> {{
    return vec3<f32>(positions[v * 3u], positions[v * 3u + 1u], positions[v * 3u + 2u]);
}}

fn cell(p: vec3<f32>) -> vec3<i32> {{
    return vec3<i32>(floor(p / REACH));
}}

fn bucket(c: vec3<i32>) -> u32 {{
    let h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u)
        ^ (bitcast<u32>(c.z) * 83492791u);
    return h % BUCKETS;
}}

// A triangle's cell range: the cells of its bounds' corners.
fn face_low(f: u32) -> vec3<i32> {{
    return cell(min(min(point(faces[f * 3u]), point(faces[f * 3u + 1u])), point(faces[f * 3u + 2u])));
}}

fn face_high(f: u32) -> vec3<i32> {{
    return cell(max(max(point(faces[f * 3u]), point(faces[f * 3u + 1u])), point(faces[f * 3u + 2u])));
}}
"#,
        envelope = wgsl::constant("LAYER_STACK_ENVELOPE_M", LAYER_STACK_ENVELOPE_M),
        buckets = BUCKETS,
    )
}

fn cells_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> faces: array<u32>;
@group(0) @binding(2) var<storage, read_write> keys: array<u32>;
@group(0) @binding(3) var<storage, read_write> values: array<u32>;
@group(0) @binding(4) var<storage, read_write> overflow: array<atomic<u32>>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(5) var<uniform> params: Params;
{cells}
const CELLS_PER_FACE: u32 = {per_face}u;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let f = id.x;
    if (f >= params.count) {{
        return;
    }}
    let low = face_low(f);
    let span = vec3<u32>(face_high(f) - low + vec3<i32>(1));
    var slot = 0u;
    if (span.x * span.y * span.z <= CELLS_PER_FACE) {{
        for (var x = 0u; x < span.x; x = x + 1u) {{
            for (var y = 0u; y < span.y; y = y + 1u) {{
                for (var z = 0u; z < span.z; z = z + 1u) {{
                    keys[f * CELLS_PER_FACE + slot] = bucket(low + vec3<i32>(vec3<u32>(x, y, z)));
                    values[f * CELLS_PER_FACE + slot] = f;
                    slot = slot + 1u;
                }}
            }}
        }}
    }} else {{
        let index = atomicAdd(&overflow[0], 1u);
        atomicStore(&overflow[1u + index], f);
    }}
    for (; slot < CELLS_PER_FACE; slot = slot + 1u) {{
        keys[f * CELLS_PER_FACE + slot] = BUCKETS;
        values[f * CELLS_PER_FACE + slot] = f;
    }}
}}
"#,
        cells = cells(),
        per_face = CELLS_PER_FACE,
    )
}

fn ranges_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> ranges: array<u32>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;
const BUCKETS: u32 = {buckets}u;

// Each run of equal buckets marks its first and one-past-last slot.
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    if (slot >= params.count) {{
        return;
    }}
    let key = keys[slot];
    if (key >= BUCKETS) {{
        return;
    }}
    if (slot == 0u || keys[slot - 1u] != key) {{
        ranges[key * 2u] = slot;
    }}
    if (slot + 1u == params.count || keys[slot + 1u] != key) {{
        ranges[key * 2u + 1u] = slot + 1u;
    }}
}}
"#,
        buckets = BUCKETS,
    )
}

/// One invocation per vertex: the room along its offset direction before a facing surface.
const QUERY_GAPS: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    ray_vertex = id.x;
    if (ray_vertex >= params.count) {
        return;
    }
    origin = point(ray_vertex);
    direction = vec3<f32>(
        directions[ray_vertex * 3u],
        directions[ray_vertex * 3u + 1u],
        directions[ray_vertex * 3u + 2u],
    );
    let end = vec3<f32>(
        origin.x + direction.x * REACH,
        origin.y + direction.y * REACH,
        origin.z + direction.z * REACH,
    );
    ray_low = cell(min(origin, end));
    ray_high = cell(max(origin, end));
    room = LAYER_STACK_ENVELOPE_M;
    let span = vec3<u32>(ray_high - ray_low + vec3<i32>(1));
    if (span.x * span.y * span.z > RAY_CELLS) {
        fail(STATUS_LONG_RAY);
    } else {
        for (var x = 0u; x < span.x; x = x + 1u) {
            for (var y = 0u; y < span.y; y = y + 1u) {
                for (var z = 0u; z < span.z; z = z + 1u) {
                    let b = bucket(ray_low + vec3<i32>(vec3<u32>(x, y, z)));
                    for (var s = ranges[b * 2u]; s < ranges[b * 2u + 1u]; s = s + 1u) {
                        consider(values[s]);
                    }
                }
            }
        }
    }
    for (var k = 0u; k < overflow[0]; k = k + 1u) {
        consider(overflow[1u + k]);
    }
    rooms[ray_vertex] = room;
}
"#;

fn query_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> directions: array<f32>;
@group(0) @binding(2) var<storage, read> faces: array<u32>;
@group(0) @binding(3) var<storage, read> ranges: array<u32>;
@group(0) @binding(4) var<storage, read> values: array<u32>;
@group(0) @binding(5) var<storage, read> overflow: array<u32>;
@group(0) @binding(6) var<storage, read_write> rooms: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{status}
{vector}
{cells}
{roundoff}
{allocation}
// The cells a unit ray of the reach can span.
const RAY_CELLS: u32 = 8u;

var<private> ray_vertex: u32;
var<private> origin: vec3<f32>;
var<private> direction: vec3<f32>;
var<private> ray_low: vec3<i32>;
var<private> ray_high: vec3<i32>;
var<private> room: f32;

// Limit the room by the distance along the ray to triangle `f`, if it faces
// back at the ray and lies within its cells and reach.
fn consider(f: u32) {{
    let a = faces[f * 3u];
    let b = faces[f * 3u + 1u];
    let c = faces[f * 3u + 2u];
    if (a == ray_vertex || b == ray_vertex || c == ray_vertex) {{
        return;
    }}
    let low = face_low(f);
    let high = face_high(f);
    if (any(high < ray_low) || any(low > ray_high)) {{
        return;
    }}
    let t0 = point(a);
    let e = point(b) - t0;
    let g = point(c) - t0;
    let p = cross3(direction, g);
    let determinant = dot3(e, p);
    if (determinant <= 0.0) {{
        return;
    }}
    let t = origin - t0;
    let u = dot3(t, p) / determinant;
    let q = cross3(t, e);
    let v = dot3(direction, q) / determinant;
    if (u < 0.0 || v < 0.0 || u + v > 1.0) {{
        return;
    }}
    let distance = dot3(g, q) / determinant;
    if (distance > RAY_ROUNDOFF_M && distance < REACH) {{
        room = min(room, distance * GAP_ALLOCATION);
    }}
}}

{query_gaps}
"#,
        query_gaps = QUERY_GAPS,
        status = wgsl::status(),
        vector = wgsl::VECTOR,
        cells = cells(),
        roundoff = wgsl::constant("RAY_ROUNDOFF_M", RAY_ROUNDOFF_M),
        allocation = wgsl::constant("GAP_ALLOCATION", GAP_ALLOCATION),
    )
}
