//! Cloth surface contacts with swept vertex/triangle and edge/edge detection.
//! All garments share one particle array. Only shared vertices and sewn copies are excluded.
//! Zero inverse mass means prescribed motion, not a stationary collider:
//! previous and current positions must both be supplied for animated bodies.
use crate::ccd::{self, Pair};
use crate::{ParticleInverseMass, ParticleMobility};
use fabelgeist_bvh::{Aabb, Bvh};
use fabelgeist_math::Vec3;
use std::collections::BTreeSet;

mod broad_phase;
mod solve;
use broad_phase::FixedBounds;

pub struct SurfaceContacts {
    fixed_bounds: Option<FixedBounds>,
    static_positions: Vec<Vec3>,
    static_clearance: f32,
    faces: Vec<[u32; 3]>,
    edges: Vec<[u32; 2]>,
    seam_copies: Vec<BTreeSet<u32>>,
}

impl SurfaceContacts {
    pub fn new(count: usize, faces: Vec<[u32; 3]>) -> Self {
        let seam_copies = vec![BTreeSet::new(); count];
        let mut edges = BTreeSet::new();
        for face in &faces {
            for k in 0..3 {
                let (a, b) = (face[k], face[(k + 1) % 3]);
                assert!((a as usize) < count && (b as usize) < count);
                edges.insert([a.min(b), a.max(b)]);
            }
        }
        Self {
            fixed_bounds: None,
            static_positions: Vec::new(),
            static_clearance: 0.0,
            faces,
            edges: edges.into_iter().collect(),
            seam_copies,
        }
    }

    /// Exclude shared seam vertices, never their neighbouring faces.
    /// Seam copies represent one physical vertex even before sewing settles.
    pub fn with_seams(mut self, seams: &[[u32; 2]]) -> Self {
        let count = self.seam_copies.len();
        let mut groups: Vec<usize> = (0..count).collect();
        for &[a, b] in seams {
            let from = groups[b as usize];
            let to = groups[a as usize];
            for group in &mut groups {
                if *group == from {
                    *group = to;
                }
            }
        }
        for i in 0..count {
            self.seam_copies[i] = (0..count)
                .filter(|&j| groups[i] == groups[j])
                .map(|j| j as u32)
                .collect();
        }
        self
    }

    /// Include fixed obstacle triangles in the same swept surface solve.
    /// Their interiors and edges constrain cloth even when no cloth vertex
    /// touches the obstacle; zero inverse mass keeps the obstacle unchanged.
    pub fn set_static_surface(&mut self, positions: &[Vec3], faces: &[[u32; 3]], clearance: f32) {
        assert!(clearance.is_finite() && clearance >= 0.0);
        let count = self.seam_copies.len() - self.static_positions.len();
        let mut triangles: Vec<_> = self
            .faces
            .iter()
            .copied()
            .filter(|f| f.iter().all(|&i| (i as usize) < count))
            .collect();
        triangles.extend(faces.iter().map(|f| f.map(|i| i + count as u32)));
        let mut next = Self::new(count + positions.len(), triangles);
        next.seam_copies[..count].clone_from_slice(&self.seam_copies[..count]);
        next.fixed_bounds = FixedBounds::new(&next.faces, &next.edges, count, positions);
        next.static_positions = positions.to_vec();
        next.static_clearance = clearance;
        *self = next;
    }

