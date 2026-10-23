//! Coarse cloned cloth retains source points and bounds cut-edge simplification.
use super::cloth_boundary_sampling::BoundaryChains;
use super::{CUT_VERTEX_SNAP_METRES, Plane, SourcePoint, SurfaceCut, dot};
use anyhow::{Result, ensure};
use bevy::math::DVec3;
use std::collections::{BTreeMap, BTreeSet};

const MINIMUM_SAMPLED_TRIANGLE_ALTITUDE_M: f64 = 0.00001;
const COPLANAR_TRIM_TOLERANCE_M: f64 = 0.000001;

#[path = "cloth_conditioning.rs"]
mod conditioning;

#[derive(Clone, Default)]
struct Quadric {
    matrix: [[f64; 4]; 4],
    weight: f64,
}

struct ClothPatch {
    points: Vec<SourcePoint>,
    source_owners: Vec<u32>,
    source_facets: Vec<BTreeSet<u32>>,
    positions: Vec<DVec3>,
    faces: Vec<[u32; 3]>,
    quadrics: Vec<Quadric>,
    boundary: BoundaryChains,
    protected: BTreeSet<u32>,
    apertures: Vec<super::ClothAperture>,
}

/// Sampled surface and the retained native vertex ownership after collapses.
pub(crate) struct SampledCloth {
    pub surface: SurfaceCut,
    pub ancestry: SamplingAncestry,
}

#[derive(Clone)]
pub(crate) struct SamplingAncestry {
    /// Original dense facet ordinals owned by each final coarse facet.
    pub source_facets: Vec<BTreeSet<u32>>,
}

struct Collapse {
    removed: u32,
    retained: u32,
    error: f64,
}

/// Source geometry at this numerical boundary is expressed in metres.
pub(crate) enum ClothSampling {
    /// Remove only numerical coplanar subdivisions in a construction trim.
    CoplanarTrim,
    /// Coarse anatomical fitting carrier, retaining fixed body ancestry.
    AnatomicalCarrier { preserve: BTreeSet<u32> },
}

#[derive(Clone, Copy, PartialEq)]
enum SamplingPhase {
    CutBoundary,
    Interior,
}

impl SurfaceCut {
    /// Native body boundary: sample once on the reference wearer, then retain
    /// the resulting source points/connectivity for every morph realization.
    /// Source points stay exact. Body-panel boundaries simplify within a
    /// bounded chord error before interior sampling, retaining the original
    /// accumulated planes throughout both phases. Finishing edges stay exact.
    pub(crate) fn sample_cloth(
        self,
        body: &[[f32; 3]],
        body_faces: &[[u32; 3]],
        sampling: ClothSampling,
        seam_planes: &[Plane],
    ) -> Result<SampledCloth> {
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
        let reference = positions
            .iter()
            .map(|p| p.as_vec3().to_array())
            .collect::<Vec<_>>();
        let mut protected = match &sampling {
            ClothSampling::AnatomicalCarrier { preserve } => preserve.clone(),
            _ => BTreeSet::new(),
        };
        for border in self.boundary_loops(&reference)? {
            for (index, &vertex) in border.iter().enumerate() {
                let previous = border[(index + border.len() - 1) % border.len()];
                let next = border[(index + 1) % border.len()];
                let on_plane = |vertex: u32, plane: &Plane| {
                    (dot(reference[vertex as usize], plane.normal) - plane.offset).abs()
                        <= CUT_VERTEX_SNAP_METRES
                };
                // Preserve a seam's end where the boundary leaves its plane.
                // Otherwise a legal chord approximation can erase the physical
                // junction needed by closures on that seam.
                if self
                    .apertures
                    .iter()
                    .any(|a| a.contains(reference[vertex as usize]))
                    || seam_planes.iter().any(|plane| {
                        on_plane(vertex, plane)
                            && (!on_plane(previous, plane) || !on_plane(next, plane))
                    })
                {
                    protected.insert(vertex);
                }
            }
        }
        let original_faces = self.faces.clone();
        let mut patch = ClothPatch {
            source_facets: Vec::new(),
            source_owners: (0..u32::try_from(positions.len())?).collect(),
            protected,
            apertures: self.apertures,
            quadrics: vec![Quadric::default(); positions.len()],
            boundary: BoundaryChains::new(
                &self.faces,
                &positions,
                sampling.boundary_chord_deviation_metres(),
            ),
            positions,
            points: self.points,
            faces: self.faces,
        };
        patch.accumulate_planes();
        while let Some(collapse) =
            patch.next_collapse(sampling.plane_error_squared(), SamplingPhase::CutBoundary)
        {
            patch.collapse(collapse);
        }
        while patch.faces.len() > sampling.face_target() {
            let Some(collapse) =
                patch.next_collapse(sampling.plane_error_squared(), SamplingPhase::Interior)
            else {
                break;
            };
            patch.collapse(collapse);
        }
        patch.assign_native_facets(&original_faces)?;
        if matches!(sampling, ClothSampling::AnatomicalCarrier { .. }) {
            patch.condition_edges();
        }
        Ok(patch.compact())
    }
}

