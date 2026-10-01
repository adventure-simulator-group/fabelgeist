//! Ring-and-panel textile topology. Only chart coordinates are built on the
//! host; the device evaluates every vertex against its fitted limb frame.
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::{AUTHORED, CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::{ArmorComponentRole, BoundaryNormals, GenerateError, PuffAndSlashDesign};

const RING_COLUMNS: usize = 64;
const LINING_ROWS: usize = 64;
const PANEL_COLUMNS: usize = 8;
const PANEL_ROWS: usize = 16;
const BAND_ROWS: usize = 4;
/// Separate cuff and panel boundaries by a construction-scale seam.
const PANEL_SEAM_GAP_M: f32 = 0.0001;

#[derive(Clone, Copy)]
enum Fabric {
    Lining,
    Puff,
    Band,
}

/// Record flexible puffed clothing, before anatomical fitting and thickening.
pub fn record_puff_and_slash(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &PuffAndSlashDesign,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let kernel = CoordKernel::new(
        gpu,
        &format!("{AUTHORED}{}", include_str!("puff_and_slash.wgsl")),
    )?;
    let mut recipe = PartRecipe::new();
    recipe.push_coord(
        patch(
            design,
            [0.0, 1.0],
            [0.0; 2],
            None,
            LINING_ROWS,
            Fabric::Lining,
        ),
        kernel.clone(),
    )?;
    recipe.component(ArmorComponentRole::Undercloth, None);
    let course = 1.0 / f32::from(design.puff_count);
    let half_band = course * design.constriction_width.unit() * 0.5;
    for band in 0..=design.puff_count {
        let center = f32::from(band) * course;
        recipe.push_coord(
            patch(
                design,
                [(center - half_band).max(0.0), (center + half_band).min(1.0)],
                [0.0; 2],
                None,
                BAND_ROWS,
                Fabric::Band,
            ),
            kernel.clone(),
        )?;
    }
    for puff in 0..design.puff_count {
        add_puff(&mut recipe, &kernel, design, puff)?;
    }
    recipe.component(ArmorComponentRole::OuterFabric, None);
    recipe.record(
        gpu,
        batch,
        &[
            f32::from(design.puff_count),
            design.puff_fullness.metres(),
            design.distal_fullness.unit(),
            design.puff_roundness.unit(),
            design.constriction_width.unit(),
            design.length.unit(),
            design.proximal_position.unit(),
            design.clearance.metres(),
            design.thickness.metres(),
        ],
        &[frame],
    )
}

fn patch(
    design: &PuffAndSlashDesign,
    span: [f32; 2],
    seams: [f32; 2],
    angles: Option<[f32; 2]>,
    rows: usize,
    fabric: Fabric,
) -> CoordShell {
    let periodic = angles.is_none();
    let columns = if periodic {
        RING_COLUMNS
    } else {
        PANEL_COLUMNS
    };
    let stride = columns + usize::from(!periodic);
    let angles = angles.unwrap_or([0.0, std::f32::consts::TAU]);
    let mut coords = Vec::with_capacity(stride * (rows + 1));
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..=rows {
        let t = row as f32 / rows as f32;
        for column in 0..stride {
            coords.push([
                angles[0] + (angles[1] - angles[0]) * column as f32 / columns as f32,
                span[0] + (span[1] - span[0]) * t,
                seams[0] + (seams[1] - seams[0]) * t,
                match fabric {
                    Fabric::Lining => 0.0,
                    Fabric::Puff => 1.0,
                    Fabric::Band => 2.0,
                },
            ]);
        }
    }
    for row in 0..rows {
        for column in 0..columns {
            let next = (column + 1) % stride;
            let a = (row * stride + column) as u32;
            let b = (row * stride + next) as u32;
            let c = ((row + 1) * stride + column) as u32;
            let d = ((row + 1) * stride + next) as u32;
            indices.extend([a, b, d, a, d, c]);
        }
    }
    CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness: design.thickness.metres(),
        extrusion: CoordExtrusion::Normal,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    }
}

fn add_puff(
    recipe: &mut PartRecipe,
    kernel: &CoordKernel,
    design: &PuffAndSlashDesign,
    puff: u8,
) -> Result<(), GenerateError> {
    let course = 1.0 / f32::from(design.puff_count);
    let half_band = course * design.constriction_width.unit() * 0.5;
    let low = f32::from(puff) * course + half_band;
    let high = f32::from(puff + 1) * course - half_band;
    if design.slash_count == 0 {
        recipe.push_coord(
            patch(
                design,
                [low, high],
                [0.0; 2],
                None,
                PANEL_ROWS,
                Fabric::Puff,
            ),
            kernel.clone(),
        )?;
        return Ok(());
    }
    let margin = (high - low) * (1.0 - design.slash_length.unit()) * 0.5;
    let slash = [low + margin, high - margin];
    for (span, seams) in [
        ([low, slash[0]], [0.0, -PANEL_SEAM_GAP_M]),
        ([slash[1], high], [PANEL_SEAM_GAP_M, 0.0]),
    ] {
        recipe.push_coord(
            patch(design, span, seams, None, BAND_ROWS, Fabric::Puff),
            kernel.clone(),
        )?;
    }
    let repeat = std::f32::consts::TAU / f32::from(design.slash_count);
    let half_width = repeat * (1.0 - design.slash_width.unit()) * 0.5;
    for panel in 0..design.slash_count {
        let center = repeat * (f32::from(panel) + design.rotation.unit());
        recipe.push_coord(
            patch(
                design,
                slash,
                [PANEL_SEAM_GAP_M, -PANEL_SEAM_GAP_M],
                Some([center - half_width, center + half_width]),
                PANEL_ROWS,
                Fabric::Puff,
            ),
            kernel.clone(),
        )?;
    }

    Ok(())
}
