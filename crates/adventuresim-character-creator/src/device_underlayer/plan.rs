//! The frozen cut plan of an underlayer and the shell connectivity it implies.
//!
//! The Boolean cut is made once, on the wearer's own body, and every body
//! realization afterwards only moves its barycentric source points. The cut
//! is therefore the garment's topology: it decides the vertex and triangle
//! counts that size every device buffer, and its welding of cut points
//! shared by neighbouring body triangles relies on the host's `f64`
//! clipping. It stays on the host; everything it feeds -- offsets, layers,
//! normals and attributes -- is evaluated on the device.

use anyhow::{Result, ensure};

use crate::armor_frames::Wearer;
use crate::surface_cut::{SurfaceCut, interpolate};
use crate::underlayer::{RegionFrame, UnderlayerDesign, regions};

/// Words per cut point in [`CutPlan::table`]: the source triangle's body
/// vertices, its surface-coordinate vertices, then the barycentric weights
/// as `f32` bits.
pub const POINT_WORDS: usize = 9;

/// The cut of one underlayer, and its two-layer shell with separate cut-edge
/// vertices, as index buffers.
#[derive(Clone, Debug, PartialEq)]
pub struct CutPlan {
    /// Cut points, each a body triangle and barycentric weights.
    pub table: Vec<u32>,
    /// Shell triangles: outer layer, reversed inner layer, then cut edges.
    pub indices: Vec<u32>,
    /// For each shell vertex, which layer point it copies: the outer layer's
    /// points first, then the inner layer's, each `point_count` long.
    pub sources: Vec<u32>,
    pub point_count: u32,
}

impl CutPlan {
    /// Cut the design's regions from `body`, whose triangles carry the
    /// surface-coordinate triangles `uv_faces`; `frame` fits the part frames
    /// that size the regions.
    pub fn new(
        design: &UnderlayerDesign,
        placement: &str,
        body: &Wearer<'_>,
        uv_faces: &[[u32; 3]],
        frame: RegionFrame<'_>,
    ) -> Result<Self> {
        design.validate()?;
        let (include, subtract) = regions(design, placement, body, frame)?;
        let cut = SurfaceCut::new(body.positions, body.faces, uv_faces, &include, &subtract);
        ensure!(
            !cut.faces.is_empty(),
            "underlayer cuts removed the whole garment"
        );
        Ok(Self::from_cut(&cut, body, uv_faces))
    }

    /// The shell of an existing cut of `body`.
    pub fn from_cut(cut: &SurfaceCut, body: &Wearer<'_>, uv_faces: &[[u32; 3]]) -> Self {
        let positions = cut
            .points
            .iter()
            .map(|s| {
                interpolate(
                    body.faces[s.triangle].map(|v| body.positions[v as usize]),
                    s.weights,
                )
            })
            .collect::<Vec<_>>();
        let borders = cut.borders(&positions);
        let count = cut.points.len() as u32;
        let mut sources = (0..count * 2).collect::<Vec<_>>();
        let mut indices = Vec::with_capacity(cut.faces.len() * 6 + borders.len() * 6);
        for &[a, b, c] in &cut.faces {
            indices.extend([a, b, c, c + count, b + count, a + count]);
        }
        for &[a, b] in &borders {
            // A textile cut edge has its own shading normal. Sharing these
            // vertices with the inner face would darken the hem and collar.
            let first = sources.len() as u32;
            sources.extend([a, b, a + count, b + count]);
            indices.extend([first + 1, first, first + 2, first + 1, first + 2, first + 3]);
        }
        let mut table = Vec::with_capacity(cut.points.len() * POINT_WORDS);
        for point in &cut.points {
            table.extend(body.faces[point.triangle]);
            table.extend(uv_faces[point.triangle]);
            table.extend(point.weights.map(f32::to_bits));
        }
        Self {
            table,
            indices,
            sources,
            point_count: count,
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.sources.len() as u32
    }

    pub fn triangle_count(&self) -> u32 {
        (self.indices.len() / 3) as u32
    }
}

/// Each body vertex's triangles, ascending, once per corner it occupies:
/// `vertex_count + 1` offsets, then the triangle lists they delimit.
///
/// The body's connectivity never changes between its realizations, so this
/// is built once from the faces alone.
pub fn incidence(vertex_count: usize, faces: &[[u32; 3]]) -> Vec<u32> {
    let mut counts = vec![0u32; vertex_count + 1];
    for face in faces {
        for &v in face {
            counts[v as usize + 1] += 1;
        }
    }
    for v in 0..vertex_count {
        counts[v + 1] += counts[v];
    }
    let offset = counts.len() as u32;
    let mut cursor = counts.clone();
    let mut result = counts;
    for entry in &mut result {
        *entry += offset;
    }
    result.resize(offset as usize + faces.len() * 3, 0);
    for (triangle, face) in faces.iter().enumerate() {
        for &v in face {
            let slot = &mut cursor[v as usize];
            result[(offset + *slot) as usize] = triangle as u32;
            *slot += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incidence_lists_each_corner_in_triangle_order() {
        let faces = [[0, 1, 2], [2, 1, 3], [3, 1, 0]];
        let table = incidence(4, &faces);
        let list = |v: usize| {
            let (start, end) = (table[v] as usize, table[v + 1] as usize);
            table[start..end].to_vec()
        };
        assert_eq!(list(0), [0, 2]);
        assert_eq!(list(1), [0, 1, 2]);
        assert_eq!(list(2), [0, 1]);
        assert_eq!(list(3), [1, 2]);
    }
}
