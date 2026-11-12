//! Pattern-cut garments and articulated waist plates on the device: the
//! carriers of tubes, faulds and tassets.
//!
//! A tube and the waist plates are authored in the garment's frame and fitted
//! afterwards by moving their carriers. The torso's panels are placed by the
//! fit alone, so its shape is the fitter's: this module owns its topology and
//! the fitter supplies the WGSL that places every vertex.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::chart::{AUTHORED_ORIGIN, ChartBoundary, ChartKernel, PlateChart};
use super::coord::{AUTHORED, CoordExtrusion, CoordKernel, CoordShell};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::garment_armor::{
    GARMENT_ARMPIT_ROW as ARMPIT_ROW, GARMENT_AXIAL_SEGMENTS as ALONG,
    GARMENT_PANEL_ACROSS as PANEL_ACROSS, GARMENT_PANEL_ALONG as PANEL_ALONG,
    GARMENT_RING_SEGMENTS as AROUND, GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER_DEPTH,
};
use crate::{
    BoundaryNormals, GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape, GenerateError,
};

/// Lames overlap the next one down by this fraction of their pitch.
const LAME_OVERLAP: f32 = 0.12;
/// Rows along a fauld lame and a tasset lame.
const FAULD_ROWS: usize = 16;
const TASSET_ROWS: usize = 8;

/// A padded garment's quilting: a shallow relief of twelve ridges.
const QUILT: &str = r#"
const QUILT_RELIEF_METRES: f32 = 0.0015;

fn quilt(padded: bool, coordinate: f32) -> f32 {
    if (padded) {
        return QUILT_RELIEF_METRES * pow2(cos(coordinate * PI * 12.0));
    }
    return 0.0;
}
"#;

/// Rings of columns down a sleeve, chausse or skirt; `coord` is the column
/// and row.
const TUBE: &str = r#"
const SKIRT: u32 = 0u;
const PADDED: u32 = 1u;
const FLARE: u32 = 2u;
const LENGTH: u32 = 3u;
const CLEARANCE: u32 = 4u;
const WALL: u32 = 5u;

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let axial = coord.y / f32(ALONG);
    var radius = 0.70 + 0.30 * axial;
    if (design[SKIRT] != 0.0) {
        radius = 1.0 + design[FLARE] * (1.0 - axial);
    }
    let u = coord.x / f32(AROUND);
    let angle = u * TAU;
    let padding = design[CLEARANCE] + design[WALL] + quilt(design[PADDED] != 0.0, u);
    return ShellVertex(
        vec3<f32>(
            (width * radius + padding) * sin(angle),
            height - 2.0 * height * design[LENGTH] * (1.0 - axial),
            (depth * radius + padding) * cos(angle),
        ),
        0.0,
    );
}
"#;

/// One fauld lame; `value0` and `value1` are its axial span.
const FAULD: &str = r#"
const FLARE: u32 = 0u;
const LENGTH: u32 = 1u;
const FRONT_ARCH: u32 = 2u;
const PADDING: u32 = 3u;
const LAME_STEP: u32 = 4u;

fn chart_offset(u: f32, axial: f32) -> f32 {
    return 0.0;
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let axial = params.value0 + (params.value1 - params.value0) * v;
    let angle = (u - 0.5) * TAU;
    let radius = 1.0 + design[FLARE] * (1.0 - axial);
    let padding = design[PADDING] + design[LAME_STEP] * (1.0 - v);
    return vec3<f32>(
        (width * radius + padding) * sin(angle),
        height - 2.0 * height * design[LENGTH] * (1.0 - axial)
            + height * design[FRONT_ARCH] * pow4(max(cos(angle), 0.0)) * pow4(1.0 - axial),
        (depth * radius + padding) * cos(angle),
    );
}
"#;

/// One tasset lame; `value0` is the side, `value1` and `value2` the axial
/// span, and `value3` whether it is the lowest lame.
const TASSET: &str = r#"
const INNER_CUTAWAY: u32 = 0u;
const HEM_POINT: u32 = 1u;
const SCALE: u32 = 2u;
const GAP: u32 = 3u;
const HEM_ROUNDNESS: u32 = 4u;
const LENGTH: u32 = 5u;
const FLARE: u32 = 6u;
const CLEARANCE: u32 = 7u;
const GAUGE: u32 = 8u;

