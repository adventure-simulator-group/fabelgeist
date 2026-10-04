//! Coincident body vertices, found on the device.
//!
//! Vertices whose positions round to the same micrometre are one physical
//! vertex: UV seams duplicate a vertex, and both copies must move together.
//! Every vertex hashes its rounded position, a stable radix sort gathers
//! equal hashes in ascending vertex order, and each vertex links to the
//! lowest and to the next vertex of its group within its hash run.

use anyhow::Result;
use fabelgeist_armor::gpu::device_error;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use super::wgsl;
use super::workspace::Workspace;

/// Positions agreeing to this many parts per metre are one physical vertex.
pub const WELD_PRECISION: f32 = 1_000_000.0;
/// The most vertices one hash may gather before the search gives up.
const RUN_CAPACITY: u32 = 64;

impl Workspace<'_> {
    /// Record the physical groups of a body realization into `links`, two
    /// words per vertex.
    pub(super) fn record_weld(
        &mut self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        links: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let count = self.vertex_count;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (count).into());
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad.into(), (0u32).into());
        }
        parameters.insert("positions".into(), (positions.clone()).into());
        parameters.insert("keys".into(), (self.keys.clone()).into());
        parameters.insert("values".into(), (self.values.clone()).into());
        parameters.insert("quantized".into(), (self.quantized.clone()).into());
        let hash = gpu
            .cache()
            .get(gpu.context(), &hash_source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&hash, &parameters, (count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        self.sort
            .record(
                batch,
                &self.keys,
                &self.values,
                &mut self.sort_scratch,
                count.into(),
                32.into(),
            )
            .map_err(device_error)?;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (count).into());
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad.into(), (0u32).into());
        }
        parameters.insert("keys".into(), (self.keys.clone()).into());
        parameters.insert("values".into(), (self.values.clone()).into());
        parameters.insert("quantized".into(), (self.quantized.clone()).into());
        parameters.insert("links".into(), (links.clone()).into());
        parameters.insert("status".into(), (self.status.clone()).into());
        let link = gpu
            .cache()
            .get(gpu.context(), &link_source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&link, &parameters, (count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(())
    }
}

fn hash_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read_write> keys: array<u32>;
@group(0) @binding(2) var<storage, read_write> values: array<u32>;
@group(0) @binding(3) var<storage, read_write> quantized: array<i32>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
{precision}

// Rounded to the nearest integer, halves away from zero.
fn quantize(x: f32) -> i32 {{
    let scaled = x * WELD_PRECISION;
    let whole = trunc(scaled);
    if (abs(scaled - whole) >= 0.5) {{
        return i32(whole + sign(scaled));
    }}
    return i32(whole);
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let v = id.x;
    if (v >= params.count) {{
        return;
    }}
    var hash = 2166136261u;
    for (var axis = 0u; axis < 3u; axis = axis + 1u) {{
        let q = quantize(positions[v * 3u + axis]);
        quantized[v * 3u + axis] = q;
        hash = (hash ^ bitcast<u32>(q)) * 16777619u;
    }}
    keys[v] = hash;
    values[v] = v;
}}
"#,
        precision = wgsl::constant("WELD_PRECISION", WELD_PRECISION),
    ))
}

fn link_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read> values: array<u32>;
@group(0) @binding(2) var<storage, read> quantized: array<i32>;
@group(0) @binding(3) var<storage, read_write> links: array<u32>;
@group(0) @binding(4) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(5) var<uniform> params: Params;
{status}
{groups}
const RUN_CAPACITY: u32 = {run}u;

fn same(a: u32, b: u32) -> bool {{
    return quantized[a * 3u] == quantized[b * 3u]
        && quantized[a * 3u + 1u] == quantized[b * 3u + 1u]
        && quantized[a * 3u + 2u] == quantized[b * 3u + 2u];
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    if (slot >= params.count) {{
        return;
    }}
    let key = keys[slot];
    let v = values[slot];
    var start = slot;
    while (start > 0u && keys[start - 1u] == key) {{
        start = start - 1u;
        if (slot - start > RUN_CAPACITY) {{
            fail(STATUS_CROWDED_WELD);
            break;
        }}
    }}
    // Equal hashes keep ascending vertex order, so the first match is the
    // group's lowest vertex.
    var lowest = v;
    for (var other = start; other < slot; other = other + 1u) {{
        if (same(values[other], v)) {{
            lowest = values[other];
            break;
        }}
    }}
    var next = NO_VERTEX;
    for (var other = slot + 1u; other < params.count && keys[other] == key; other = other + 1u) {{
        if (other - slot > RUN_CAPACITY) {{
            fail(STATUS_CROWDED_WELD);
            break;
        }}
        if (same(values[other], v)) {{
            next = values[other];
            break;
        }}
    }}
    links[v * 2u] = lowest;
    links[v * 2u + 1u] = next;
}}
"#,
        status = wgsl::status(),
        groups = wgsl::GROUPS,
        run = RUN_CAPACITY,
    ))
}
