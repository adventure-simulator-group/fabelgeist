//! What a triangle mesh implies about the constraints on it.
//!
//! Stretch lives on the edges, and bending lives on the pairs of triangles
//! that share one. Both fall straight out of the triangle list, so a caller
//! only ever supplies geometry.

use crate::{ParticleArealDensity, ParticleInverseMass, ParticleMass};
use std::collections::HashMap;

/// A pair of triangles sharing an edge, in the order the bending constraint
/// wants: `shared` is the hinge, `wings` are the two opposite vertices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BendQuad {
    /// The two ends of the shared edge.
    pub shared: [u32; 2],
    /// The vertex of each triangle that is not on the shared edge.
    pub wings: [u32; 2],
}

impl BendQuad {
    /// Flattened in the order the kernel reads: the hinge, then the wings.
    ///
    /// The bending weights are computed in this same order, so the two must
    /// stay in step -- a permutation here silently pairs each particle with
    /// another one's weight.
    pub fn particles(&self) -> [u32; 4] {
        [self.shared[0], self.shared[1], self.wings[0], self.wings[1]]
    }
}

#[derive(Clone, Debug, Default)]
pub struct Topology {
    /// Every edge, once, with the lower index first.
    pub edges: Vec<[u32; 2]>,
    /// One per interior edge -- an edge shared by exactly two triangles.
    pub bends: Vec<BendQuad>,
    /// Edges on the boundary: shared by one triangle only. A panel's outline.
    pub boundary_edges: Vec<[u32; 2]>,
}

