//! Plate charts evaluated on the device.
//!
//! A chart is a grid of rows along the plate and columns across it. The host
//! decides the columns -- evenly spaced, or aligned to a flute pattern's
//! crests and lands -- and the connectivity; a kernel built from a shape's
//! two WGSL functions evaluates every vertex:
//!
//! ```wgsl
//! fn chart_point(u: f32, v: f32) -> vec3<f32>   // local position
//! fn chart_offset(u: f32, axial: f32) -> f32    // height along the extrusion
//! ```
//!
//! Both may read the shape's `design` floats, the shell's `params.value*`,
//! and `fit`, the part frame. A shape also defines `chart_origin()`, the
//! local origin of its extrusion; [`AUTHORED_ORIGIN`] is the usual one. The
//! kernel adds the flute relief, places the point through the frame, and
//! writes the shell's extrusion geometry.

use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::part::{Extrusion, PartBuild, SHELL_WORDS, ShellSpec};
use super::{ArmorGpu, device_error};
use crate::{BoundaryNormals, GenerateError, PlateFluting};

mod chart_wgsl;

use chart_wgsl::source;
pub(crate) use chart_wgsl::{AUTHORED_ORIGIN, FLUTING};

/// Columns across an unfluted chart.
const AROUND: usize = 40;
/// Latitude rings closing a capped chart's tip, the tip vertex excluded.
const TIP_RINGS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ChartBoundary {
    Open,
    Cyclic,
    /// Closed by a rounded tip whose length follows the fitted frame:
    /// `per_half_height` of its half height, plus `fixed`.
    CappedByFrame {
        per_half_height: f32,
        fixed: f32,
    },
    /// The last row welded to one apex vertex.
    Apex,
}

/// Boundary codes the chart kernel reads.
const BOUNDARY_ROWS_ONLY: u32 = 0;
const BOUNDARY_CAPPED: u32 = 2;
const BOUNDARY_APEX: u32 = 3;

impl ChartBoundary {
    fn code(self) -> u32 {
        match self {
            Self::Open | Self::Cyclic => BOUNDARY_ROWS_ONLY,
            Self::CappedByFrame { .. } => BOUNDARY_CAPPED,
            Self::Apex => BOUNDARY_APEX,
        }
    }
}

/// One plate chart, as the host describes it.
#[derive(Clone, Debug)]
pub(crate) struct PlateChart {
    pub rows: usize,
    pub boundary: ChartBoundary,
    pub thickness: f32,
    pub extrusion: Extrusion,
    /// Extrusion origin and axis (or direction) in the chart's local frame.
    /// A capped chart's origin height is taken from its own first row.
    pub origin: [f32; 3],
    pub axis: [f32; 3],
    pub fluting: Option<PlateFluting>,
    pub span: [f32; 2],
    /// Shape constants that differ between the shells of one part.
    pub values: [f32; 4],
    /// Reflected across its local x before the part frame places it.
    pub mirrored: bool,
    /// Which of the part's frames places this chart.
    pub frame: usize,
}

impl PlateChart {
    fn columns(&self) -> Vec<f32> {
        let mut columns = self.fluting.map_or_else(
            || (0..=AROUND).map(|i| i as f32 / AROUND as f32).collect(),
            |pattern| pattern.columns(AROUND),
        );
        if self.boundary == ChartBoundary::Cyclic {
            columns.pop();
        }
        columns
    }

    fn last_row(&self) -> usize {
        if self.boundary == ChartBoundary::Apex {
            self.rows - 1
        } else {
            self.rows
        }
    }

    /// Carrier connectivity: two triangles per grid cell, closed around a
    /// cyclic chart, and the boundary's cap.
    fn indices(&self, stride: usize) -> Vec<u32> {
        let cyclic = self.boundary == ChartBoundary::Cyclic;
        let last_row = self.last_row();
        let segments = if cyclic { stride } else { stride - 1 };
        let mut indices = Vec::new();
        for row in 0..last_row {
            for col in 0..segments {
                let next = (col + 1) % stride;
                let [a, b, c, d] = [
                    row * stride + col,
                    row * stride + next,
                    (row + 1) * stride + col,
                    (row + 1) * stride + next,
                ]
                .map(|i| i as u32);
                indices.extend([a, b, d, a, d, c]);
            }
        }
        let grid = (last_row + 1) * stride;
        match self.boundary {
            ChartBoundary::Apex => {
                let tip = grid as u32;
                let start = grid - stride;
                for col in 0..stride - 1 {
                    let a = (start + col) as u32;
                    indices.extend([a, a + 1, tip]);
                }
            }
            ChartBoundary::CappedByFrame { .. } => {
                let mut previous: Vec<u32> = (0..stride as u32).collect();
                let mut next_index = grid as u32;
                for _ in 1..TIP_RINGS {
                    let next: Vec<u32> = (0..stride as u32).map(|c| next_index + c).collect();
                    next_index += stride as u32;
                    for col in 0..stride - 1 {
                        let [a, b, c, d] =
                            [previous[col], previous[col + 1], next[col], next[col + 1]];
                        indices.extend([a, d, b, a, c, d]);
                    }
                    previous = next;
                }
                let tip = next_index;
                for col in 0..stride - 1 {
                    indices.extend([tip, previous[col + 1], previous[col]]);
                }
            }
            ChartBoundary::Open | ChartBoundary::Cyclic => {}
        }
        indices
    }