fn chart_offset(u: f32, axial: f32) -> f32 {
    let bottom = params.value1;
    let top = params.value2;
    return design[GAUGE] * 3.0 * (top - axial) / (top - bottom);
}

fn chart_point(column: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let side = params.value0;
    let bottom = params.value1;
    let top = params.value2;
    let half_width = width * 0.44 * design[SCALE];
    let u = 2.0 * column - 1.0;
    let descent = 1.0 - (bottom + (top - bottom) * v);
    let inner = pow3((1.0 - side * u) * 0.5);
    let upper_cutaway = design[INNER_CUTAWAY] * inner;
    let span = (1.0 - 2.0 * upper_cutaway) * (1.0 - design[HEM_ROUNDNESS] * pow8(u));
    let shaped_descent = upper_cutaway + span * descent;
    var point_drop = 0.0;
    if (params.value3 != 0.0) {
        point_drop = design[HEM_POINT] * max(1.0 - abs(u - side * 0.35) / 1.35, 0.0) * (1.0 - v);
    }
    // Taper follows the trimmed height, preserving the planar chart's
    // orientation even at a deep inner cutaway.
    let taper = 1.0 - 0.14 * (shaped_descent + point_drop);
    return vec3<f32>(
        side * (half_width + width * design[GAP] * 0.5) + half_width * taper * u,
        height - 2.0 * height * design[LENGTH] * (shaped_descent + point_drop),
        depth * (1.0 + design[FLARE] * descent) + design[CLEARANCE] + design[GAUGE]
            + width * 0.18 * (1.0 - u * u),
    );
}
"#;

fn padded(kind: GarmentArmorKind) -> bool {
    matches!(
        kind,
        GarmentArmorKind::ArmingDoublet
            | GarmentArmorKind::PaddedChausses
            | GarmentArmorKind::PaddedSkirt
            | GarmentArmorKind::QuiltedSleeve
    )
}

pub(super) fn constants() -> String {
    format!(
        "const ALONG: u32 = {ALONG}u;\nconst AROUND: u32 = {AROUND}u;\n\
         const PANEL_ACROSS: u32 = {PANEL_ACROSS}u;\nconst PANEL_ALONG: u32 = {PANEL_ALONG}u;\n\
         const ARMPIT_ROW: u32 = {ARMPIT_ROW}u;\nconst SHOULDER_DEPTH: u32 = {SHOULDER_DEPTH}u;\n"
    )
}

pub(super) fn coord_shell(
    coords: Vec<[f32; 4]>,
    indices: Vec<u32>,
    thickness: f32,
    passes: u32,
) -> CoordShell {
    CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness,
        extrusion: CoordExtrusion::Normal,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
        passes,
        hinge: None,
    }
}

/// Record a sleeve, chausse or skirt tube, authored in the part frame at the
/// start of `frame`.
pub fn record_garment_tube(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &GarmentArmorDesign,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let skirt = matches!(
        design.kind,
        GarmentArmorKind::MailSkirt | GarmentArmorKind::PaddedSkirt
    );
    let mut coords = Vec::with_capacity((ALONG + 1) * AROUND);
    for row in 0..=ALONG {
        for col in 0..AROUND {
            coords.push([col as f32, row as f32, 0.0, 0.0]);
        }
    }
    let mut indices = Vec::with_capacity(ALONG * AROUND * 6);
    for row in 0..ALONG {
        for col in 0..AROUND {
            let a = (row * AROUND + col) as u32;
            let b = (row * AROUND + (col + 1) % AROUND) as u32;
            let (c, d) = (b + AROUND as u32, a + AROUND as u32);
            indices.extend([a, b, c, a, c, d]);
        }
    }
    let kernel = CoordKernel::new(gpu, &format!("{}{AUTHORED}{QUILT}{TUBE}", constants()))?;
    let mut part = PartRecipe::new();
    part.push_coord(
        coord_shell(coords, indices, design.wall_thickness.metres(), 1),
        kernel,
    )?;
    let flag = |b: bool| if b { 1.0 } else { 0.0 };
    part.record(
        gpu,
        batch,
        &[
            flag(skirt),
            flag(padded(design.kind)),
            design.flare.unit(),
            design.length.unit(),
            design.clearance.metres(),
            design.wall_thickness.metres(),
        ],
        &[frame],
    )
}

