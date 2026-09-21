//! The topology of a thickened plate, decided on the host from its carrier's
//! triangles alone.
//!
//! A carrier becomes a closed shell of three parts: the carrier itself as the
//! outer wall, a copy offset inward as the inner wall, and a return wall
//! closing every boundary edge between them. Which vertices exist, and which
//! triangles join them, depends only on the carrier's connectivity; where
//! they end up is the device's business. Each final vertex is therefore
//! described by its [`VertexSource`]: the carrier vertex it derives from and
//! the wall it lies on.

use std::collections::BTreeMap;

use crate::{BoundaryNormals, GenerateError};

/// The wall a final vertex lies on, and the carrier vertex it derives from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VertexSource {
    Outer(u32),
    Inner(u32),
}

impl VertexSource {
    /// Packed for the device: the carrier index, with the top bit set for the
    /// inner wall.
    pub(crate) fn packed(self) -> u32 {
        match self {
            Self::Outer(i) => i,
            Self::Inner(i) => i | INNER_BIT,
        }
    }
}

pub(crate) const INNER_BIT: u32 = 1 << 31;

/// The final vertices and triangles of one thickened carrier, indexed from
/// zero within the shell.
#[derive(Clone, Debug)]
pub(crate) struct ShellPlan {
    pub sources: Vec<VertexSource>,
    /// The carrier's own triangles first, as the outer wall, then the inner
    /// and return walls.
    pub indices: Vec<u32>,
    pub carrier_triangles: usize,
}

impl ShellPlan {
    /// Thicken a carrier of `count` vertices wound outward, closing every
    /// boundary edge. Seams must share indices; an edge used more than twice,
    /// or twice in the same direction, is not a surface.
    pub(crate) fn new(
        count: u32,
        carrier: &[u32],
        boundary_normals: BoundaryNormals,
    ) -> Result<Self, GenerateError> {
        if count == 0
            || carrier.is_empty()
            || !carrier.len().is_multiple_of(3)
            || carrier.iter().any(|i| *i >= count)
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        for triangle in carrier.as_chunks::<3>().0 {
            for (a, b) in [
                (triangle[0], triangle[1]),
                (triangle[1], triangle[2]),
                (triangle[2], triangle[0]),
            ] {
                edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
            }
        }
        if edges
            .values()
            .any(|e| e.len() > 2 || (e.len() == 2 && e[0] != (e[1].1, e[1].0)))
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut sources = (0..count)
            .map(VertexSource::Outer)
            .chain((0..count).map(VertexSource::Inner))
            .collect::<Vec<_>>();
        let mut indices = carrier.to_vec();
        indices.extend(
            carrier
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|t| [t[0] + count, t[2] + count, t[1] + count]),
        );
        for edge in edges.values().filter(|e| e.len() == 1) {
            let (a, b) = edge[0];
            // Keep alias ordering independent of winding, including reflected fits.
            let reversed = a > b;
            let (a, b) = if reversed { (b, a) } else { (a, b) };
            let [a, b, c, d] = match boundary_normals {
                BoundaryNormals::Smooth => [a, b, a + count, b + count],
                BoundaryNormals::Separate => {
                    let first = sources.len() as u32;
                    sources.extend([
                        VertexSource::Outer(a),
                        VertexSource::Outer(b),
                        VertexSource::Inner(a),
                        VertexSource::Inner(b),
                    ]);
                    [first, first + 1, first + 2, first + 3]
                }
            };
            indices.extend(if reversed {
                [b, c, a, b, d, c]
            } else {
                [b, a, c, b, c, d]
            });
        }
        Ok(Self {
            sources,
            indices,
            carrier_triangles: carrier.len() / 3,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_triangle_closes_into_a_prism() {
        let plan = ShellPlan::new(3, &[0, 1, 2], BoundaryNormals::Smooth).unwrap();
        assert_eq!(plan.sources.len(), 6);
        // Outer, inner and one quad per boundary edge.
        assert_eq!(plan.indices.len(), 3 + 3 + 3 * 6);
        let mut uses = BTreeMap::<(u32, u32), i32>::new();
        for t in plan.indices.as_chunks::<3>().0 {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                *uses.entry((a, b)).or_default() += 1;
                *uses.entry((b, a)).or_default() -= 1;
            }
        }
        assert!(
            uses.values().all(|v| *v == 0),
            "every edge is matched by its reverse"
        );
    }

    #[test]
    fn separate_walls_own_their_vertices() {
        let plan = ShellPlan::new(3, &[0, 1, 2], BoundaryNormals::Separate).unwrap();
        assert_eq!(plan.sources.len(), 6 + 3 * 4);
        assert_eq!(plan.sources[6], VertexSource::Outer(0));
        assert_eq!(plan.sources[8], VertexSource::Inner(0));
    }

    #[test]
    fn a_folded_edge_is_not_a_surface() {
        assert!(ShellPlan::new(4, &[0, 1, 2, 0, 1, 3], BoundaryNormals::Smooth).is_err());
    }
}
