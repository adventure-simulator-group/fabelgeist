//! A finished part placed by a frame.
//!
//! Most parts are evaluated where they are worn: their shapes place every
//! carrier through the part frame, and the shell stage thickens them there.
//! A part whose features are much smaller than its distance from the origin
//! -- a visor's rounded slot corners, a fraction of a millimetre apart --
//! cannot be: world coordinates would round its carrier normals, and so its
//! inner walls, too coarsely. Such a part is evaluated and thickened in its
//! own frame, and this stage places its final vertices and hinges, rounding
//! every step with exact device arithmetic ([`host_float`]), before the final
//! normals are taken.

use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::part::HINGE_WORDS;
use super::{ArmorGpu, device_error, wgsl};
use crate::GenerateError;

/// Where a placed part goes, and what of it moves.
pub(crate) struct Placement<'a> {
    /// A frame buffer: origin, axes, half extents, as the part frames are.
    pub frame: &'a Buffer,
    pub positions: &'a Buffer,
    pub count: u32,
    pub hinges: &'a Buffer,
    pub hinge_count: u32,
    pub status: &'a Buffer,
}

/// Record the placement of a part's final vertices and hinges.
pub(crate) fn record(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    placement: &Placement,
) -> Result<(), GenerateError> {
    let mut parameters = PassParameters::new();
    parameters.insert("count", placement.count);
    parameters.insert("hinge_count", placement.hinge_count);
    parameters.insert("pad0", 0u32);
    parameters.insert("pad1", 0u32);
    parameters.insert(host_float::ZERO_FIELD, 0u32);
    for pad in ["pad2", "pad3", "pad4"] {
        parameters.insert(pad, 0.0f32);
    }
    parameters.insert("frames", placement.frame.clone());
    parameters.insert("positions", placement.positions.clone());
    parameters.insert("hinges", placement.hinges.clone());
    parameters.insert("status", placement.status.clone());
    batch
        .dispatch_items(
            &*kernel(gpu)?,
            &parameters,
            placement.count.max(placement.hinge_count),
        )
        .map_err(device_error)?;
    Ok(())
}

fn kernel(gpu: &ArmorGpu) -> Result<Arc<Kernel>, GenerateError> {
    let source = format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read_write> positions: array<f32>;
@group(0) @binding(2) var<storage, read_write> hinges: array<f32>;
@group(0) @binding(3) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    hinge_count: u32,
    pad0: u32,
    pad1: u32,
    zero: u32,
    pad2: f32,
    pad3: f32,
    pad4: f32,
}};
@group(0) @binding(4) var<uniform> params: Params;
{frame}
{zero_hook}{host_float}
{status_code}
{positions}

// A vector rotated by the frame's axes, as `PartFrame::point` sums them, each
// step exactly rounded.
fn rotated(f: Frame, v: vec3<f32>) -> vec3<f32> {{
    let m = mat3x3<f32>(f.x, f.y, f.z);
    var r: vec3<f32>;
    for (var axis = 0u; axis < 3u; axis = axis + 1u) {{
        r[axis] = host_add(
            host_add(host_mul(m[0][axis], v.x), host_mul(m[1][axis], v.y)),
            host_mul(m[2][axis], v.z),
        );
    }}
    return r;
}}

fn hinge_vector(slot: u32, offset: u32) -> vec3<f32> {{
    let at = slot * {hinge_words}u + offset;
    return vec3<f32>(hinges[at], hinges[at + 1u], hinges[at + 2u]);
}}

fn set_hinge_vector(slot: u32, offset: u32, v: vec3<f32>) {{
    let at = slot * {hinge_words}u + offset;
    hinges[at] = v.x;
    hinges[at + 1u] = v.y;
    hinges[at + 2u] = v.z;
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    let f = frame_at(0u);
    // A reflecting frame would also turn the triangles over.
    if (host_dot(host_cross(f.x, f.y), f.z) < 0.0) {{
        if (i == 0u) {{
            fail(STATUS_INVALID_SURFACE);
        }}
        return;
    }}
    if (i < params.count) {{
        positions_set(i, host_add3(f.origin, rotated(f, positions_at(i))));
    }}
    if (i < params.hinge_count) {{
        set_hinge_vector(i, 0u, host_add3(f.origin, rotated(f, hinge_vector(i, 0u))));
        set_hinge_vector(i, 4u, rotated(f, hinge_vector(i, 4u)));
    }}
}}
"#,
        frame = wgsl::FRAME,
        zero_hook = host_float::PARAMS_ZERO_HOOK,
        host_float = host_float::wgsl(),
        status_code = wgsl::STATUS,
        positions = wgsl::points("positions"),
        hinge_words = HINGE_WORDS,
    );
    gpu.cache()
        .get(gpu.context(), &source.as_str().into())
        .map_err(device_error)
}