fn lame_chart(
    rows: usize,
    boundary: ChartBoundary,
    design: &GarmentArmorDesign,
    extrusion: Extrusion,
    axis: [f32; 3],
    span: [f32; 2],
    values: [f32; 4],
) -> PlateChart {
    PlateChart {
        rows,
        boundary,
        thickness: design.wall_thickness.metres(),
        extrusion,
        origin: [0.0; 3],
        axis,
        fluting: design.fluting,
        span,
        values,
        mirrored: false,
        frame: 0,
        around: super::chart::AROUND,
    }
}

/// Record a fauld's overlapping radial lames, authored in the part frame at
/// the start of `frame`.
pub fn record_fauld(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &GarmentArmorDesign,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let GarmentPlateShape::Fauld { front_arch, .. } = design.plate_shape else {
        return Err(GenerateError::InvalidSurface);
    };
    const LAME_SPACING_GAUGES: f32 = 2.5;
    let gauge = design.wall_thickness.metres();
    let count = usize::from(design.lame_count);
    let kernel = ChartKernel::new(gpu, &format!("{AUTHORED_ORIGIN}{FAULD}"))?;
    let mut part = PartRecipe::new();
    for lame in 0..count {
        let top = 1.0 - lame as f32 / count as f32;
        let bottom = (1.0
            - (lame + 1) as f32 / count as f32
            - if lame + 1 < count {
                LAME_OVERLAP / count as f32
            } else {
                0.0
            })
        .max(0.0);
        part.push_chart(
            lame_chart(
                FAULD_ROWS,
                ChartBoundary::Cyclic,
                design,
                Extrusion::Radial,
                [0.0, 1.0, 0.0],
                [bottom, top],
                [bottom, top, 0.0, 0.0],
            ),
            kernel.clone(),
        )?;
    }
    let lame_step = if count > 1 {
        gauge * LAME_SPACING_GAUGES
    } else {
        0.0
    };
    part.record(
        gpu,
        batch,
        &[
            design.flare.unit(),
            design.length.unit(),
            front_arch.unit(),
            design.clearance.metres() + gauge,
            lame_step,
        ],
        &[frame],
    )
}

/// Record both tassets' lames, authored in the part frame at the start of
/// `frame`.
pub fn record_tassets(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &GarmentArmorDesign,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let GarmentPlateShape::Tassets {
        inner_cutaway,
        hem_point,
        width,
        gap,
        hem_roundness,
    } = design.plate_shape
    else {
        return Err(GenerateError::InvalidSurface);
    };
    let count = usize::from(design.lame_count);
    let kernel = ChartKernel::new(gpu, &format!("{AUTHORED_ORIGIN}{TASSET}"))?;
    let mut part = PartRecipe::new();
    for side in [-1.0f32, 1.0] {
        for lame in 0..count {
            let top = 1.0 - lame as f32 / count as f32;
            let bottom = 1.0
                - (lame as f32 + 1.0 + if lame + 1 < count { LAME_OVERLAP } else { 0.0 })
                    / count as f32;
            let last = if lame + 1 == count { 1.0 } else { 0.0 };
            part.push_chart(
                lame_chart(
                    TASSET_ROWS,
                    ChartBoundary::Open,
                    design,
                    Extrusion::Along,
                    [0.0, 0.0, 1.0],
                    [bottom, top],
                    [side, bottom, top, last],
                ),
                kernel.clone(),
            )?;
        }
    }
    part.record(
        gpu,
        batch,
        &[
            inner_cutaway.unit(),
            hem_point.unit(),
            width.unit(),
            gap.unit(),
            hem_roundness.unit(),
            design.length.unit(),
            design.flare.unit(),
            design.clearance.metres(),
            design.wall_thickness.metres(),
        ],
        &[frame],
    )
}
