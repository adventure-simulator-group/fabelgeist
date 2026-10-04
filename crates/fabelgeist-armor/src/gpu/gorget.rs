//! Gorget plates on the device: a shoulder bib and overlapping collar lames.
//!
//! The collar and bib surfaces belong to whoever fits them, so the caller
//! supplies them as WGSL:
//!
//! ```wgsl
//! fn gorget_collar(t: f32, angle: f32) -> vec3<f32>  // local point
//! fn gorget_bib(t: f32, angle: f32) -> vec3<f32>     // local point
//! fn gorget_center() -> vec2<f32>                    // local x and z
//! ```
//!
//! Both read the fit from `frames`, whose first fifteen floats are the part
//! frame. This module owns the rest: the column chart, rows, flute relief,
//! and which surface each row samples.

use std::f32::consts::TAU;

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::chart::{FLUTING, flute_words};
use super::coord::{CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::{
    BoundaryNormals, GarmentArmorDesign, GarmentPlateShape, GenerateError, PlateFluting,
    gorget_control_angle,
};

/// Columns around the neck before the rear sweep and flutes add theirs.
const AROUND: usize = 64;
/// Rows down the bib, and down each collar lame.
const BIB_ROWS: usize = 24;
const LAME_ROWS: usize = 8;
/// A collar lame laps the one above it by this fraction of its pitch.
const LAP_FRACTION: f32 = 0.12;
const MERGE_TOLERANCE: f32 = 1e-6;

/// Where the flute pattern's eight floats start in the design floats.
const FLUTE_WORDS_AT: usize = 2;

/// `value0` is 0 for the bib and 1 for a lame; a lame spans `value1` to
/// `value2` of the collar, and `value3` is its height along its extrusion.
const PLATES: &str = r#"
const FLUTED: u32 = 0u;
const COLLAR_START: u32 = 1u;
// The collar's lowest band is formed into the bib as one sheet.


fn shell_pass(index: u32, coord: vec4<f32>) -> u32 {
    return 0u;
}

fn shell_origin() -> vec3<f32> {
    let center = gorget_center();
    return vec3<f32>(center.x, 0.0, center.y);
}

fn shell_hinge() -> array<vec3<f32>, 2> {
    return array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let t = coord.x;
    let u = coord.y;
    let lame = params.value0 != 0.0;
    var mapped = u;
    var relief = 0.0;
    // The formed front bib owns its flute chart; the posterior shoulder
    // return stays plain.
    let front_u = 2.0 * (u - 0.25);
    if (!lame && design[FLUTED] != 0.0 && front_u >= 0.0 && front_u <= 1.0) {
        let pattern_v = 1.0 - max((t - COLLAR_ROWS_FRACTION) / (1.0 - COLLAR_ROWS_FRACTION), 0.0);
        mapped = 0.25 + 0.5 * flute_fan(front_u, pattern_v);
        relief = flute_relief(front_u, pattern_v);
    }
    let angle = (mapped - 0.5) * TAU;
    var point: vec3<f32>;
    if (lame) {
        point = gorget_collar(params.value1 + (params.value2 - params.value1) * t, angle);
    } else if (t <= COLLAR_ROWS_FRACTION) {
        let start = design[COLLAR_START];
        point = gorget_collar(start + (1.0 - start) * t / COLLAR_ROWS_FRACTION, angle);
    } else {
        point = gorget_bib((t - COLLAR_ROWS_FRACTION) / (1.0 - COLLAR_ROWS_FRACTION), angle);
    }
    return ShellVertex(point, params.value3 + relief);
}
"#;

/// A plate's columns: even, then the rear sweep's, then the flutes'.
fn columns(design: &GarmentArmorDesign, fluting: Option<&PlateFluting>) -> Vec<f32> {
    let mut columns = (0..AROUND)
        .map(|i| i as f32 / AROUND as f32)
        .collect::<Vec<_>>();
    if let GarmentPlateShape::Gorget { rear_sweep, .. } = design.plate_shape {
        columns.extend((0..AROUND).map(|i| {
            let angle = (i as f32 / AROUND as f32 - 0.5) * TAU;
            (gorget_control_angle(angle, rear_sweep) / TAU + 0.5).rem_euclid(1.0)
        }));
        columns.sort_by(f32::total_cmp);
        columns.dedup_by(|a, b| (*a - *b).abs() < MERGE_TOLERANCE);
    }
    if let Some(pattern) = fluting {
        columns.extend(pattern.columns(AROUND).into_iter().map(|u| 0.25 + 0.5 * u));
        columns.sort_by(f32::total_cmp);
        columns.dedup_by(|a, b| (*a - *b).abs() < MERGE_TOLERANCE);
    }
    columns
}

/// A cyclic plate of `rows` rows over `columns`, running from the neck.
fn plate(
    columns: &[f32],
    rows: usize,
    thickness: f32,
    extrusion: CoordExtrusion,
    values: [f32; 4],
) -> CoordShell {
    let stride = columns.len();
    let mut coords = Vec::with_capacity((rows + 1) * stride);
    let mut indices = Vec::with_capacity(rows * stride * 6);
    for row in 0..=rows {
        let t = row as f32 / rows as f32;
        coords.extend(columns.iter().map(|u| [t, *u, 0.0, 0.0]));
        if row == 0 {
            continue;
        }
        for col in 0..stride {
            let a = ((row - 1) * stride + col) as u32;
            let b = (row * stride + col) as u32;
            let c = (row * stride + (col + 1) % stride) as u32;
            let d = ((row - 1) * stride + (col + 1) % stride) as u32;
            indices.extend([a, b, c, a, c, d]);
        }
    }
    CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness,
        extrusion,
        values,
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    }
}

