//! Design-only strip topology; the skirt kernel seats every support on the GPU.

use super::coord::{CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use crate::{
    ArmorComponentRole, BoundaryNormals, GenerateError, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS,
    SkirtDesign,
};

pub(super) fn append(
    recipe: &mut PartRecipe,
    kernel: &CoordKernel,
    design: &SkirtDesign,
    first_row: usize,
) -> Result<(), GenerateError> {
    let Some(support) = &design.support else {
        return Ok(());
    };
    let first = first_row as f32;
    if let Some(stays) = &support.stays {
        for stay in 0..stays.count.0 {
            let column = SKIRT_RING_SEGMENTS as f32 * f32::from(stay) / f32::from(stays.count.0);
            let end = first + (SKIRT_AXIAL_SEGMENTS as f32 - first) * stays.length.unit();
            let mut coords = Vec::new();
            for row in 0..=SKIRT_AXIAL_SEGMENTS {
                let descent = first + (end - first) * row as f32 / SKIRT_AXIAL_SEGMENTS as f32;
                for side in [-1.0, 1.0] {
                    coords.push([column, descent, side, 3.0]);
                }
            }
            recipe.push_coord(
                strip(
                    coords,
                    2,
                    SKIRT_AXIAL_SEGMENTS,
                    false,
                    stays.width.metres(),
                    stays.thickness.metres(),
                    design.thickness.metres(),
                ),
                kernel.clone(),
            )?;
        }
        recipe.component(ArmorComponentRole::Boning, None);
    }
    if let Some(hoops) = &support.hoops {
        for hoop in 0..hoops.count.0 {
            let progress = if hoops.count.0 == 1 {
                0.5
            } else {
                f32::from(hoop) / f32::from(hoops.count.0 - 1)
            };
            let descent = first
                + (hoops.start.unit() + (hoops.end.unit() - hoops.start.unit()) * progress)
                    * (SKIRT_AXIAL_SEGMENTS as f32 - first);
            let mut coords = Vec::new();
            for side in [-1.0, 1.0] {
                for column in 0..SKIRT_RING_SEGMENTS {
                    coords.push([column as f32, descent, side, 4.0]);
                }
            }
            let mut shell = strip(
                coords,
                SKIRT_RING_SEGMENTS,
                1,
                true,
                hoops.width.metres(),
                hoops.thickness.metres(),
                design.thickness.metres(),
            );
            shell.values[2] = f32::from(hoops.count.0);
            recipe.push_coord(shell, kernel.clone())?;
        }
        recipe.component(ArmorComponentRole::Hoops, None);
    }
    Ok(())
}

/// Native strip lattice: columns run around the pelvis, rows toward the hem.
fn strip(
    coords: Vec<[f32; 4]>,
    columns: usize,
    rows: usize,
    cyclic: bool,
    width: f32,
    thickness: f32,
    cloth_gauge: f32,
) -> CoordShell {
    let mut indices = Vec::new();
    for row in 0..rows {
        for column in 0..columns - usize::from(!cyclic) {
            let a = (row * columns + column) as u32;
            let b = (row * columns + (column + 1) % columns) as u32;
            let c = b + columns as u32;
            let d = a + columns as u32;
            indices.extend([a, c, b, a, d, c]);
        }
    }
    CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness,
        extrusion: CoordExtrusion::Normal,
        values: [width, cloth_gauge, 0.0, 0.0],
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    }
}
