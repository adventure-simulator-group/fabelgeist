//! Export construction correspondence on the actual rendered outer wall.
use super::topology::{MidTopology, SolidTopology};
use crate::{PlateFace, SurfaceEdgeDistance, SurfaceGrid, SurfaceSample};

impl SolidTopology {
    pub(super) fn install_grid(
        &mut self,
        mid: &MidTopology,
        outer: &MidTopology,
        boundaries: &[[u32; 2]],
    ) {
        let count = mid.vertex_count() as u32;
        let course = mid
            .course_coordinates
            .as_ref()
            .map(|course| course.on_boundaries(boundaries));
        let active = self
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .zip(&self.faces)
            .filter(|(_, face)| **face == PlateFace::Outer)
            .flat_map(|(face, _)| face.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        self.grids.push(SurfaceGrid {
            rows: mid.rows as u32,
            columns: outer.construction_columns.width() as u32,
            cyclic: false,
            vertices: outer
                .grid_vertices
                .clone()
                .unwrap_or_else(|| mid.rows_downward().collect())
                .into_iter()
                .map(|vertex| vertex + count)
                .collect(),
            samples: active
                .into_iter()
                .map(|vertex| SurfaceSample {
                    vertex,
                    carrier_column: mid.surface_column(self.mid_of(vertex)),
                    column: outer
                        .construction_columns
                        .column(mid.surface_column(self.mid_of(vertex)), mid.width()),
                    edge_distances: course
                        .as_ref()
                        .map_or([SurfaceEdgeDistance::Rail; 2], |course| {
                            course.edge_distances(self.mid_of(vertex))
                        }),
                })
                .collect(),
        });
    }
}
