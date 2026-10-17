//! Coarse cloned cloth preserves every cut edge and retained body source point.
use super::{SourcePoint, SurfaceCut};
use anyhow::{Result, ensure};
use bevy::math::DVec3;
use std::collections::{BTreeMap, BTreeSet};

const CLOTH_PANEL_FACE_TARGET: usize = 500;
/// RMS distance to accumulated source planes, in canonical body metres.
const CLOTH_PANEL_PLANE_ERROR_METRES: f64 = 0.003;

#[derive(Clone, Default)]
struct Quadric {
    matrix: [[f64; 4]; 4],
    weight: f64,
}

struct ClothPatch {
    points: Vec<SourcePoint>,
    positions: Vec<DVec3>,
    faces: Vec<[u32; 3]>,
    quadrics: Vec<Quadric>,
}

struct Collapse {
    removed: u32,
    retained: u32,
    error: f64,
}

impl SurfaceCut {
    /// Native body boundary: sample once on the reference wearer, then retain
    /// the resulting source points/connectivity for every morph realization.
    /// Boundary vertices stay exact; only interior edges may collapse.
    pub(crate) fn sample_cloth(self, body: &[[f32; 3]], body_faces: &[[u32; 3]]) -> Result<Self> {
        let positions = self
            .points
            .iter()
            .map(|point| {
                let source = body_faces[point.triangle].map(|v| body[v as usize]);
                DVec3::from_array(super::interpolate(source, point.weights).map(f64::from))
            })
            .collect::<Vec<_>>();
        ensure!(
            positions.iter().all(|p| p.is_finite()),
            "nonfinite cloth source"
        );
        let mut patch = ClothPatch {
            quadrics: vec![Quadric::default(); positions.len()],
            positions,
            points: self.points,
            faces: self.faces,
        };
        patch.accumulate_planes();
        while patch.faces.len() > CLOTH_PANEL_FACE_TARGET {
            let Some(collapse) = patch.next_collapse() else {
                break;
            };
            let removed = patch.quadrics[collapse.removed as usize].clone();
            patch.quadrics[collapse.retained as usize].add(&removed);
            for face in &mut patch.faces {
                for vertex in face {
                    if *vertex == collapse.removed {
                        *vertex = collapse.retained;
                    }
                }
            }
            patch.faces.retain(|[a, b, c]| a != b && b != c && c != a);
        }
        Ok(patch.compact())
    }
}

impl Quadric {
    fn add(&mut self, other: &Self) {
        for row in 0..4 {
            for column in 0..4 {
                self.matrix[row][column] += other.matrix[row][column];
            }
        }
        self.weight += other.weight;
    }

    fn error(&self, other: &Self, point: DVec3) -> f64 {
        let p = [point.x, point.y, point.z, 1.];
        let mut cost = 0.;
        for row in 0..4 {
            for column in 0..4 {
                cost += p[row] * p[column] * (self.matrix[row][column] + other.matrix[row][column]);
            }
        }
        cost.max(0.) / (self.weight + other.weight)
    }
}

impl ClothPatch {
    fn accumulate_planes(&mut self) {
        for face in &self.faces {
            let p = face.map(|v| self.positions[v as usize]);
            let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
            let plane = [normal.x, normal.y, normal.z, -normal.dot(p[0])];
            let quadric = Quadric {
                matrix: std::array::from_fn(|i| std::array::from_fn(|j| plane[i] * plane[j])),
                weight: 1.,
            };
            for &vertex in face {
                self.quadrics[vertex as usize].add(&quadric);
            }
        }
    }

    fn next_collapse(&self) -> Option<Collapse> {
        let mut edges = BTreeMap::<[u32; 2], Vec<u32>>::new();
        let mut adjacent = vec![BTreeSet::new(); self.points.len()];
        for &[a, b, c] in &self.faces {
            for (mut edge, opposite) in [([a, b], c), ([b, c], a), ([c, a], b)] {
                adjacent[edge[0] as usize].insert(edge[1]);
                adjacent[edge[1] as usize].insert(edge[0]);
                edge.sort_unstable();
                edges.entry(edge).or_default().push(opposite);
            }
        }
        let boundary = edges
            .iter()
            .filter(|(_, faces)| faces.len() != 2)
            .flat_map(|(edge, _)| *edge)
            .collect::<BTreeSet<_>>();
        let mut best: Option<Collapse> = None;
        for (edge, opposite) in &edges {
            if opposite.len() != 2 {
                continue;
            }
            let common = adjacent[edge[0] as usize]
                .intersection(&adjacent[edge[1] as usize])
                .copied()
                .collect::<BTreeSet<_>>();
            if common != opposite.iter().copied().collect() {
                continue;
            }
            for [removed, retained] in [*edge, [edge[1], edge[0]]] {
                if boundary.contains(&removed) {
                    continue;
                }
                let error = self.quadrics[removed as usize].error(
                    &self.quadrics[retained as usize],
                    self.positions[retained as usize],
                );
                if error > CLOTH_PANEL_PLANE_ERROR_METRES.powi(2)
                    || best.as_ref().is_some_and(|best| error >= best.error)
                    || !self.preserves_faces(removed, retained)
                {
                    continue;
                }
                best = Some(Collapse {
                    removed,
                    retained,
                    error,
                });
            }
        }
        best
    }

    fn preserves_faces(&self, removed: u32, retained: u32) -> bool {
        self.faces
            .iter()
            .filter(|face| face.contains(&removed) && !face.contains(&retained))
            .all(|face| {
                let before = face.map(|v| self.positions[v as usize]);
                let after =
                    face.map(|v| self.positions[if v == removed { retained } else { v } as usize]);
                let old_normal = (before[1] - before[0]).cross(before[2] - before[0]);
                let new_normal = (after[1] - after[0]).cross(after[2] - after[0]);
                old_normal.dot(new_normal) > 0.
            })
    }

    fn compact(self) -> SurfaceCut {
        let mut remap = BTreeMap::new();
        let points = self
            .faces
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|old| {
                remap.insert(old, remap.len() as u32);
                self.points[old as usize]
            })
            .collect();
        let faces = self
            .faces
            .into_iter()
            .map(|face| face.map(|v| remap[&v]))
            .collect();
        SurfaceCut { points, faces }
    }
}