    fn extra_vertices(&self, stride: usize) -> usize {
        match self.boundary {
            ChartBoundary::Apex => 1,
            ChartBoundary::CappedByFrame { .. } => (TIP_RINGS - 1) * stride + 1,
            ChartBoundary::Open | ChartBoundary::Cyclic => 0,
        }
    }

    /// The shell this chart becomes, and what its kernel needs to fill it.
    pub(crate) fn layout(&self) -> (ShellSpec, ChartSlots) {
        let columns = self.columns();
        let stride = columns.len();
        let grid = (self.last_row() + 1) * stride;
        let total = grid + self.extra_vertices(stride);
        (
            ShellSpec {
                carrier_count: total as u32,
                carrier_indices: self.indices(stride),
                boundary_normals: if self.fluting.is_some() {
                    BoundaryNormals::Separate
                } else {
                    BoundaryNormals::Smooth
                },
                thickness: self.thickness,
                extrusion: self.extrusion,
            },
            ChartSlots {
                columns,
                grid: grid as u32,
                total: total as u32,
            },
        )
    }
}

/// What the host computed about a chart's vertices.
#[derive(Clone, Debug)]
pub(crate) struct ChartSlots {
    columns: Vec<f32>,
    grid: u32,
    total: u32,
}

/// The flute pattern as eight floats: count, width, depth, spread, lower
/// spread, start, end, fade.
pub(crate) fn flute_words(fluting: Option<&PlateFluting>) -> [f32; 8] {
    fluting.map_or([0.0; 8], |f| {
        [
            f32::from(f.count.0),
            f.width.unit(),
            f.depth.metres(),
            f.spread.unit(),
            f.lower_spread.unit(),
            f.start.unit(),
            f.end.unit(),
            f.fade.unit(),
        ]
    })
}

/// A shape's chart kernel.
#[derive(Clone, Debug)]
pub(crate) struct ChartKernel(Arc<Kernel>);

impl ChartKernel {
    /// Compile the chart template around a shape's `chart_point` and
    /// `chart_offset`.
    pub(crate) fn new(gpu: &ArmorGpu, shape: &str) -> Result<Self, GenerateError> {
        Ok(Self(
            gpu.cache()
                .get(gpu.context(), &source(shape))
                .map_err(device_error)?,
        ))
    }
}

/// Everything a chart dispatch reads besides the chart itself.
pub(crate) struct ChartInputs<'a> {
    pub design: &'a Buffer,
    /// The part's frames, each a buffer starting with a frame's fifteen
    /// floats; a chart reads the one its `frame` names.
    pub frames: &'a [&'a Buffer],
}

/// Record the evaluation of one chart into its shell's carriers.
#[expect(
    clippy::too_many_arguments,
    reason = "a chart records against its part, its slots, its kernel and the part's inputs"
)]
pub(crate) fn record(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    build: &PartBuild,
    shell: usize,
    chart: &PlateChart,
    slots: &ChartSlots,
    kernel: &ChartKernel,
    inputs: &ChartInputs,
) -> Result<(), GenerateError> {
    let mut parameters = PassParameters::new();
    let unsigned = [
        ("first", build.layout().first_carrier(shell)),
        ("shell", shell as u32 * SHELL_WORDS),
        ("stride", slots.columns.len() as u32),
        ("rows", chart.rows as u32),
        ("boundary", chart.boundary.code()),
        ("grid", slots.grid),
        ("total", slots.total),
        ("fluted", u32::from(chart.fluting.is_some())),
        ("frame", 0),
        ("mirrored", u32::from(chart.mirrored)),
        ("pad1", 0),
        ("pad2", 0),
    ];
    for (name, value) in unsigned {
        parameters.insert(name, value);
    }
    let (tip_length, tip_per_half_height) = match chart.boundary {
        ChartBoundary::CappedByFrame {
            per_half_height,
            fixed,
        } => (fixed, per_half_height),
        _ => (0.0, 0.0),
    };
    let floats = [
        ("span0", chart.span[0]),
        ("span1", chart.span[1]),
        ("tip_length", tip_length),
        ("tip_per_half_height", tip_per_half_height),
        ("value0", chart.values[0]),
        ("value1", chart.values[1]),
        ("value2", chart.values[2]),
        ("value3", chart.values[3]),
        ("origin_x", chart.origin[0]),
        ("origin_y", chart.origin[1]),
        ("origin_z", chart.origin[2]),
        ("pad4", 0.0),
        ("axis_x", chart.axis[0]),
        ("axis_y", chart.axis[1]),
        ("axis_z", chart.axis[2]),
        ("pad5", 0.0),
    ];
    for (name, value) in floats {
        parameters.insert(name, value);
    }
    parameters.insert("columns", gpu.upload(&slots.columns)?);
    parameters.insert("flute", gpu.upload(&flute_words(chart.fluting.as_ref()))?);
    parameters.insert("design", inputs.design.clone());
    let frames = inputs
        .frames
        .get(chart.frame)
        .ok_or(GenerateError::InvalidSurface)?;
    parameters.insert("frames", (*frames).clone());
    parameters.insert("carriers", build.carriers.clone());
    parameters.insert("heights", build.heights.clone());
    parameters.insert("shells", build.shells.clone());
    batch
        .dispatch_items(&kernel.0, &parameters, slots.total)
        .map_err(device_error)?;
    Ok(())
}
