//! The garment slices a boot shaft clears, cut on the device.
//!
//! A narrow vertex band around a tilted ring only sees an arc, whose changing
//! bounds would corrugate the shaft, so every garment triangle edge is cut by
//! each section's plane instead. Each (section, edge) pair owns one slot of
//! the output: a crossing writes its point there, flagged in `w`, and a miss
//! leaves the slot unflagged. A closed garment's edges come in both
//! directions, so the crossings do not depend on its winding.

use anyhow::Result;
use fabelgeist_armor::DevicePart;
use fabelgeist_armor::gpu::wgsl;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use crate::device_foot_sections::PROFILE_SAMPLES;
use crate::device_frames::DeviceWearer;

/// Garment slices in the foot frame, flagged in `w`.
pub(crate) struct GarmentSlices {
    pub points: Buffer,
    pub count: u32,
}

impl DeviceWearer<'_> {
    /// Record the slices of `garments` by the planes of the sections between
    /// `bounds`; `hems` holds each garment's axial bounds, in order.
    pub(crate) fn record_garment_slices(
        &self,
        batch: &mut KernelBatch,
        frame: &Buffer,
        bounds: &Buffer,
        hems: &Buffer,
        garments: &[DevicePart],
    ) -> Result<GarmentSlices> {
        let gpu = self.gpu;
        let slots = |garment: &DevicePart| PROFILE_SAMPLES * garment.triangle_count() * 3;
        let count = garments.iter().map(slots).sum::<u32>();
        let points = gpu.scratch((count as u64 * 16).into(), ("boot layer slices").into())?;
        let kernel = gpu
            .cache()
            .get(gpu.context(), &source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        let mut first = 0;
        for (index, garment) in garments.iter().enumerate() {
            let mut parameters = PassParameters::new();
            parameters.insert("triangles".into(), (garment.triangle_count()).into());
            parameters.insert("garment".into(), (index as u32).into());
            parameters.insert("first".into(), (first).into());
            parameters.insert("pad0".into(), (0u32).into());
            parameters.insert("frames".into(), (frame.clone()).into());
            parameters.insert("bounds".into(), (bounds.clone()).into());
            parameters.insert("hems".into(), (hems.clone()).into());
            parameters.insert("positions".into(), (garment.positions().clone()).into());
            parameters.insert("indices".into(), (garment.indices().clone()).into());
            parameters.insert("points".into(), (points.clone()).into());
            batch
                .dispatch_items(&kernel, &parameters, (slots(garment)).into())
                .map_err(fabelgeist_armor::GenerateError::from)?;
            first += slots(garment);
        }
        Ok(GarmentSlices { points, count })
    }
}

fn source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> bounds: array<u32>;
@group(0) @binding(2) var<storage, read> hems: array<u32>;
@group(0) @binding(3) var<storage, read> positions: array<f32>;
@group(0) @binding(4) var<storage, read> indices: array<u32>;
@group(0) @binding(5) var<storage, read_write> points: array<vec4<f32>>;
struct Params {{
    triangles: u32,
    garment: u32,
    first: u32,
    pad0: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;
{math}
{frame}
{ordered}
{positions}

const SAMPLES: u32 = {samples}u;
const PROFILE_WINDOW_M: f32 = 0.008;

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {{
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}}

fn local_vertex(f: Frame, vertex: u32) -> vec3<f32> {{
    let d = positions_at(vertex) - f.origin;
    return vec3<f32>(host_dot(d, f.x), host_dot(d, f.y), host_dot(d, f.z));
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let slot = id.x;
    let edges = params.triangles * 3u;
    if (slot >= SAMPLES * edges) {{
        return;
    }}
    let section = slot / edges;
    let edge = slot % edges;
    let triangle = edge / 3u;
    let corner = edge % 3u;
    let fit = frame_at(0u);
    let low = float_from_ordered(bounds[0]);
    let high = float_from_ordered(bounds[1]);
    let height = low + (high - low) * f32(section) / f32(SAMPLES - 1u);
    // A complete low-shaft contour continues into the foot: fading its
    // radius below the hem would leave a projecting cuff shelf.
    let hem = float_from_ordered(hems[params.garment * 2u]);
    let plane = max(height, hem + PROFILE_WINDOW_M);
    let a = local_vertex(fit, indices[triangle * 3u + corner]);
    let b = local_vertex(fit, indices[triangle * 3u + (corner + 1u) % 3u]);
    var point = vec4<f32>(0.0);
    if ((a.y <= plane && b.y > plane) || (b.y <= plane && a.y > plane)) {{
        let t = (plane - a.y) / (b.y - a.y);
        point = vec4<f32>(a.x + (b.x - a.x) * t, height, a.z + (b.z - a.z) * t, 1.0);
    }}
    points[params.first + slot] = point;
}}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        ordered = wgsl::ORDERED_FLOAT,
        positions = wgsl::read_points("positions"),
        samples = PROFILE_SAMPLES,
    ))
}