    /// Project a completed GPU substep before the next integration begins.
    /// Particle spheres alone cannot detect triangle-interior crossings.
    /// Resolve relative normal velocity with mass-weighted impulses; uploading
    /// through `write_positions` would incorrectly reset every velocity.
    /// `interval_start` holds positions from before several GPU substeps;
    /// without it only the last substep's motion is swept.
    pub async fn project_particles(
        &self,
        context: &fabelgeist_gpu::globals::WgpuContext,
        particles: &fabelgeist_xpbd::Particles,
        thickness: f32,
        iterations: u32,
        interval_start: Option<&[Vec3]>,
    ) -> anyhow::Result<usize> {
        let mut previous = match interval_start {
            Some(start) => {
                anyhow::ensure!(
                    start.len() == particles.count() as usize,
                    "interval start does not match the particle count"
                );
                start.to_vec()
            }
            None => {
                let records = fabelgeist_xpbd::ParticlePositions::from(
                    particles
                        .previous
                        .read::<fabelgeist_xpbd::ParticlePositionRecord>(context)
                        .await?,
                );
                records
                    .positions()
                    .take(particles.count() as usize)
                    .collect()
            }
        };
        let predicted = particles.read_positions(context).await?;
        let mut corrected = predicted.clone();
        let mut velocities = particles.read_velocities(context).await?;
        let count = corrected.len();
        let mut masses = particles.inverse_masses().to_vec();
        previous.extend_from_slice(&self.static_positions);
        corrected.extend_from_slice(&self.static_positions);
        masses.resize(corrected.len(), ParticleInverseMass::PINNED);
        velocities.resize(corrected.len(), Vec3::default());
        let contacts = self.solve_inner(
            &mut corrected,
            &previous,
            &masses,
            thickness,
            iterations,
            Some(&mut velocities),
        );
        if contacts > 0 {
            particles.positions.write(
                context,
                fabelgeist_xpbd::ParticlePositions::new(
                    &corrected[..count],
                    particles.inverse_masses(),
                )?
                .upload(),
            )?;
            particles.velocities.write(
                context,
                fabelgeist_xpbd::ParticleVelocities::from(&velocities[..count]).upload(),
            )?;
        }
        Ok(contacts)
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
        inverse_masses: &[ParticleInverseMass],
        thickness: f32,
        iterations: u32,
    ) -> usize {
        if !self.static_positions.is_empty() {
            let count = positions.len();
            let mut full = positions.to_vec();
            let mut start = previous.to_vec();
            let mut masses = inverse_masses.to_vec();
            full.extend_from_slice(&self.static_positions);
            start.extend_from_slice(&self.static_positions);
            masses.resize(full.len(), ParticleInverseMass::PINNED);
            let contacts =
                self.solve_inner(&mut full, &start, &masses, thickness, iterations, None);
            positions.copy_from_slice(&full[..count]);
            return contacts;
        }
        self.solve_inner(
            positions,
            previous,
            inverse_masses,
            thickness,
            iterations,
            None,
        )
    }
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
        contacts.solve(
            &mut p,
            &previous,
            &[0.0.into(), 0.0.into(), 0.0.into(), 1.0.into()],
            0.005,
            2,
        );
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
        assert_eq!(
            contacts.solve(&mut p, &previous, &[1.0.into(); 4], 0.01, 2),
            0
        );
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
        assert!(contacts.solve(&mut p, &previous, &[1.0.into(); 6], 0.01, 4) > 0);
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
            &[
                0.0.into(),
                0.0.into(),
                0.0.into(),
                1.0.into(),
                1.0.into(),
                1.0.into(),
            ],
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
        contacts.solve(
            &mut end,
            &previous,
            &[0.0.into(), 0.0.into(), 0.0.into(), 1.0.into()],
            0.004,
            4,
        );
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
        contacts.solve(
            &mut end,
            &previous,
            &[0.0.into(), 0.0.into(), 0.0.into(), 1.0.into()],
            0.004,
            8,
        );
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

#[cfg(test)]
mod gpu_contact_regression {
    use super::*;

