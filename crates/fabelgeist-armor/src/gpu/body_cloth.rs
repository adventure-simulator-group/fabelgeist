//! Native body-patch topology; positions are evaluated by the fitting owner.
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::{AUTHORED, CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::{ArmorComponentRole, BoundaryNormals, GenerateError};

/// Each fitted point is six floats: canonical metre position and unit normal.
pub const BODY_CLOTH_POINT_WORDS: u32 = 6;
pub const BODY_CLOTH_FIRST_POINT: u32 = 16;

/// One independently thickened layer over a shared fitted point table.
pub struct BodyClothLayer {
    pub point_count: u32,
    pub indices: Vec<u32>,
    pub role: ArmorComponentRole,
    /// Optional logical fabric coordinates per carrier; independent of refit.
    pub chart: Option<Vec<[f32; 2]>>,
    /// Outward offset to the face; the wall extends inward by `thickness`.
    pub offset_metres: f32,
    pub thickness_metres: f32,
}

/// `fit` begins with the identity frame and contains body points in canonical
/// unposed metres. The host supplies connectivity, never evaluated positions.
pub fn record_body_cloth(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    layers: &[BodyClothLayer],
    fit: &Buffer,
) -> Result<DevicePart, GenerateError> {
    let kernel = CoordKernel::with_extrusion(
        gpu,
        &format!(
            "const FIRST_POINT: u32 = {BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS: u32 = {BODY_CLOTH_POINT_WORDS}u;\n{AUTHORED}{}",
            include_str!("body_cloth.wgsl")
        ),
        include_str!("body_cloth_extrusion.wgsl"),
    )?;
    let mut recipe = PartRecipe::new();
    recipe.textile_chart();
    let mut first = 0;
    for layer in layers {
        if layer.point_count == 0
            || layer.indices.is_empty()
            || !layer.indices.len().is_multiple_of(3)
            || layer.indices.iter().any(|&v| v >= layer.point_count)
            || !layer.offset_metres.is_finite()
            || !layer.thickness_metres.is_finite()
            || layer.thickness_metres <= 0.
            || layer.offset_metres < layer.thickness_metres
        {
            return Err(GenerateError::InvalidSurface);
        }
        if let Some(chart) = &layer.chart {
            if chart.len() != layer.point_count as usize
                || chart.iter().flatten().any(|v| !v.is_finite())
            {
                return Err(GenerateError::InvalidSurface);
            }
            recipe.textile_chart_points(first, chart);
        }
        recipe.push_coord(
            CoordShell {
                coords: (first..first + layer.point_count)
                    .map(|point| [point as f32, layer.offset_metres, 0.0, 0.0])
                    .collect(),
                indices: layer.indices.clone(),
                boundary_normals: BoundaryNormals::Smooth,
                thickness: layer.thickness_metres,
                extrusion: CoordExtrusion::FieldNormal,
                values: [0.0; 4],
                mirrored: false,
                frame: 0,
                passes: 1,
                hinge: None,
            },
            kernel.clone(),
        )?;
        recipe.component(layer.role, None);
        first += layer.point_count;
    }
    recipe.record(gpu, batch, &[0.0], &[fit])
}
