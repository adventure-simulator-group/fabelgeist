//! Cloth surface contacts with swept vertex/triangle and edge/edge detection.
//! All garments share one particle array. Mesh neighbours are excluded.
//! Zero inverse mass means prescribed motion, not a stationary collider:
//! previous and current positions must both be supplied for animated bodies.
use crate::ccd::{self, Pair};
use fabelgeist_bvh::{Aabb, Bvh};
use fabelgeist_math::Vec3;
use std::collections::BTreeSet;

pub struct SurfaceContacts {
    faces: Vec<[u32; 3]>,
    edges: Vec<[u32; 2]>,
    neighbours: Vec<BTreeSet<u32>>,
}

impl SurfaceContacts {
    pub fn new(count: usize, faces: Vec<[u32; 3]>) -> Self {
        let mut neighbours = vec![BTreeSet::new(); count];
        let mut edges = BTreeSet::new();
        for face in &faces {
            for k in 0..3 {
                let (a, b) = (face[k], face[(k + 1) % 3]);
                assert!((a as usize) < count && (b as usize) < count);
                neighbours[a as usize].insert(b);
                neighbours[b as usize].insert(a);
                edges.insert([a.min(b), a.max(b)]);
            }
        }
        Self {
            faces,
            edges: edges.into_iter().collect(),
            neighbours,
        }
    }

    /// Resolve swept surface contacts. Paths are linear between the supplied
    /// positions, so callers must invoke this for every simulation substep.
    /// Contact sweeps rebuild candidate bounds from the corrected endpoints.
    /// Initially intersecting meshes need an untangled starting state; CCD
    /// prevents new crossings rather than inferring the intended layer order.
    pub fn solve(
        &self,
        positions: &mut [Vec3],
        previous: &[Vec3],
        inverse_masses: &[f32],
        thickness: f32,
        iterations: u32,
    ) -> usize {
        assert_eq!(positions.len(), self.neighbours.len());
        assert_eq!(positions.len(), previous.len());
        assert_eq!(positions.len(), inverse_masses.len());
        assert!(inverse_masses.iter().all(|m| m.is_finite() && *m >= 0.0));
        assert!(positions.iter().chain(previous).all(|p| p.is_finite()));
        if !thickness.is_finite() || thickness <= 0.0 {
            return 0;
        }
        let mut contacts = 0;
        for _ in 0..iterations {
            // Build only swept boxes. Testing endpoints alone misses an edge
            // that crosses another edge and exits before the substep ends.
            let bounds: Vec<_> = self
                .faces
                .iter()
                .map(|face| {
                    Aabb::from_points(
                        face.iter()
                            .flat_map(|&i| [positions[i as usize], previous[i as usize]]),
                    )
                    .expand(thickness)
                })
                .collect();
            let tree = Bvh::build(&bounds);
            // Fixed vertices only need to query dynamic faces, avoiding
            // expensive body-against-body searches on a dense animated mesh.
            let dynamic_faces: Vec<_> = self
                .faces
                .iter()
                .enumerate()
                .filter_map(|(i, f)| {
                    f.iter()
                        .any(|&v| inverse_masses[v as usize] > 0.0)
                        .then_some(i)
                })
                .collect();
            let dynamic_bounds: Vec<_> = dynamic_faces.iter().map(|&f| bounds[f]).collect();
            let dynamic_tree = Bvh::build(&dynamic_bounds);
            let mut candidates = Vec::new();
            for v in 0..positions.len() {
                candidates.clear();
                let query = Aabb::from_points([positions[v], previous[v]]).expand(thickness);
                if inverse_masses[v] > 0.0 {
                    tree.query_aabb(&bounds, &query, |f| candidates.push(f as usize));
                } else {
                    dynamic_tree.query_aabb(&dynamic_bounds, &query, |f| {
                        candidates.push(dynamic_faces[f as usize])
                    });
                }
                for &f in &candidates {
                    let face = self.faces[f];
                    if face
                        .iter()
                        .any(|&i| i as usize == v || self.neighbours[v].contains(&i))
                    {
                        continue;
                    }
                    contacts += resolve(
                        Pair::VertexTriangle,
                        positions,
                        previous,
                        inverse_masses,
                        [v, face[0] as usize, face[1] as usize, face[2] as usize],
                        thickness,
                    );
                }
            }
            let bounds: Vec<_> = self
                .edges
                .iter()
                .map(|edge| {
                    Aabb::from_points(
                        edge.iter()
                            .flat_map(|&i| [positions[i as usize], previous[i as usize]]),
                    )
                    .expand(thickness)
                })
                .collect();
            let tree = Bvh::build(&bounds);
            for (i, edge) in self.edges.iter().enumerate() {
                if edge.iter().all(|&v| inverse_masses[v as usize] == 0.0) {
                    continue;
                }
                candidates.clear();
                tree.query_aabb(&bounds, &bounds[i], |j| {
                    let j = j as usize;
                    if j > i
                        || self.edges[j]
                            .iter()
                            .all(|&v| inverse_masses[v as usize] == 0.0)
                    {
                        candidates.push(j);
                    }
                });
                for &j in &candidates {
                    let other = self.edges[j];
                    if edge.iter().any(|a| {
                        other
                            .iter()
                            .any(|b| a == b || self.neighbours[*a as usize].contains(b))
                    }) {
                        continue;
                    }
                    contacts += resolve(
                        Pair::EdgeEdge,
                        positions,
                        previous,
                        inverse_masses,
                        [
                            edge[0] as usize,
                            edge[1] as usize,
                            other[0] as usize,
                            other[1] as usize,
                        ],
                        thickness,
                    );
                }
            }
        }
        contacts
    }
}