    #[tokio::test]
    async fn triangle_interior_crossing_preserves_slide_and_pins() -> anyhow::Result<()> {
        let context = fabelgeist_gpu::globals::WgpuContext::new().await?;
        let start = vec![
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., 0.1, 0.),
        ];
        let masses = [0., 0., 0., 1.].map(ParticleInverseMass::from);
        let particles = fabelgeist_xpbd::Particles::from_positions(&context, &start, &masses)?;
        let mut end = start.clone();
        end[3] = Vec3::new(0.02, -0.1, 0.);
        particles.positions.write(
            &context,
            fabelgeist_xpbd::ParticlePositions::new(&end, &masses)?.upload(),
        )?;
        let mut velocity = vec![Vec3::default(); 4];
        velocity[3] = Vec3::new(1., -20., 0.);
        particles.velocities.write(
            &context,
            fabelgeist_xpbd::ParticleVelocities::from(velocity.as_slice()).upload(),
        )?;
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2]]);
        assert!(
            contacts
                .project_particles(&context, &particles, 0.005, 4, None)
                .await?
                > 0
        );
        let result = particles.read_positions(&context).await?;
        let velocity = particles.read_velocities(&context).await?;
        assert!(result[3].y >= 0.0049);
        assert!((result[3].x - end[3].x).abs() < 1e-5);
        assert!((velocity[3].x - 1.).abs() < 1e-5);
        assert!(velocity[3].y >= -1e-5);
        assert_eq!(&result[..3], &start[..3]);
        Ok(())
    }

    #[tokio::test]
    async fn interval_start_catches_a_crossing_before_the_last_substep() -> anyhow::Result<()> {
        let context = fabelgeist_gpu::globals::WgpuContext::new().await?;
        let mut before_last = vec![
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., -0.05, 0.),
        ];
        let masses = [0., 0., 0., 1.].map(ParticleInverseMass::from);
        let particles =
            fabelgeist_xpbd::Particles::from_positions(&context, &before_last, &masses)?;
        let mut end = before_last.clone();
        end[3].y = -0.1;
        particles.positions.write(
            &context,
            fabelgeist_xpbd::ParticlePositions::new(&end, &masses)?.upload(),
        )?;
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2]]);
        // The last substep alone stays below the triangle.
        assert_eq!(
            contacts
                .project_particles(&context, &particles, 0.005, 4, None)
                .await?,
            0
        );
        before_last[3].y = 0.1;
        assert!(
            contacts
                .project_particles(&context, &particles, 0.005, 4, Some(&before_last))
                .await?
                > 0
        );
        assert!(particles.read_positions(&context).await?[3].y >= 0.0049);
        Ok(())
    }

    #[test]
    fn sewn_panel_copies_do_not_repel_each_other() {
        let mut positions = vec![
            Vec3::new(0., 0., 0.),
            Vec3::new(1., 0., 0.),
            Vec3::new(0., 1., 0.),
            Vec3::new(1., 0., 0.),
            Vec3::new(1., 1., 0.),
            Vec3::new(0., 1., 0.),
        ];
        let previous = positions.clone();
        let contacts =
            SurfaceContacts::new(6, vec![[0, 1, 2], [3, 4, 5]]).with_seams(&[[1, 3], [2, 5]]);
        assert_eq!(
            contacts.solve(&mut positions, &previous, &[1.0.into(); 6], 0.005, 4),
            0
        );
        assert_eq!(positions, previous);
    }
}

#[cfg(test)]
mod relative_velocity_regression {
    use super::*;
    #[test]
    fn dynamic_contact_preserves_momentum_and_common_velocity() {
        let previous = vec![
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., 0.1, 0.),
        ];
        let mut end = previous.clone();
        end[3].y = -0.1;
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2]]);
        let solve = |boost: Vec3| -> Vec<Vec3> {
            let mut positions = end.clone();
            let mut velocities = vec![boost; 4];
            velocities[3].y -= 2.;
            let momentum = velocities
                .iter()
                .copied()
                .fold(Vec3::default(), |a, b| a + b);
            assert!(
                contacts.solve_inner(
                    &mut positions,
                    &previous,
                    &[1.0.into(); 4],
                    0.005,
                    1,
                    Some(&mut velocities)
                ) > 0
            );
            let after = velocities
                .iter()
                .copied()
                .fold(Vec3::default(), |a, b| a + b);
            assert!((after - momentum).length() < 1e-5);
            velocities
        };
        let boost = Vec3::new(3., 7., -2.);
        for (base, shifted) in solve(Vec3::default()).iter().zip(solve(boost)) {
            assert!((shifted - *base - boost).length() < 1e-5);
        }
    }
}

