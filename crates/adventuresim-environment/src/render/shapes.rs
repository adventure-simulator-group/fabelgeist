//! The base meshes instances are built from. Their positions are not places
//! but addresses the vertex shaders read: a tube's column and ring, a blade's
//! corner and number.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// The two triangles of a quad, by corner (across, along), counter-clockwise
/// from outside.
const QUAD: [[u32; 2]; 6] = [[0, 0], [1, 0], [0, 1], [1, 0], [1, 1], [0, 1]];

/// A tube's grid: `sides + 1` columns (the last closing the seam) by
/// `spans + 1` rings, each vertex at `(column, ring, 0)`.
pub(super) fn branch_grid(sides: u32, spans: u32) -> Mesh {
    let sides = sides.max(3);
    let spans = spans.max(1);
    let columns = sides + 1;
    let positions: Vec<[f32; 3]> = (0..=spans)
        .flat_map(|ring| (0..columns).map(move |column| [column as f32, ring as f32, 0.0]))
        .collect();
    let vertex = |column: u32, ring: u32| ring * columns + column;
    let indices: Vec<u32> = (0..spans)
        .flat_map(|ring| {
            (0..sides).flat_map(move |column| {
                QUAD.map(|[across, along]| vertex(column + across, ring + along))
            })
        })
        .collect();
    mesh(positions, indices)
}

/// `blades` quads, each vertex at `(across, along, blade)`.
pub(super) fn leaf_blades(blades: u32) -> Mesh {
    let blades = blades.max(1);
    let positions: Vec<[f32; 3]> = (0..blades)
        .flat_map(|blade| {
            [[0, 0], [1, 0], [0, 1], [1, 1]]
                .map(|[across, along]: [u32; 2]| [across as f32, along as f32, blade as f32])
        })
        .collect();
    let indices: Vec<u32> = (0..blades)
        .flat_map(|blade| QUAD.map(|[across, along]| blade * 4 + across + 2 * along))
        .collect();
    mesh(positions, indices)
}

fn mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices))
}
