//! A cylindrical fabric chart in the garment's local carrier frame.
use super::{ArmorGpu, device_error, wgsl};
use crate::GenerateError;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

const BOUNDS: &str = r#"
struct Params { count: u32, pad0: u32, pad1: u32, pad2: u32 }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> carriers: array<f32>;
@group(0) @binding(2) var<storage, read_write> bounds: array<f32>;
@group(0) @binding(4) var<storage, read> frames: array<f32>;
@compute @workgroup_size(1)
fn main() {
    let fit = frame_at(0u);
    var bottom = frame_local(fit, vec3<f32>(carriers[0], carriers[1], carriers[2])).y;
    var top = bottom;
    for (var i = 1u; i < params.count; i++) {
        let y = frame_local(fit, vec3<f32>(carriers[i * 3u], carriers[i * 3u + 1u], carriers[i * 3u + 2u])).y;
        bottom = min(bottom, y);
        top = max(top, y);
    }
    bounds[0] = top; bounds[1] = top - bottom;
}
"#;
const COORDINATES: &str = r#"
struct Params { count: u32, pad0: u32, pad1: u32, pad2: u32 }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> carriers: array<f32>;
@group(0) @binding(2) var<storage, read> bounds: array<f32>;
@group(0) @binding(3) var<storage, read_write> coordinates: array<f32>;
@group(0) @binding(4) var<storage, read> frames: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x; if (i >= params.count) { return; }
    let p = frame_local(frame_at(0u), vec3<f32>(carriers[i * 3u], carriers[i * 3u + 1u], carriers[i * 3u + 2u]));
    // Rear seam, proximal top, distal hem/cuff. This local Y axis belongs to
    // the fitted garment frame, so arm charts do not depend on world Y.
    coordinates[i * 2u] = fract(atan2(p.x, p.z) / 6.28318530718 + 0.5);
    coordinates[i * 2u + 1u] = (bounds[0] - p.y) / bounds[1];
}
"#;

pub(super) fn record(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    carriers: &Buffer,
    count: u32,
    frame: &Buffer,
) -> Result<Buffer, GenerateError> {
    let bounds = gpu.scratch(8, "textile chart bounds")?;
    let coordinates = gpu.scratch(u64::from(count) * 8, "textile chart coordinates")?;
    let mut params = PassParameters::new();
    for (name, value) in [("count", count), ("pad0", 0), ("pad1", 0), ("pad2", 0)] {
        params.insert(name, value);
    }
    params.insert("carriers", carriers.clone());
    params.insert("bounds", bounds);
    params.insert("frames", frame.clone());
    let kernel = gpu
        .cache()
        .get(gpu.context(), &format!("{}{BOUNDS}", wgsl::FRAME))
        .map_err(device_error)?;
    batch
        .dispatch_items(&kernel, &params, 1)
        .map_err(device_error)?;
    params.insert("coordinates", coordinates.clone());
    let kernel = gpu
        .cache()
        .get(gpu.context(), &format!("{}{COORDINATES}", wgsl::FRAME))
        .map_err(device_error)?;
    batch
        .dispatch_items(&kernel, &params, count)
        .map_err(device_error)?;
    Ok(coordinates)
}
