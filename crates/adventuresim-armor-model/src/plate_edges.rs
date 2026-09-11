//! Physical trim boundaries independent of material-atlas seams.
use super::PartMesh;
use std::collections::BTreeMap;

impl PartMesh {
    /// Authored outer-sheet boundaries, before returns close each plate.
    pub fn plate_edges(&self) -> Vec<[u32; 2]> {
        self.shells
            .iter()
            .flat_map(|shell| {
                boundary_edges(
                    &self.indices[shell.first_index..shell.first_index + shell.index_count],
                )
            })
            .collect()
    }
}

pub(crate) fn boundary_edges(indices: &[u32]) -> Vec<[u32; 2]> {
    let mut counts = BTreeMap::new();
    for triangle in indices.as_chunks::<3>().0 {
        for [a, b] in [
            [triangle[0], triangle[1]],
            [triangle[1], triangle[2]],
            [triangle[2], triangle[0]],
        ] {
            *counts.entry([a.min(b), a.max(b)]).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(edge, count)| (count == 1).then_some(edge))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{BoundaryNormals, PartMesh, ShellExtrusion};

    #[test]
    fn trim_follows_sheet_boundary_without_diagonal_or_inner_aliases() {
        let plate = PartMesh::from_surface(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 0, 2, 3],
            0.002,
            BoundaryNormals::Separate,
            ShellExtrusion::Normal,
        )
        .unwrap();
        assert_eq!(plate.plate_edges(), vec![[0, 1], [0, 3], [1, 2], [2, 3]]);
        let mut pair = plate.clone();
        let offset = pair.positions.len() as u32;
        pair.append(plate);
        assert_eq!(
            &pair.plate_edges()[4..],
            &[
                [offset, offset + 1],
                [offset, offset + 3],
                [offset + 1, offset + 2],
                [offset + 2, offset + 3]
            ]
        );
    }
}
