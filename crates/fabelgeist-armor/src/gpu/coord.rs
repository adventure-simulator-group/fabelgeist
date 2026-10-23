//! Shells of any topology, evaluated vertex by vertex on the device.
//!
//! Where a plate is not a grid of rows and columns -- a dome's pole and
//! rings, a skirt hung from part of a rim, a torso's panels joined by seams,
//! a visor pierced by a triangulated domain -- the host decides every
//! vertex and triangle from the design. It gives each vertex four floats of
//! coordinates whose meaning the shape defines, and the shape's WGSL turns
//! them into a point:
//!
//! ```wgsl
//! fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex
//! ```
//!
//! `index` is the vertex's index within the shell. A vertex that depends on
//! others -- a seam point between two panels, a cap ring gathered from a
//! rim -- belongs to a later pass: `shell_pass(index, coord)` names it, and
//! the kernel runs once per pass, so a later pass can read what an earlier one
//! wrote through `local_carrier(index)`.

use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::part::{Extrusion, HINGE_WORDS, PartBuild, SHELL_MIRRORED, SHELL_WORDS, ShellSpec};
use super::{ArmorGpu, device_error, wgsl};
use crate::{BoundaryNormals, GenerateError};

/// One shell whose vertices the host lists with their coordinates.
#[derive(Clone, Debug)]
pub struct CoordShell {
    pub coords: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub boundary_normals: BoundaryNormals,
    pub thickness: f32,
    pub extrusion: CoordExtrusion,
    /// Shape constants that differ between the shells of one part.
    pub values: [f32; 4],
    /// Reflected across its local x before the part frame places it.
    pub mirrored: bool,
    /// Which of the part's frames places this shell.
    pub frame: usize,
    /// Passes the shell's vertices are evaluated in.
    pub passes: u32,
    /// Where this shell's `shell_hinge()` is written, if it carries one.
    pub hinge: Option<super::part::HingeSlot>,
}

/// How a coordinate shell is thickened; origins and axes are local and may
/// be overridden by the shape's `shell_origin()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordExtrusion {
    Normal,
    AngleWeightedNormal,
    /// Per-carrier geometric or fitted-vector extrusion, supplied by the kernel.
    FieldNormal,
    FieldAngleWeightedNormal,
    Radial {
        origin: [f32; 3],
        axis: [f32; 3],
    },
}

impl CoordExtrusion {
    fn parts(self) -> (Extrusion, [f32; 3], [f32; 3]) {
        match self {
            Self::Normal => (Extrusion::Normal, [0.0; 3], [0.0, 1.0, 0.0]),
            Self::AngleWeightedNormal => {
                (Extrusion::AngleWeightedNormal, [0.0; 3], [0.0, 1.0, 0.0])
            }
            Self::FieldNormal => (Extrusion::FieldNormal, [0.; 3], [0., 1., 0.]),
            Self::FieldAngleWeightedNormal => {
                (Extrusion::FieldAngleWeightedNormal, [0.; 3], [0., 1., 0.])
            }
            Self::Radial { origin, axis } => (Extrusion::Radial, origin, axis),
        }
    }
}

impl CoordShell {
    pub(crate) fn spec(&self) -> ShellSpec {
        ShellSpec {
            carrier_count: self.coords.len() as u32,
            carrier_indices: self.indices.clone(),
            boundary_normals: self.boundary_normals,
            thickness: self.thickness,
            extrusion: self.extrusion.parts().0,
            grid: None,
        }
    }
}

/// A shape's coordinate-shell kernel.
#[derive(Clone, Debug)]
pub struct CoordKernel {
    kernel: Arc<Kernel>,
    extrusion: KernelExtrusion,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum KernelExtrusion {
    Geometric,
    Field,
}

impl CoordKernel {
    /// Compile the coordinate template around a shape's WGSL, which defines
    /// `shell_vertex`, `shell_pass` and `shell_origin`.
    pub fn new(gpu: &ArmorGpu, shape: &str) -> Result<Self, GenerateError> {
        Self::compile(gpu, shape, GEOMETRIC_EXTRUSION, KernelExtrusion::Geometric)
    }

