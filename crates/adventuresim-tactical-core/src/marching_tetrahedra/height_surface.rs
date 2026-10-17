//! Trim monotone vertical height columns before tetrahedron extraction.
use super::*;
use crate::volumetric_terrain::{TerrainRecipeError, TerrainRecipeResult};

/// Extract `position.y - height[x,z]` on the producer's native metre lattice.
/// Positions use identical horizontal coordinates at every elevation, with
/// increasing Y shared by all columns. Other erosion fields use the general
/// three-dimensional sampler because they can contain caves and overhangs.
pub(crate) fn marching_height_surface(
    dimensions: [usize; 3],
    position_at: impl Fn([usize; 3]) -> Vec3,
    heights: &[f32],
    transition_collar: TerrainTransitionCollar,
) -> TerrainRecipeResult<SceneTerrainPatch> {
    let [width, vertical, depth] = dimensions;
    let columns = width
        .checked_mul(depth)
        .ok_or(TerrainRecipeError::SampleCountOverflow)?;
    columns
        .checked_mul(vertical)
        .ok_or(TerrainRecipeError::SampleCountOverflow)?;
    if dimensions.iter().any(|&side| side < 2) || heights.len() != columns {
        return Err(TerrainRecipeError::VoxelGridBound);
    }
    let elevations = (0..vertical)
        .map(|y| position_at([0, y, 0]).y)
        .collect::<Vec<_>>();
    if heights
        .iter()
        .chain(&elevations)
        .any(|value| !value.is_finite())
        || heights.iter().any(|&height| {
            !(elevations[0] - height).is_finite()
                || !(elevations[vertical - 1] - height).is_finite()
        })
        || elevations.windows(2).any(|pair| pair[0] >= pair[1])
        || (0..columns).any(|column| {
            let x = column % width;
            let z = column / width;
            !position_at([x, 0, z]).is_finite() || !position_at([x, vertical - 1, z]).is_finite()
        })
    {
        return Err(TerrainRecipeError::NonFiniteField);
    }
    // First positive endpoint, using the original subtraction and zero snap.
    // No float-to-grid inversion changes classification at lattice corners.
    let crossings = heights
        .iter()
        .map(|&height| elevations.partition_point(|&y| snap_field_zero(y - height) <= 0.0))
        .collect::<Vec<_>>();
    let sample_at = |index| {
        let yz = index / width;
        let coordinate = [index % width, yz % vertical, yz / vertical];
        let position = position_at(coordinate);
        LatticeSample {
            position,
            value: snap_field_zero(position.y - heights[coordinate[2] * width + coordinate[0]]),
        }
    };
    let (positions, indices) = extract_height_surface(dimensions, &crossings, &sample_at)?;
    build_surface(positions, indices, transition_collar)
}

fn extract_height_surface(
    dimensions: [usize; 3],
    crossings: &[usize],
    sample_at: &impl Fn(usize) -> LatticeSample,
) -> TerrainRecipeResult<(Vec<Vec3>, Vec<u32>)> {
    let [width, vertical, depth] = dimensions;
    let sample_id = |x: usize, y: usize, z: usize| (z * vertical + y) * width + x;
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut edge_vertices = HashMap::new();
    for z in 0..depth - 1 {
        let rows = &crossings[z * width..(z + 2) * width];
        let first = rows.iter().copied().min().unwrap_or(0).saturating_sub(1);
        let end = rows.iter().copied().max().unwrap_or(0).min(vertical - 1);
        // Keep z/y/x/tetrahedron visitation and global edge identities exactly
        // as in the full lattice, skipping only cells with one classification.
        // Edge trimming: https://vtk.org/doc/nightly/html/classvtkFlyingEdges3D.html
        for y in first..end {
            for x in 0..width - 1 {
                let corners = [rows[x], rows[x + 1], rows[width + x], rows[width + x + 1]];
                if corners.iter().all(|&crossing| y + 1 < crossing)
                    || corners.iter().all(|&crossing| y >= crossing)
                {
                    continue;
                }
                let cube =
                    CUBE.map(|offset| sample_id(x + offset[0], y + offset[1], z + offset[2]));
                for tet in TETS {
                    emit_tetrahedron(
                        tet.map(|corner| cube[corner]),
                        sample_at,
                        &mut edge_vertices,
                        &mut positions,
                        &mut indices,
                    )?;
                }
            }
        }
    }
    Ok((positions, indices))
}

#[cfg(test)]
mod tests;
