//! The outer face of a plate as rows along it and columns across it.

/// The outer face of one plate, as the grid its carrier was laid out on:
/// `rows` rows along the plate, each `columns` vertices across it, closed
/// around when `cyclic`. A construction laid over the plate, such as scales,
/// is placed in the grid's own coordinates, so it follows the plate through
/// every body morph.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceGrid {
    pub rows: u32,
    pub columns: u32,
    pub cyclic: bool,
    /// The piece's vertex at each row and column, row by row.
    pub vertices: Vec<u32>,
    /// Every exported outer vertex, including cuts and shading aliases.
    pub samples: Vec<SurfaceSample>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSample {
    pub vertex: u32,
    pub column: SurfaceColumn,
    /// Unclamped correspondence in the original carrier's columns. Physical
    /// cut endpoints can extend beyond the retained construction rails.
    pub carrier_column: SurfaceColumn,
    /// Signed inward distance from the first and last construction edges.
    pub edge_distances: [SurfaceEdgeDistance; 2],
}

/// A regular rail or an authored physical course cut defines the seam.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceEdgeDistance {
    Rail,
    /// Perpendicular distance from the inner carrier's cut plane, in metres.
    /// Both physical walls retain the same material coordinate.
    Cut {
        metres: f32,
    },
}

impl SurfaceSample {
    pub fn edge_distance(&self, end: crate::PlateGridEnd) -> SurfaceEdgeDistance {
        self.edge_distances[match end {
            crate::PlateGridEnd::First => 0,
            crate::PlateGridEnd::Last => 1,
        }]
    }
}

/// A continuous column coordinate in the plate's construction grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceColumn(f32);

impl SurfaceColumn {
    pub fn at(column: u32) -> Self {
        Self(column as f32)
    }
    pub fn between(a: Self, b: Self, blend: f32) -> Self {
        Self(a.0 + blend * (b.0 - a.0))
    }
    pub fn distance(self, other: Self) -> f32 {
        (self.0 - other.0).abs()
    }
    pub fn total_cmp(self, other: Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
    pub fn bracket(self, columns: u32) -> Option<(u32, u32, f32)> {
        if columns == 0 || !self.0.is_finite() || self.0 < 0.0 || self.0 > (columns - 1) as f32 {
            return None;
        }
        let first = self.0.floor() as u32;
        Some((first, (first + 1).min(columns - 1), self.0 - first as f32))
    }
}

impl SurfaceGrid {
    pub fn regular(rows: u32, columns: u32, cyclic: bool, vertices: Vec<u32>) -> Self {
        let samples = vertices
            .iter()
            .enumerate()
            .map(|(i, &vertex)| SurfaceSample {
                vertex,
                column: SurfaceColumn::at(i as u32 % columns),
                carrier_column: SurfaceColumn::at(i as u32 % columns),
                edge_distances: [SurfaceEdgeDistance::Rail; 2],
            })
            .collect();
        Self {
            rows,
            columns,
            cyclic,
            vertices,
            samples,
        }
    }
    pub fn remap_vertices(&mut self, mut remap: impl FnMut(u32) -> u32) {
        for vertex in &mut self.vertices {
            *vertex = remap(*vertex);
        }
        for sample in &mut self.samples {
            sample.vertex = remap(sample.vertex);
        }
    }
    /// The piece's vertex at `row` and `column`.
    pub fn vertex(&self, row: u32, column: u32) -> u32 {
        self.vertices[(row * self.columns + column) as usize]
    }
}