impl ClothSampling {
    /// Native original-polyline to replacement-chord deviation on the reference
    /// wearer. Shared panels preserve their existing fitted shape; finishing
    /// strips have their own sewing-scale bound. Original samples stay bounded.
    fn boundary_chord_deviation_metres(&self) -> f64 {
        match self {
            Self::CoplanarTrim => COPLANAR_TRIM_TOLERANCE_M,
            Self::AnatomicalCarrier { .. } => 0.003,
        }
    }
    fn face_target(&self) -> usize {
        match self {
            Self::CoplanarTrim => 0,
            Self::AnatomicalCarrier { .. } => 320,
        }
    }
    /// Squared RMS distance to accumulated source planes, in metres squared.
    fn plane_error_squared(&self) -> f64 {
        let metres: f64 = match self {
            Self::CoplanarTrim => COPLANAR_TRIM_TOLERANCE_M,
            Self::AnatomicalCarrier { .. } => 0.003,
        };
        metres.powi(2)
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
    fn collapse(&mut self, collapse: Collapse) {
        self.boundary.collapse(collapse.removed, collapse.retained);
        for owner in &mut self.source_owners {
            if *owner == collapse.removed {
                *owner = collapse.retained;
            }
        }
        let removed = self.quadrics[collapse.removed as usize].clone();
        self.quadrics[collapse.retained as usize].add(&removed);
        for face in &mut self.faces {
            for vertex in face {
                if *vertex == collapse.removed {
                    *vertex = collapse.retained;
                }
            }
        }
        self.faces.retain(|[a, b, c]| a != b && b != c && c != a);
    }
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

    fn next_collapse(&self, plane_error_squared: f64, phase: SamplingPhase) -> Option<Collapse> {
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
            let expected = if phase == SamplingPhase::CutBoundary {
                1
            } else {
                2
            };
            if opposite.len() != expected {
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
                if self.protected.contains(&removed)
                    || (phase == SamplingPhase::Interior && boundary.contains(&removed))
                    || (phase == SamplingPhase::CutBoundary
                        && !self.boundary.permits(removed, retained, &self.positions))
                {
                    continue;
                }
                let error = self.quadrics[removed as usize].error(
                    &self.quadrics[retained as usize],
                    self.positions[retained as usize],
                );
                if error > plane_error_squared
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
                let longest_edge_squared = [
                    (after[1] - after[0]).length_squared(),
                    (after[2] - after[1]).length_squared(),
                    (after[0] - after[2]).length_squared(),
                ]
                .into_iter()
                .fold(0_f64, f64::max);
                old_normal.dot(new_normal) > 0.
                    && new_normal.length_squared()
                        >= longest_edge_squared * MINIMUM_SAMPLED_TRIANGLE_ALTITUDE_M.powi(2)
            })
    }

    fn assign_native_facets(&mut self, original: &[[u32; 3]]) -> Result<()> {
        self.source_facets = vec![BTreeSet::new(); self.faces.len()];
        for (source, face) in original.iter().enumerate() {
            let owners = face
                .iter()
                .map(|&v| self.source_owners[v as usize])
                .collect::<BTreeSet<_>>();
            let mut assigned = false;
            for (coarse, face) in self.faces.iter().enumerate() {
                // Before conditioning, a surviving triangle is exact; a
                // collapsed edge/vertex belongs only to its incident star.
                if owners.iter().all(|vertex| face.contains(vertex)) {
                    self.source_facets[coarse].insert(u32::try_from(source)?);
                    assigned = true;
                }
            }
            ensure!(
                assigned,
                "collapsed native facet has no surviving topological star"
            );
        }
        Ok(())
    }

    fn compact(self) -> SampledCloth {
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
        SampledCloth {
            surface: SurfaceCut {
                points,
                faces,
                apertures: self.apertures,
            },
            ancestry: SamplingAncestry {
                source_facets: self.source_facets,
            },
        }
    }
}
