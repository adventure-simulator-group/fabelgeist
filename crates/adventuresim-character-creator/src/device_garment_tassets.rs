//! Tassets seated on the device: a smooth depth datum is sampled from the
//! front of the hips, and every carrier keeps its authored offset above it, so
//! neighbouring lames stay lapped on one surface.

use adventuresim_armor_model::DevicePart;
use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_garment_skirt::{DEPTH_SIDE, layout};

impl DeviceWearer<'_> {
    /// Record the tasset seating: the depth datum, then every carrier seated
    /// on it.
    pub(crate) fn record_tasset_fit(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        support: &Buffer,
        part: &DevicePart,
    ) -> Result<()> {
        dispatch(
            self,
            batch,
            &format!("{}{TASSET_DEPTH}", layout()),
            &[
                read("positions", &self.body.positions),
                read("normals", &self.body.normals),
                read_u32("support", support),
                write("fit", fit),
            ],
            &[],
            Grid::Items(DEPTH_SIDE * DEPTH_SIDE),
        )?;
        dispatch(
            self,
            batch,
            &format!("{}{TASSET_FIT}", layout()),
            &[read("fit", fit), write("carriers", part.carriers())],
            &[Word::U("count", part.carrier_count())],
            Grid::Items(part.carrier_count()),
        )
    }
}

/// One invocation per datum sample: the inverse-distance mean depth of the
/// twelve nearest forward-facing skin samples.
const TASSET_DEPTH: &str = r#"
const SUPPORT_MASK: u32 = 1u;
const SURFACE_SAMPLE_COUNT: u32 = 12u;
const FACING: f32 = 0.12;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= DEPTH_SIDE * DEPTH_SIDE) {
        return;
    }
    let f = frame_at(0u);
    let e = f.half_extents;
    let row = index / DEPTH_SIDE;
    let column = index % DEPTH_SIDE;
    let x = (2.0 * f32(column) / f32(DEPTH_SIDE - 1u) - 1.0) * e.x * 1.2;
    let y = (2.0 * f32(row) / f32(DEPTH_SIDE - 1u) - 1.0) * e.y * 1.2;
    var distances: array<f32, SURFACE_SAMPLE_COUNT>;
    var values: array<f32, SURFACE_SAMPLE_COUNT>;
    var kept = 0u;
    let count = arrayLength(&support);
    for (var v = 0u; v < count; v = v + 1u) {
        if ((support[v] & SUPPORT_MASK) == 0u || !(host_dot(f.z, normals_at(v)) > FACING)) {
            continue;
        }
        let p = host_local(f, positions_at(v));
        let distance = pow2(p.x - x) + pow2(p.y - y);
        if (kept == SURFACE_SAMPLE_COUNT && !(distance < distances[kept - 1u])) {
            continue;
        }
        // Insert after every kept sample no farther away.
        var j = min(kept, SURFACE_SAMPLE_COUNT - 1u);
        loop {
            if (j == 0u || !(distance < distances[j - 1u])) {
                break;
            }
            distances[j] = distances[j - 1u];
            values[j] = values[j - 1u];
            j = j - 1u;
        }
        distances[j] = distance;
        values[j] = p.z;
        kept = min(kept + 1u, SURFACE_SAMPLE_COUNT);
    }
    var sum = 0.0;
    var weight = 0.0;
    for (var k = 0u; k < kept; k = k + 1u) {
        let influence = 1.0 / (distances[k] + 0.0001);
        sum = sum + values[k] * influence;
        weight = weight + influence;
    }
    var depth = e.z;
    if (weight > 0.0) {
        depth = sum / weight;
    }
    fit[DEPTH + index] = depth;
}
"#;

/// Every carrier seated on the datum's cubic B-spline depth beneath it,
/// keeping its authored offset.
const TASSET_FIT: &str = r#"
fn weights(t: f32) -> vec4<f32> {
    return vec4<f32>(
        pow3(1.0 - t) / 6.0,
        (3.0 * pow3(t) - 6.0 * t * t + 4.0) / 6.0,
        (-3.0 * pow3(t) + 3.0 * t * t + 3.0 * t + 1.0) / 6.0,
        pow3(t) / 6.0,
    );
}

fn depth_at(p: vec3<f32>, e: vec3<f32>) -> f32 {
    let last = f32(DEPTH_SIDE - 1u);
    let px = clamp((p.x / (e.x * 1.2) + 1.0) * 0.5 * last, 0.0, last);
    let py = clamp((p.y / (e.y * 1.2) + 1.0) * 0.5 * last, 0.0, last);
    let bx = i32(floor(px));
    let by = i32(floor(py));
    let wx = weights(px - f32(bx));
    let wy = weights(py - f32(by));
    var depth = 0.0;
    for (var j = 0; j < 4; j = j + 1) {
        for (var i = 0; i < 4; i = i + 1) {
            let column = u32(clamp(bx + i - 1, 0, i32(DEPTH_SIDE) - 1));
            let row = u32(clamp(by + j - 1, 0, i32(DEPTH_SIDE) - 1));
            depth = depth + wx[i] * wy[j] * fit[DEPTH + row * DEPTH_SIDE + column];
        }
    }
    return depth;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let f = frame_at(0u);
    var local = host_local(f, carriers_at(i));
    let authored_offset = local.z - f.half_extents.z;
    local.z = depth_at(local, f.half_extents) + authored_offset;
    carriers_set(i, host_point(f, local));
}
"#;