fn key(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

/// Derive the edges and bend pairs of a triangle mesh.
///
/// An edge shared by more than two triangles -- a non-manifold seam left in by
/// mistake -- contributes its stretch constraint but no bending: there is no
/// single pair of triangles to hinge.
pub fn build(triangles: &[[u32; 3]]) -> Topology {
    let mut adjacency: HashMap<(u32, u32), Vec<u32>> = HashMap::new();

    for triangle in triangles {
        for pair in [
            (triangle[0], triangle[1], triangle[2]),
            (triangle[1], triangle[2], triangle[0]),
            (triangle[2], triangle[0], triangle[1]),
        ] {
            let (a, b, opposite) = pair;
            adjacency.entry(key(a, b)).or_default().push(opposite);
        }
    }

    let mut edges = Vec::with_capacity(adjacency.len());
    let mut bends = Vec::new();
    let mut boundary_edges = Vec::new();

    // Sorted, so that the same mesh always produces the same constraint order
    // -- which makes the graph colouring, and therefore the whole simulation,
    // reproducible.
    let mut keys: Vec<(u32, u32)> = adjacency.keys().copied().collect();
    keys.sort_unstable();

    for edge in keys {
        let opposites = &adjacency[&edge];
        edges.push([edge.0, edge.1]);
        match opposites.len() {
            1 => boundary_edges.push([edge.0, edge.1]),
            2 => bends.push(BendQuad {
                shared: [edge.0, edge.1],
                wings: [opposites[0], opposites[1]],
            }),
            _ => {}
        }
    }

    Topology {
        edges,
        bends,
        boundary_edges,
    }
}

/// Area-weighted vertex masses: each triangle gives a third of its mass to
/// each of its corners.
///
/// This is what makes a fine panel and a coarse one fall the same way. Giving
/// every particle the same mass instead would make a densely meshed region
/// heavier than a sparse one of the same size.
pub fn vertex_masses(
    positions: &[fabelgeist_math::Vec3],
    triangles: &[[u32; 3]],
    density: ParticleArealDensity,
) -> Vec<ParticleMass> {
    let mut masses = vec![ParticleMass::ZERO; positions.len()];
    for triangle in triangles {
        let a = positions[triangle[0] as usize];
        let b = positions[triangle[1] as usize];
        let c = positions[triangle[2] as usize];
        let share = density.vertex_share([a, b, c]);
        for &vertex in triangle {
            masses[vertex as usize] += share;
        }
    }
    masses
}

/// Inverse masses, with zero for anything that ended up massless -- a vertex
/// in no triangle. Zero pins it, which is the safe answer: it cannot be moved
/// by a constraint, rather than being moved infinitely far by one.
pub fn inverse_masses(masses: &[ParticleMass]) -> Vec<ParticleInverseMass> {
    masses
        .iter()
        .copied()
        .map(ParticleMass::inverse_mass)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_math::Vec3;

    /// Two triangles sharing an edge: one hinge, five edges, four boundary.
    #[test]
    fn finds_the_hinge_of_a_pair() {
        let triangles = [[0u32, 1, 2], [1, 3, 2]];
        let topology = build(&triangles);

        assert_eq!(topology.edges.len(), 5);
        assert_eq!(topology.bends.len(), 1);
        assert_eq!(topology.boundary_edges.len(), 4);

        let bend = topology.bends[0];
        assert_eq!(bend.shared, [1, 2], "the shared edge is 1-2");
        let mut wings = bend.wings;
        wings.sort_unstable();
        assert_eq!(wings, [0, 3], "the wings are the two opposite corners");
    }

    #[test]
    fn a_lone_triangle_has_no_hinge() {
        let topology = build(&[[0u32, 1, 2]]);
        assert_eq!(topology.edges.len(), 3);
        assert!(topology.bends.is_empty());
        assert_eq!(topology.boundary_edges.len(), 3);
    }

    /// A quad grid: interior edges hinge, the outline does not.
    #[test]
    fn counts_a_grid_correctly() {
        let (width, height) = (5u32, 4u32);
        let index = |x: u32, y: u32| y * width + x;
        let mut triangles = Vec::new();
        for y in 0..height - 1 {
            for x in 0..width - 1 {
                triangles.push([index(x, y), index(x + 1, y), index(x, y + 1)]);
                triangles.push([index(x + 1, y), index(x + 1, y + 1), index(x, y + 1)]);
            }
        }

        let topology = build(&triangles);
        // Euler: a triangulated grid of v vertices and t triangles has
        // (3t + boundary) / 2 edges.
        let boundary = 2 * (width - 1) + 2 * (height - 1);
        let expected_edges = (3 * triangles.len() as u32 + boundary) / 2;
        assert_eq!(topology.edges.len() as u32, expected_edges);
        assert_eq!(topology.boundary_edges.len() as u32, boundary);
        assert_eq!(
            topology.bends.len(),
            topology.edges.len() - topology.boundary_edges.len()
        );
    }

    /// A non-manifold edge still gets its stretch constraint, but hinging it
    /// would be meaningless.
    #[test]
    fn skips_non_manifold_edges() {
        let triangles = [[0u32, 1, 2], [1, 3, 2], [1, 4, 2]];
        let topology = build(&triangles);
        assert!(
            topology.bends.iter().all(|b| b.shared != [1, 2]),
            "an edge with three triangles must not hinge"
        );
        assert!(topology.edges.contains(&[1, 2]));
    }

    #[test]
    fn is_deterministic() {
        let triangles = [[0u32, 1, 2], [1, 3, 2], [2, 3, 4]];
        assert_eq!(build(&triangles).edges, build(&triangles).edges);
    }

    #[test]
    fn masses_follow_area_not_vertex_count() {
        // Two unit squares, one split into 2 triangles and one into 8.
        let coarse_positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let coarse = [[0u32, 1, 2], [0, 2, 3]];
        let masses = vertex_masses(&coarse_positions, &coarse, 2.0.into());
        let total: ParticleMass = masses.iter().sum();
        assert!(
            (total - ParticleMass::from(2.0)).absolute() < ParticleMass::from(1e-5),
            "a 1 m^2 panel at 2 kg/m^2 weighs {total}"
        );
    }

    #[test]
    fn a_massless_vertex_is_pinned() {
        let inverse = inverse_masses(&[1.0.into(), 0.0.into(), 4.0.into()]);
        assert_eq!(inverse, vec![1.0.into(), 0.0.into(), 0.25.into()]);
    }
}
