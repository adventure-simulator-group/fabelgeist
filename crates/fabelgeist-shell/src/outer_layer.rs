//! One-sided outer layers: normals point into the space occupied by the cloth.
use crate::{ParticleInverseMass, ParticleMobility};
use fabelgeist_bvh::TriangleBvh;
use fabelgeist_math::Vec3;
mod surface;

/// Maximum tolerated fitting residual in metres.
pub const CLEARANCE_TOLERANCE: f32 = 0.0005;

pub struct OuterLayer {
    surface: TriangleBvh,
    pub clearance: f32,
    inward: Vec3,
}

impl OuterLayer {
    pub fn new(positions: Vec<Vec3>, faces: Vec<[u32; 3]>, clearance: f32, inward: Vec3) -> Self {
        assert!(inward.is_finite() && inward.length() > 0.0);
        assert!(clearance.is_finite() && clearance >= 0.0);
        Self {
            surface: TriangleBvh::new(positions, faces),
            clearance,
            inward: inward / inward.length(),
        }
    }

    /// Find the first finite plate seen from inside the layer. All triangles
    /// must face the given inward direction. This supports open and overlapping
    /// plates without extending their edges into infinite collision planes.
    fn correction(&self, point: Vec3) -> Option<(Vec3, Vec3)> {
        let bounds = self.surface.bvh.bounds();
        let reach = (point - bounds.center()).length() + bounds.extent().length() + self.clearance;
        let origin = point + self.inward * reach;
        let ray = fabelgeist_bvh::Ray::new(origin, self.inward * -1.0);
        let (index, distance) = self.surface.raycast(&ray, reach * 2.0)?;
        let closest = origin - self.inward * distance;
        let (a, b, c) = self.surface.triangle(index);
        let raw = (b - a).cross(c - a);
        if raw.length() <= 1e-10 {
            return None;
        }
        let normal = raw / raw.length();
        let alignment = normal.dot(self.inward);
        if alignment <= 1e-5 {
            return None;
        }
        let signed = (point - closest).dot(normal);
        if signed >= self.clearance {
            return None;
        }
        Some((point + normal * (self.clearance - signed), normal))
    }

    /// Maximum remaining displacement needed to satisfy an outer layer.
    pub fn residual(&self, positions: &[Vec3]) -> f32 {
        positions
            .iter()
            .filter_map(|&p| self.correction(p).map(|(q, _)| (q - p).length()))
            .fold(0.0, f32::max)
    }

    /// Project host particles, preserving pinned vertices and sliding velocity.
    pub fn project(
        &self,
        positions: &mut [Vec3],
        velocities: &mut [Vec3],
        masses: &[ParticleInverseMass],
        faces: &[[u32; 3]],
    ) -> bool {
        assert_eq!(positions.len(), velocities.len());
        assert_eq!(positions.len(), masses.len());
        let mut changed = false;
        for (i, point) in positions.iter_mut().enumerate() {
            if masses[i].mobility() == ParticleMobility::Prescribed {
                continue;
            }
            if let Some((corrected, normal)) = self.correction(*point) {
                *point = corrected;
                let incoming = velocities[i].dot(normal).min(0.0);
                velocities[i] -= normal * incoming;
                changed = true;
            }
        }
        changed |= self.project_surface(positions, velocities, masses, faces);
        changed
    }

    pub async fn project_particles(
        &self,
        context: &fabelgeist_gpu::globals::WgpuContext,
        particles: &fabelgeist_xpbd::Particles,
        faces: &[[u32; 3]],
    ) -> Result<(), fabelgeist_xpbd::ParticleError> {
        let mut positions = particles.read_positions(context).await?;
        let mut velocities = particles.read_velocities(context).await?;
        let changed = self.project(
            &mut positions,
            &mut velocities,
            particles.inverse_masses(),
            faces,
        );
        if changed {
            particles.positions.write(
                context,
                fabelgeist_xpbd::ParticlePositions::new(&positions, particles.inverse_masses())?
                    .upload(),
            );
            particles.velocities.write(
                context,
                fabelgeist_xpbd::ParticleVelocities::from(velocities.as_slice()).upload(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sloped_contact_moves_normally_and_preserves_tangential_velocity() {
        let layer = OuterLayer::new(
            vec![
                Vec3::new(-1., -1., -1.),
                Vec3::new(0., 1., 0.),
                Vec3::new(1., -1., 1.),
            ],
            vec![[0, 1, 2]],
            0.003,
            Vec3::new(0., 0., -1.),
        );
        let start = Vec3::new(0., 0., 0.01);
        let mut points = [start];
        let tangent = Vec3::new(1., 0., 1.);
        let mut velocities = [tangent];
        layer.project(&mut points, &mut velocities, &[1.0.into()], &[]);
        assert!((points[0] - start).dot(tangent).abs() < 1e-6);
        assert!(points[0].x > start.x);
        assert!((velocities[0] - tangent).length() < 1e-6);
        assert!(layer.surface_residual(&points, &[]) <= CLEARANCE_TOLERANCE);
    }

    #[test]
    fn confines_the_surface_but_leaves_its_open_edges_free() {
        let layer = OuterLayer::new(
            vec![
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
            ],
            vec![[0, 1, 2]],
            0.003,
            Vec3::new(0.0, 0.0, -1.0),
        );
        let (corrected, _) = layer.correction(Vec3::new(0.0, 0.0, 0.1)).unwrap();
        assert!((corrected.z + 0.003).abs() < 1e-6);
        assert!(layer.correction(Vec3::new(0.0, 0.0, -0.1)).is_none());
        assert!(layer.correction(Vec3::new(2.0, 0.0, 0.1)).is_none());
    }
    #[test]
    fn an_open_edge_does_not_hide_an_overlapping_plate() {
        let layer = OuterLayer::new(
            vec![
                Vec3::new(-0.1, -0.1, 0.05),
                Vec3::new(0.0, 0.1, 0.05),
                Vec3::new(0.1, -0.1, 0.05),
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
            ],
            vec![[0, 1, 2], [3, 4, 5]],
            0.003,
            Vec3::new(0.0, 0.0, -1.0),
        );
        let (corrected, _) = layer.correction(Vec3::new(0.06, 0.0, 0.1)).unwrap();
        assert!((corrected.z + 0.003).abs() < 1e-6);
    }
    #[test]
    fn residual_uses_the_innermost_overlapping_plate() {
        let mut points = Vec::new();
        for z in [0.0029, -0.010] {
            points.extend([
                Vec3::new(-1.0, -1.0, z),
                Vec3::new(0.0, 1.0, z),
                Vec3::new(1.0, -1.0, z),
            ]);
        }
        let layer = OuterLayer::new(
            points,
            vec![[0, 1, 2], [3, 4, 5]],
            0.003,
            Vec3::new(0.0, 0.0, -1.0),
        );
        assert!((layer.residual(&[Vec3::default()]) - 0.013).abs() < 1e-6);
    }
}