fn resolve(
    pair: Pair,
    positions: &mut [Vec3],
    previous: &[Vec3],
    masses: &[f32],
    ids: [usize; 4],
    thickness: f32,
) -> usize {
    let start = ids.map(|i| previous[i]);
    let end = ids.map(|i| positions[i]);
    // The CCD guard is inside the resting contact shell. This lets touching
    // cloth slide tangentially without returning time zero on every step.
    let contact = ccd::sweep(pair, start, end, thickness * 0.5)
        .unwrap_or_else(|| ccd::proximity(pair, end, start));
    debug_assert!((0.0..=1.0).contains(&contact.time));
    let separation = ids
        .iter()
        .zip(contact.weights)
        .fold(Vec3::default(), |sum, (&i, w)| sum + positions[i] * w)
        .dot(contact.normal);
    let depth = thickness - separation;
    if depth <= 0.0 {
        return 0;
    }
    let denominator: f32 = ids
        .iter()
        .zip(contact.weights)
        .map(|(&i, w)| masses[i] * w * w)
        .sum();
    if denominator <= 1e-12 {
        return 0;
    }
    for (i, w) in ids.into_iter().zip(contact.weights) {
        positions[i] = positions[i] + contact.normal * (depth * w * masses[i] / denominator);
    }
    1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vertex_crossing_triangle_interior_is_returned_to_approach_side() {
        let mut p = vec![
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., -0.1, 0.),
        ];
        let mut previous = p.clone();
        previous[3].y = 0.1;
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2]]);
        contacts.solve(&mut p, &previous, &[0., 0., 0., 1.], 0.005, 2);
        assert!(p[3].y >= 0.0049);
        assert_eq!(p[0], previous[0]);
    }
    #[test]
    fn adjacent_faces_are_not_inflated() {
        let mut p = vec![
            Vec3::new(0., 0., 0.),
            Vec3::new(1., 0., 0.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., 1.),
        ];
        let previous = p.clone();
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2], [1, 3, 2]]);
        assert_eq!(contacts.solve(&mut p, &previous, &[1.; 4], 0.01, 2), 0);
        assert_eq!(p, previous);
    }
    #[test]
    fn edge_interiors_separate_without_close_endpoints() {
        let mut p = vec![
            Vec3::new(-1., 0., 0.),
            Vec3::new(1., 0., 0.),
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0.001, -1.),
            Vec3::new(0., 0.001, 1.),
            Vec3::new(1., 0.001, 1.),
        ];
        let previous = p.clone();
        let contacts = SurfaceContacts::new(6, vec![[0, 1, 2], [3, 4, 5]]);
        assert!(contacts.solve(&mut p, &previous, &[1.; 6], 0.01, 4) > 0);
        assert!(
            ccd::proximity(
                Pair::EdgeEdge,
                [p[0], p[1], p[3], p[4]],
                [previous[0], previous[1], previous[3], previous[4]]
            )
            .distance
                > 0.008
        );
    }

    #[test]
    fn swept_edge_bounds_find_fast_crossings_and_preserve_pinned_edges() {
        let previous = vec![
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, -1.0),
            Vec3::new(0.0, 10.0, -1.0),
            Vec3::new(0.0, 10.0, 1.0),
            Vec3::new(1.0, 10.0, 1.0),
        ];
        let mut end = previous.clone();
        for p in &mut end[3..] {
            p.y = -10.0;
        }
        let contacts = SurfaceContacts::new(6, vec![[0, 1, 2], [3, 4, 5]]);
        contacts.solve(
            &mut end,
            &previous,
            &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            0.004,
            8,
        );
        assert_eq!(&end[..3], &previous[..3]);
        let remaining = ccd::sweep(
            Pair::EdgeEdge,
            [previous[0], previous[1], previous[3], previous[4]],
            [end[0], end[1], end[3], end[4]],
            0.001,
        );
        assert!(
            remaining.is_none(),
            "edge still crosses: {remaining:?}; positions {end:?}"
        );
    }

    #[test]
    fn translating_body_pushes_a_stationary_vertex_to_the_approach_side() {
        let previous = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(0.0, 1.0, -1.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let mut end = previous.clone();
        for p in &mut end[..3] {
            p.z = 1.0;
        }
        let body_end = end[..3].to_vec();
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2]]);
        contacts.solve(&mut end, &previous, &[0.0, 0.0, 0.0, 1.0], 0.004, 4);
        assert!(
            end[3].z >= 1.0039,
            "body passed through vertex: {:?}",
            end[3]
        );
        assert_eq!(&end[..3], &body_end);
    }

    #[test]
    fn corrected_deforming_triangle_path_does_not_cross_the_vertex() {
        let previous = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(0.0, 1.0, -0.5),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let mut end = previous.clone();
        end[0].z = 0.5;
        end[1].z = 1.5;
        end[2].z = 1.0;
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2]]);
        contacts.solve(&mut end, &previous, &[0.0, 0.0, 0.0, 1.0], 0.004, 8);
        assert!(
            ccd::sweep(
                Pair::VertexTriangle,
                [previous[3], previous[0], previous[1], previous[2]],
                [end[3], end[0], end[1], end[2]],
                0.001
            )
            .is_none()
        );
    }
}