    /// `extrusion` defines `shell_extrusion(index, coord) -> ShellExtrusion`.
    /// Its tagged result chooses geometric thickness or the exact fitted
    /// displacement vector used by the shape to place that carrier point.
    pub fn with_extrusion(
        gpu: &ArmorGpu,
        shape: &str,
        extrusion: &str,
    ) -> Result<Self, GenerateError> {
        Self::compile(gpu, shape, extrusion, KernelExtrusion::Field)
    }

    fn compile(
        gpu: &ArmorGpu,
        shape: &str,
        field: &str,
        extrusion: KernelExtrusion,
    ) -> Result<Self, GenerateError> {
        Ok(Self {
            kernel: gpu
                .cache()
                .get(gpu.context(), &source(shape, field))
                .map_err(device_error)?,
            extrusion,
        })
    }
}

/// A shape whose extrusion origin is the one its host description gives,
/// and whose vertices are all evaluated in the first pass.
pub const AUTHORED: &str = r#"
fn shell_origin() -> vec3<f32> {
    return vec3<f32>(params.origin_x, params.origin_y, params.origin_z);
}

fn shell_pass(index: u32, coord: vec4<f32>) -> u32 {
    return 0u;
}

fn shell_hinge() -> array<vec3<f32>, 2> {
    return array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
}
"#;

/// Record every pass of one coordinate shell into its carriers.
#[expect(
    clippy::too_many_arguments,
    reason = "a shell records against its part, its uploads and the part's shared inputs"
)]
pub(crate) fn record(
    batch: &mut KernelBatch,
    build: &PartBuild,
    shell: usize,
    coord_shell: &CoordShell,
    coords: &Buffer,
    kernel: &CoordKernel,
    design: &Buffer,
    frames: &[&Buffer],
) -> Result<(), GenerateError> {
    let (_, origin, axis) = coord_shell.extrusion.parts();
    if matches!(
        coord_shell.extrusion,
        CoordExtrusion::FieldNormal | CoordExtrusion::FieldAngleWeightedNormal
    ) && kernel.extrusion != KernelExtrusion::Field
    {
        return Err(GenerateError::InvalidSurface);
    }
    let frame = frames
        .get(coord_shell.frame)
        .ok_or(GenerateError::InvalidSurface)?;
    for pass in 0..coord_shell.passes.max(1) {
        let mut parameters = PassParameters::new();
        for (name, value) in [
            ("first", build.layout().first_carrier(shell)),
            ("shell", shell as u32 * SHELL_WORDS),
            ("total", coord_shell.coords.len() as u32),
            ("stage", pass),
            ("mirrored", u32::from(coord_shell.mirrored)),
            ("hinge", coord_shell.hinge.map_or(NO_HINGE, |slot| slot.0)),
            ("carrier_capacity", build.layout().total_carriers()),
            ("pad2", 0),
        ] {
            parameters.insert(name, value);
        }
        for (name, value) in [
            ("value0", coord_shell.values[0]),
            ("value1", coord_shell.values[1]),
            ("value2", coord_shell.values[2]),
            ("value3", coord_shell.values[3]),
            ("origin_x", origin[0]),
            ("origin_y", origin[1]),
            ("origin_z", origin[2]),
            ("pad3", 0.0),
            ("axis_x", axis[0]),
            ("axis_y", axis[1]),
            ("axis_z", axis[2]),
            ("pad4", 0.0),
        ] {
            parameters.insert(name, value);
        }
        parameters.insert("coords", coords.clone());
        parameters.insert("design", design.clone());
        parameters.insert("frames", (*frame).clone());
        parameters.insert("carriers", build.carriers.clone());
        parameters.insert("heights", build.heights.clone());
        parameters.insert("shells", build.shells.clone());
        parameters.insert("hinges", build.hinges.clone());
        batch
            .dispatch_items(&kernel.kernel, &parameters, coord_shell.coords.len() as u32)
            .map_err(device_error)?;
    }
    Ok(())
}

/// The vertex a shape computes, in its local frame.
const VERTEX: &str = r#"
struct ShellVertex {
    point: vec3<f32>,
    height: f32,
};
const GEOMETRIC_SHELL_EXTRUSION: u32 = 0u;
const FITTED_SHELL_EXTRUSION: u32 = 1u;
struct ShellExtrusion { kind: u32, direction: vec3<f32> };
"#;

