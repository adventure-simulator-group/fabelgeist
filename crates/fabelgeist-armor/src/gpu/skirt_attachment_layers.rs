//! Finishing sheets share the skirt attachment's fitted point table.
use super::SkirtAttachment;
use super::coord::{CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use crate::{BoundaryNormals, GenerateError};

pub(super) fn append(
    recipe: &mut PartRecipe,
    kernel: &CoordKernel,
    patch: &SkirtAttachment,
    mut first_carrier: u32,
) -> Result<(), GenerateError> {
    for attached in &patch.layers {
        let layer = &attached.layer;
        if layer.point_count == 0
            || layer.indices.is_empty()
            || !layer.indices.len().is_multiple_of(3)
            || layer.indices.iter().any(|&v| v >= layer.point_count)
            || !layer.offset_metres.is_finite()
            || layer.offset_metres < 0.
            || !layer.thickness_metres.is_finite()
            || layer.thickness_metres <= 0.
        {
            return Err(GenerateError::InvalidSurface);
        }
        if let Some(chart) = &layer.chart {
            if chart.len() != layer.point_count as usize
                || chart.iter().flatten().any(|v| !v.is_finite())
            {
                return Err(GenerateError::InvalidSurface);
            }
            recipe.textile_chart_points(first_carrier, chart);
        }
        // Native attachment point ordinal, face offset in metres, finishing tag.
        const ATTACHED_CLOTH_COORDINATE: f32 = 6.;
        let end = attached
            .first_point
            .checked_add(layer.point_count)
            .ok_or(GenerateError::InvalidSurface)?;
        recipe.push_coord(
            CoordShell {
                coords: (attached.first_point..end)
                    .map(|i| [i as f32, layer.offset_metres, 0., ATTACHED_CLOTH_COORDINATE])
                    .collect(),
                indices: layer.indices.clone(),
                boundary_normals: BoundaryNormals::Smooth,
                thickness: layer.thickness_metres,
                extrusion: CoordExtrusion::AngleWeightedNormal,
                values: [0.; 4],
                mirrored: false,
                frame: 0,
                passes: 1,
                hinge: None,
            },
            kernel.clone(),
        )?;
        recipe.component(layer.role, None);
        first_carrier += layer.point_count;
    }
    Ok(())
}