#[cfg(test)]
mod fold_regression {
    use super::*;
    #[test]
    fn opposite_vertex_cannot_fold_through_its_adjacent_triangle() {
        let previous = vec![
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., 0.1, 0.),
        ];
        let mut positions = previous.clone();
        positions[3].y = -0.1;
        let contacts = SurfaceContacts::new(4, vec![[0, 1, 2], [1, 3, 2]]);
        assert!(
            contacts.solve(
                &mut positions,
                &previous,
                &[0.0.into(), 0.0.into(), 0.0.into(), 1.0.into()],
                0.005,
                4
            ) > 0
        );
        assert!(positions[3].y >= 0.0049);
    }
}

#[cfg(test)]
mod obstacle_regression {
    use super::*;
    #[test]
    fn fixed_triangle_interior_blocks_a_cloth_face_without_vertex_overlap() {
        let previous = vec![
            Vec3::new(-1.0, -1.0, 0.1),
            Vec3::new(1.0, -1.0, 0.1),
            Vec3::new(0.0, 1.0, 0.1),
        ];
        let mut positions: Vec<_> = previous.iter().map(|p| Vec3::new(p.x, p.y, -0.1)).collect();
        let mut contacts = SurfaceContacts::new(3, vec![[0, 1, 2]]);
        contacts.set_static_surface(
            &[
                Vec3::new(-0.1, -0.1, 0.0),
                Vec3::new(0.1, -0.1, 0.0),
                Vec3::new(0.0, 0.1, 0.0),
            ],
            &[[0, 1, 2]],
            0.003,
        );
        assert!(contacts.solve(&mut positions, &previous, &[1.0.into(); 3], 0.003, 8) > 0);
        let normal = (positions[1] - positions[0]).cross(positions[2] - positions[0]);
        for point in &contacts.static_positions {
            assert!((*point - positions[0]).dot(normal) / normal.length() <= -0.0025);
        }
    }
}
#[cfg(test)]
mod fixed_bounds_regression {
    use super::*;
    #[test]
    fn replacing_and_removing_an_obstacle_rebuilds_cached_bounds() {
        let start = vec![
            Vec3::new(-0.1, 0.1, -0.1),
            Vec3::new(0.1, 0.1, -0.1),
            Vec3::new(0.0, 0.1, 0.1),
        ];
        let end: Vec<_> = start.iter().map(|p| Vec3::new(p.x, -0.1, p.z)).collect();
        let mut contacts = SurfaceContacts::new(3, vec![[0, 1, 2]]);
        let body = [
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
        ];
        contacts.set_static_surface(&body, &[[0, 1, 2]], 0.003);
        let mut points = end.clone();
        contacts.solve(&mut points, &start, &[1.0.into(); 3], 0.003, 4);
        assert!(points.iter().all(|p| p.y >= 0.0029));
        contacts.set_static_surface(&body.map(|p| Vec3::new(p.x, -1., p.z)), &[[0, 1, 2]], 0.003);
        points.clone_from(&end);
        assert_eq!(
            contacts.solve(&mut points, &start, &[1.0.into(); 3], 0.003, 4),
            0
        );
        assert_eq!(points, end);
        contacts.set_static_surface(&[], &[], 0.0);
        assert_eq!(
            contacts.solve(&mut points, &start, &[1.0.into(); 3], 0.003, 4),
            0
        );
    }
}
#[cfg(test)]
mod obstacle_clearance_regression {
    use super::*;
    #[test]
    fn resting_cloth_reaches_body_ease_without_inflating_self_contacts() {
        let start = vec![
            Vec3::new(-0.1, -0.1, 0.002),
            Vec3::new(0.1, -0.1, 0.002),
            Vec3::new(0., 0.1, 0.002),
        ];
        let mut points = start.clone();
        let mut contacts = SurfaceContacts::new(3, vec![[0, 1, 2]]);
        contacts.set_static_surface(
            &[
                Vec3::new(-1., -1., 0.),
                Vec3::new(1., -1., 0.),
                Vec3::new(0., 1., 0.),
            ],
            &[[0, 1, 2]],
            0.005,
        );
        assert!(contacts.solve(&mut points, &start, &[1.0.into(); 3], 0.0006, 4) > 0);
        assert!(points.iter().all(|p| p.z >= 0.00499));
    }
}