/// The coordinate kernel's bindings and parameters.
const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> coords: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> design: array<f32>;
@group(0) @binding(2) var<storage, read> frames: array<f32>;
@group(0) @binding(3) var<storage, read_write> carriers: array<f32>;
@group(0) @binding(4) var<storage, read_write> heights: array<f32>;
@group(0) @binding(5) var<storage, read_write> shells: array<f32>;
@group(0) @binding(7) var<storage, read_write> hinges: array<f32>;
struct Params {
    first: u32,
    shell: u32,
    total: u32,
    stage: u32,
    mirrored: u32,
    hinge: u32,
    carrier_capacity: u32,
    pad2: u32,
    value0: f32,
    value1: f32,
    value2: f32,
    value3: f32,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    pad3: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    pad4: f32,
};
@group(0) @binding(6) var<uniform> params: Params;
"#;

/// A shell's hinge slot when it carries no hinge.
const NO_HINGE: u32 = u32::MAX;

const GEOMETRIC_EXTRUSION: &str = r#"
fn shell_extrusion(index: u32, coord: vec4<f32>) -> ShellExtrusion {
    return ShellExtrusion(GEOMETRIC_SHELL_EXTRUSION, vec3<f32>(0.0));
}
"#;

fn source(shape: &str, extrusion: &str) -> String {
    format!(
        r#"
{BINDINGS}{math}
{frame}
{carriers}
{vertex}

var<private> fit: Frame;

// A local point placed through the shell's own reflection, then the frame.
fn placed(local: vec3<f32>) -> vec3<f32> {{
    var p = local;
    if (params.mirrored != 0u) {{
        p.x = -p.x;
    }}
    return frame_point(fit, p);
}}

fn placed_vector(local: vec3<f32>) -> vec3<f32> {{
    var v = local;
    if (params.mirrored != 0u) {{
        v.x = -v.x;
    }}
    return frame_vector(fit, v);
}}

// A carrier of this shell written in an earlier pass, back in local terms.
fn local_carrier(index: u32) -> vec3<f32> {{
    var p = frame_local(fit, carriers_at(params.first + index));
    if (params.mirrored != 0u) {{
        p.x = -p.x;
    }}
    return p;
}}

{shape}
{extrusion}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.total) {{
        return;
    }}
    fit = frame_at(0u);
    let coord = coords[i];
    if (shell_pass(i, coord) == params.stage) {{
        let vertex = shell_vertex(i, coord);
        carriers_set(params.first + i, placed(vertex.point));
        heights[params.first + i] = vertex.height;
        let field = shell_extrusion(i, coord);
        let direction = placed_vector(field.direction);
        let attributes = params.carrier_capacity + (params.first + i) * 4u;
        heights[attributes] = bitcast<f32>(field.kind);
        heights[attributes + 1u] = direction.x;
        heights[attributes + 2u] = direction.y;
        heights[attributes + 3u] = direction.z;
    }}
    if (i == 0u && params.stage == 0u) {{
        let origin = placed(shell_origin());
        let axis = placed_vector(vec3<f32>(params.axis_x, params.axis_y, params.axis_z));
        for (var k = 0u; k < 3u; k = k + 1u) {{
            shells[params.shell + 4u + k] = origin[k];
            shells[params.shell + 8u + k] = axis[k];
        }}
        let reflects = dot(cross(fit.x, fit.y), fit.z) < 0.0;
        let mirrored = reflects != (params.mirrored != 0u);
        shells[params.shell + {mirrored_word}u] = bitcast<f32>(select(0u, 1u, mirrored));
        if (params.hinge != {no_hinge}u) {{
            let hinge = shell_hinge();
            let hinge_origin = placed(hinge[0]);
            let hinge_axis = placed_vector(hinge[1]);
            for (var k = 0u; k < 3u; k = k + 1u) {{
                hinges[params.hinge * {hinge_words}u + k] = hinge_origin[k];
                hinges[params.hinge * {hinge_words}u + 4u + k] = hinge_axis[k];
            }}
        }}
    }}
}}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        carriers = wgsl::points("carriers"),
        vertex = VERTEX,
        mirrored_word = SHELL_MIRRORED,
        no_hinge = NO_HINGE,
        hinge_words = HINGE_WORDS,
    )
}