/// Record a shoulder bib and its separately thickened overlapping collar
/// lames from the surfaces `cage` defines over the fit in `fit`.
pub fn record_gorget_plates(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &GarmentArmorDesign,
    fit: &Buffer,
    cage: &str,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let count = usize::from(design.lame_count);
    let gauge = design.wall_thickness.metres();
    let flutes = FLUTING.replace("flute[", "design[FLUTE + ");
    let kernel = CoordKernel::new(
        gpu,
        &format!(
            "const COLLAR_ROWS_FRACTION: f32 = {};
const FLUTE: u32 = {FLUTE_WORDS_AT}u;
{cage}{flutes}{PLATES}",
            crate::GORGET_FORMED_COLLAR_FRACTION
        ),
    )?;
    let mut part = PartRecipe::new();
    let bib_columns = columns(design, design.fluting.as_ref());
    part.push_grid_coord(
        plate(
            &bib_columns,
            BIB_ROWS,
            gauge,
            CoordExtrusion::AngleWeightedNormal,
            [0.0; 4],
        ),
        kernel.clone(),
        super::part::GridShape {
            rows: (BIB_ROWS + 1) as u32,
            columns: bib_columns.len() as u32,
            cyclic: true,
        },
    )?;
    let lame_columns = columns(design, None);
    for lame in 0..count - 1 {
        let start = lame as f32 / count as f32;
        let end = (lame as f32 + 1.0 + LAP_FRACTION) / count as f32;
        part.push_grid_coord(
            plate(
                &lame_columns,
                LAME_ROWS,
                gauge,
                CoordExtrusion::Radial {
                    origin: [0.0; 3],
                    axis: [0.0, 1.0, 0.0],
                },
                [1.0, start, end, gauge * 1.5 * (count - lame - 1) as f32],
            ),
            kernel.clone(),
            super::part::GridShape {
                rows: (LAME_ROWS + 1) as u32,
                columns: lame_columns.len() as u32,
                cyclic: true,
            },
        )?;
    }
    let mut floats = vec![
        if design.fluting.is_some() { 1.0 } else { 0.0 },
        (count - 1) as f32 / count as f32,
    ];
    debug_assert_eq!(floats.len(), FLUTE_WORDS_AT);
    floats.extend(flute_words(design.fluting.as_ref()));
    part.record(gpu, batch, &floats, &[fit])
}
