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

/// The affine weights the quadratic bending constraint is built on.
///
/// Four points that lie in a plane always admit exactly one dependency
/// `sum(k_i) = 0` with `sum(k_i * x_i) = 0`. That combination is unchanged by
/// any affine map of the four points, so it measures departure from flatness
/// and nothing else -- which is precisely bending.
///
/// Found as the null vector of
///
/// ```text
/// [  1    1    1    1  ]
/// [ u0   u1   u2   u3  ]
/// [ v0   v1   v2   v3  ]
/// ```
///
/// where `u` and `v` are coordinates in the points' own plane, via the four
/// signed 3x3 minors.
///
/// The result is normalised and then scaled by the hinge's own length scale,
/// which is what puts the `Fabric` bend compliances into a sensible band --
/// see the presets, which were calibrated against this scaling.
///
/// It does **not** make bending fully resolution-independent. Sweeping the
/// exponent on that length scale (the `bend_scaling_grid` test in
/// `fabelgeist-cloth`, which is `#[ignore]`d and meant to be run by hand) shows
/// the compliance at which a flap starts to droop still shifts by something
/// like a decade when the target edge length is halved, and no exponent tried
/// removed that. Part of the drift is not stiffness at all: a flap only a few
/// hinges wide cannot represent much curvature whatever its compliance, so the
/// coarse end of the sweep is measuring the mesh rather than the material.
/// Expect to retune `bend_compliance` after a large change in resolution.
///
/// `points` is ordered hinge, hinge, wing, wing -- the order
/// [`BendQuad::particles`] produces.
///
/// Degenerate input -- three collinear points, a collapsed triangle -- has a
/// larger null space and no single answer, so it returns `None` and the caller
/// leaves that hinge unconstrained.
pub fn bending_weights(points: [fabelgeist_math::Vec3; 4]) -> Option<[f32; 4]> {
    use fabelgeist_math::Vec3;

    // A basis for the plane the four points lie in. The longest edge from
    // point 0 gives the most stable first axis.
    let spokes = [
        points[1] - points[0],
        points[2] - points[0],
        points[3] - points[0],
    ];
    let first = spokes
        .iter()
        .copied()
        .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))?;
    if first.length_squared() < 1e-24 {
        return None;
    }
    let u = first.normalize();

    // The second axis is whichever spoke is furthest off the first.
    let mut best = Vec3::default();
    let mut best_length = 0.0f32;
    for spoke in spokes {
        let perpendicular = spoke - u * spoke.dot(u);
        if perpendicular.length_squared() > best_length {
            best_length = perpendicular.length_squared();
            best = perpendicular;
        }
    }
    if best_length < 1e-24 {
        return None;
    }
    let v = best.normalize();

    let coordinates: [(f32, f32); 4] = points.map(|point| {
        let local = point - points[0];
        (local.dot(u), local.dot(v))
    });

    // Signed 3x3 minors of the 3x4 matrix, alternating sign: the null vector.
    let minor = |a: usize, b: usize, c: usize| {
        let (ua, va) = coordinates[a];
        let (ub, vb) = coordinates[b];
        let (uc, vc) = coordinates[c];
        ua * (vb - vc) - va * (ub - uc) + (ub * vc - vb * uc)
    };
    let weights = [
        -minor(1, 2, 3),
        minor(0, 2, 3),
        -minor(0, 1, 3),
        minor(0, 1, 2),
    ];

    let norm = weights.iter().map(|w| w * w).sum::<f32>().sqrt();
    if norm < 1e-12 {
        return None;
    }

    // The local length scale: the root of the two triangles' combined area.
    // More robust than any single edge when the triangles are not equilateral.
    let hinge = (points[0], points[1]);
    let area = |wing: Vec3| (hinge.1 - hinge.0).cross(wing - hinge.0).length() * 0.5;
    let scale = (area(points[2]) + area(points[3])).sqrt();
    if scale < 1e-9 {
        return None;
    }

    Some(weights.map(|w| w / norm * scale))
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

    /// The defining property: the weights annihilate any affine image of the
    /// rest points, and only that.
    #[test]
    fn bending_weights_annihilate_affine_maps() {
        let points = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.3, 0.8, 0.0),
            Vec3::new(0.7, -0.9, 0.0),
        ];
        let weights = bending_weights(points).expect("a proper planar quad has weights");

        // Sum to zero, so the measure is translation-invariant.
        assert!(weights.iter().sum::<f32>().abs() < 1e-5);

        let combine = |points: [Vec3; 4]| {
            points
                .iter()
                .zip(&weights)
                .fold(Vec3::default(), |acc, (p, &k)| acc + *p * k)
        };
        assert!(combine(points).length() < 1e-5, "rest is not annihilated");

        // Translation, rotation, uniform scale and shear are all affine, so
        // every one of them must still measure zero.
        let translated = points.map(|p| p + Vec3::new(3.0, -2.0, 7.0));
        assert!(combine(translated).length() < 1e-5, "translation");

        let rotated = points.map(|p| {
            let (s, c) = 0.7f32.sin_cos();
            Vec3::new(p.x * c - p.y * s, p.x * s + p.y * c, p.z)
        });
        assert!(combine(rotated).length() < 1e-5, "rotation");

        let scaled = points.map(|p| p * 2.5);
        assert!(combine(scaled).length() < 1e-5, "uniform scale");

        let sheared = points.map(|p| Vec3::new(p.x + 0.4 * p.y, p.y, p.z));
        assert!(combine(sheared).length() < 1e-4, "shear");

        // Folding one wing out of the plane is not affine, and must register.
        let mut folded = points;
        folded[3] = Vec3::new(0.7, -0.4, 0.5);
        assert!(
            combine(folded).length() > 0.05,
            "a fold produced no bending measure"
        );
    }

    /// The measure has to grow with the fold, or compliance means nothing.
    #[test]
    fn bending_weights_grow_with_the_fold() {
        let points = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 0.6, 0.0),
            Vec3::new(0.5, -0.6, 0.0),
        ];
        let weights = bending_weights(points).unwrap();
        let combine = |points: [Vec3; 4]| {
            points
                .iter()
                .zip(&weights)
                .fold(Vec3::default(), |acc, (p, &k)| acc + *p * k)
                .length()
        };

        let mut previous = 0.0;
        for lift in [0.05f32, 0.1, 0.2, 0.4] {
            let mut folded = points;
            folded[3] = Vec3::new(0.5, -0.6, lift);
            let measure = combine(folded);
            assert!(measure > previous, "lifting to {lift} did not increase it");
            previous = measure;
        }
    }

    #[test]
    fn bending_weights_reject_degenerate_quads() {
        // All four collinear: no unique dependency.
        let collinear = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
        ];
        assert!(bending_weights(collinear).is_none());

        // All coincident.
        assert!(bending_weights([Vec3::default(); 4]).is_none());
    }

    #[test]
    fn a_massless_vertex_is_pinned() {
        let inverse = inverse_masses(&[1.0.into(), 0.0.into(), 4.0.into()]);
        assert_eq!(inverse, vec![1.0.into(), 0.0.into(), 0.25.into()]);
    }
}
