//! The kernels that thicken every shell of a part at once: rewinding
//! reflected shells, offsetting the relief and inner walls along each
//! shell's extrusion, and assembling the final vertices from the walls.

use fabelgeist_gpu::prelude::ShaderSource;
use std::sync::Arc;

use fabelgeist_compute::Kernel;

use super::super::shell_plan::INNER_BIT;
use super::super::{ArmorGpu, wgsl};
use super::{Extrusion, SHELL_MIRRORED, SHELL_WORDS};
use crate::GenerateError;

pub(super) struct ShellKernels {
    pub winding: Arc<Kernel>,
    pub walls: Arc<Kernel>,
    pub assemble: Arc<Kernel>,
}

impl ShellKernels {
    pub(super) fn get(gpu: &ArmorGpu) -> Result<Self, GenerateError> {
        Ok(Self {
            winding: gpu
                .cache()
                .get(gpu.context(), &winding_source())
                .map_err(crate::GenerateError::from)?,
            walls: gpu
                .cache()
                .get(gpu.context(), &walls_source())
                .map_err(crate::GenerateError::from)?,
            assemble: gpu
                .cache()
                .get(gpu.context(), &assemble_source())
                .map_err(crate::GenerateError::from)?,
        })
    }
}

/// Triangles of reflected shells turn over, so that they face outward again.
fn winding_source() -> ShaderSource {
    let counted = wgsl::COUNTED;
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> authored: array<u32>;
@group(0) @binding(1) var<storage, read> triangle_shells: array<u32>;
@group(0) @binding(2) var<storage, read> shells: array<f32>;
@group(0) @binding(3) var<storage, read_write> wound: array<u32>;
{counted}
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let t = id.x;
    if (t >= params.count) {{
        return;
    }}
    let flag = shells[triangle_shells[t] * {SHELL_WORDS}u + {SHELL_MIRRORED}u];
    let mirrored = bitcast<u32>(flag) != 0u;
    let a = authored[t * 3u];
    let b = authored[t * 3u + 1u];
    let c = authored[t * 3u + 2u];
    wound[t * 3u] = a;
    wound[t * 3u + 1u] = select(b, c, mirrored);
    wound[t * 3u + 2u] = select(c, b, mirrored);
}}
"#
    ))
}

/// How far a point moves per unit of thickness, by the shell's extrusion kind.
const EXTRUSION_OFFSET: &str = r#"
fn unit_axis(v: vec3<f32>) -> bool {
    return abs(((v.x * v.x + v.y * v.y) + v.z * v.z) - 1.0) <= 0.001;
}

// How far a point moves per unit of thickness, given its carrier normal and
// the shell's extrusion kind.
fn extrusion_offset(
    kind: u32,
    origin: vec3<f32>,
    axis: vec3<f32>,
    p: vec3<f32>,
    n: vec3<f32>,
) -> vec3<f32> {
    switch kind {
        case CAPPED_AXIS: {
            let delta = p - origin;
            let along = max(dot(delta, axis), 0.0);
            let radial = delta - axis * along;
            let projection = dot(radial, n);
            if (!unit_axis(axis) || !(projection > 1e-6)) {
                fail(STATUS_INVALID_SURFACE);
                return n;
            }
            return radial / projection;
        }
        case ALONG: {
            let projection = dot(axis, n);
            if (!unit_axis(axis) || !(projection > 1e-6)) {
                fail(STATUS_INVALID_SURFACE);
                return n;
            }
            return axis / projection;
        }
        case RADIAL: {
            let radial = p - origin;
            let projected = radial - axis * dot(radial, axis);
            let projection = dot(projected, n);
            if (!unit_axis(axis) || !(projection >= 1e-6)) {
                fail(STATUS_INVALID_SURFACE);
                return n;
            }
            return projected / projection;
        }
        default: {
            return n;
        }
    }
}

"#;

/// Relief along the extrusion, then the inner wall a gauge further in.
fn walls_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> carriers: array<f32>;
@group(0) @binding(1) var<storage, read> heights: array<f32>;
@group(0) @binding(2) var<storage, read> shell_of: array<u32>;
@group(0) @binding(3) var<storage, read> shells: array<f32>;
@group(0) @binding(4) var<storage, read> area_normals: array<f32>;
@group(0) @binding(5) var<storage, read> angle_normals: array<f32>;
@group(0) @binding(6) var<storage, read_write> walls: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
{counted}
@group(0) @binding(8) var<uniform> params: Params;
{math}
{status_code}
{carriers}
{area}
{angle}
{walls}

const ANGLE_WEIGHTED_NORMAL: u32 = {angle_weighted}u;
const CAPPED_AXIS: u32 = {capped_axis}u;
const ALONG: u32 = {along}u;
const RADIAL: u32 = {radial}u;

{extrusion_offset}
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let c = id.x;
    if (c >= params.count) {{
        return;
    }}
    let shell = shell_of[c] * {SHELL_WORDS}u;
    let kind = bitcast<u32>(shells[shell]);
    let thickness = shells[shell + 1u];
    let origin = vec3<f32>(shells[shell + 4u], shells[shell + 5u], shells[shell + 6u]);
    let axis = vec3<f32>(shells[shell + 8u], shells[shell + 9u], shells[shell + 10u]);
    var n = area_normals_at(c);
    if (kind == ANGLE_WEIGHTED_NORMAL) {{
        n = angle_normals_at(c);
    }}
    let carrier = carriers_at(c);
    let relief = carrier + extrusion_offset(kind, origin, axis, carrier, n) * heights[c];
    walls_set(c, relief);
    walls_set(params.count + c, relief - extrusion_offset(kind, origin, axis, relief, n) * thickness);
}}
"#,
        extrusion_offset = EXTRUSION_OFFSET,
        counted = wgsl::COUNTED,
        math = wgsl::MATH,
        status_code = wgsl::STATUS,
        carriers = wgsl::read_points("carriers"),
        area = wgsl::read_points("area_normals"),
        angle = wgsl::read_points("angle_normals"),
        walls = wgsl::points("walls"),
        angle_weighted = Extrusion::AngleWeightedNormal.code(),
        capped_axis = Extrusion::CappedAxis.code(),
        along = Extrusion::Along.code(),
        radial = Extrusion::Radial.code(),
    ))
}

/// Every final vertex is a copy of an outer or inner wall vertex.
fn assemble_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> sources: array<u32>;
@group(0) @binding(1) var<storage, read> walls: array<f32>;
@group(0) @binding(2) var<storage, read_write> positions: array<f32>;
struct Params {{
    count: u32,
    carriers: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
{walls}
{positions}
const INNER_BIT: u32 = {inner_bit}u;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let f = id.x;
    if (f >= params.count) {{
        return;
    }}
    let source = sources[f];
    let carrier = source & ~INNER_BIT;
    if ((source & INNER_BIT) != 0u) {{
        positions_set(f, walls_at(params.carriers + carrier));
    }} else {{
        positions_set(f, walls_at(carrier));
    }}
}}
"#,
        walls = wgsl::read_points("walls"),
        positions = wgsl::points("positions"),
        inner_bit = INNER_BIT,
    ))
}
