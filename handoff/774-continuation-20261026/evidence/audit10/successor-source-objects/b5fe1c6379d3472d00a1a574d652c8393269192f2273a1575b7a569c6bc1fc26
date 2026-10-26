use super::*;

use fabelgeist_xpbd::{
    BarycentricWeight, ClippingFraction, EffectiveInverseMass, IncomingNormalSpeed,
    ProjectionActivity, ProjectionDepth,
};
#[derive(Clone, Copy)]
struct ClippedSample {
    point: Vec3,
    weights: [BarycentricWeight; 3],
}

// Leave headroom for subsequent body and self-contact corrections.
const PROJECTION_TOLERANCE: f32 = CLEARANCE_TOLERANCE * 0.1;

impl OuterLayer {
    /// Maximum normal-clearance deficit anywhere on the cloth triangles.
    /// Checking only particle positions would miss a plate wholly inside a face.
    pub fn surface_residual(&self, positions: &[Vec3], faces: &[[u32; 3]]) -> f32 {
        let mut residual = self.residual(positions);
        for face in faces {
            let points = face.map(|i| positions[i as usize]);
            for index in self.overlaps(points) {
                let (a, b, c) = self.surface.triangle(index);
                let raw = (b - a).cross(c - a);
                if raw.length() <= 1e-10 {
                    continue;
                }
                let normal = raw / raw.length();
                let alignment = normal.dot(self.inward);
                if alignment <= 1e-5 {
                    continue;
                }
                for ClippedSample { point, .. } in clipped(points, [a, b, c], self.inward) {
                    residual = residual.max(self.clearance - (point - a).dot(normal));
                }
            }
        }
        residual
    }

    /// Constrain triangle interiors as well as vertices. In the layer's view,
    /// signed separation is linear over each overlap polygon, so its vertices
    /// include every possible minimum (including a plate edge through cloth).
    pub(super) fn project_surface(
        &self,
        positions: &mut [Vec3],
        velocities: &mut [Vec3],
        masses: &[ParticleInverseMass],
        faces: &[[u32; 3]],
    ) -> bool {
        let mut changed = false;
        for &face in faces {
            let points = face.map(|i| positions[i as usize]);
            let candidates = self.overlaps(points);
            for index in candidates {
                let points = face.map(|i| positions[i as usize]);
                let (a, b, c) = self.surface.triangle(index);
                let raw = (b - a).cross(c - a);
                if raw.length() <= 1e-10 {
                    continue;
                }
                let normal = raw / raw.length();
                let alignment = normal.dot(self.inward);
                if alignment <= 1e-5 {
                    continue;
                }
                for ClippedSample { weights, .. } in clipped(points, [a, b, c], self.inward) {
                    let point = (0..3).fold(Vec3::default(), |p: Vec3, i: usize| -> Vec3 {
                        p + weights[i].contribution(positions[face[i] as usize])
                    });
                    let depth = self.clearance - (point - a).dot(normal);
                    if depth <= PROJECTION_TOLERANCE {
                        continue;
                    }
                    let denominator: EffectiveInverseMass = (0..3)
                        .map(|i: usize| -> EffectiveInverseMass {
                            weights[i].inverse_response(masses[face[i] as usize])
                        })
                        .sum();
                    if denominator.layer_activity() == ProjectionActivity::Inactive {
                        continue;
                    }
                    let incoming = (0..3)
                        .map(|i: usize| -> IncomingNormalSpeed {
                            weights[i].weighted_normal_speed(velocities[face[i] as usize], normal)
                        })
                        .sum::<IncomingNormalSpeed>()
                        .incoming();
                    for i in 0..3 {
                        let vertex = face[i] as usize;
                        let share = weights[i].mass_share(masses[vertex], denominator);
                        positions[vertex] += share
                            .position_correction(ProjectionDepth::from(depth))
                            .along(normal);
                        velocities[vertex] -= share.speed_correction(incoming).along(normal);
                    }
                    changed = true;
                }
            }
        }
        changed
    }

    fn overlaps(&self, points: [Vec3; 3]) -> Vec<u32> {
        let bounds = self.surface.bvh.bounds();
        let reach = (points[0] - bounds.center()).length() + bounds.extent().length();
        let query = fabelgeist_bvh::Aabb::from_points(
            points
                .into_iter()
                .flat_map(|p| [p - self.inward * reach, p + self.inward * reach]),
        );
        let mut candidates = Vec::new();
        self.surface
            .bvh
            .candidates_aabb(&query, |i| candidates.push(i));
        candidates
    }
}

fn clipped(cloth: [Vec3; 3], plate: [Vec3; 3], inward: Vec3) -> Vec<ClippedSample> {
    let mut polygon: Vec<ClippedSample> = cloth
        .into_iter()
        .enumerate()
        .map(|(i, p): (usize, Vec3)| -> ClippedSample {
            let mut w = [BarycentricWeight::ZERO; 3];
            w[i] = BarycentricWeight::ONE;
            ClippedSample {
                point: p,
                weights: w,
            }
        })
        .collect();
    for edge in 0..3 {
        let a = plate[edge];
        let direction = plate[(edge + 1) % 3] - a;
        let signed = |point: Vec3| direction.cross(point - a).dot(inward);
        let mut output = Vec::new();
        for i in 0..polygon.len() {
            let ClippedSample {
                point: p,
                weights: pw,
            } = polygon[i];
            let ClippedSample {
                point: q,
                weights: qw,
            } = polygon[(i + 1) % polygon.len()];
            let (dp, dq) = (signed(p), signed(q));
            if dp >= 0.0 {
                output.push(ClippedSample {
                    point: p,
                    weights: pw,
                });
            }
            if (dp < 0.0) != (dq < 0.0) {
                let t = ClippingFraction::from(dp / (dp - dq));
                output.push(ClippedSample {
                    point: t.interpolate_point(p, q),
                    weights: std::array::from_fn(|k: usize| -> BarycentricWeight {
                        pw[k].interpolate(qw[k], t)
                    }),
                });
            }
        }
        polygon = output;
    }
    polygon
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plate_inside_a_cloth_triangle_cannot_be_missed_by_vertex_queries() {
        let layer = OuterLayer::new(
            vec![
                Vec3::new(-0.1, -0.1, 0.0),
                Vec3::new(0.0, 0.1, 0.0),
                Vec3::new(0.1, -0.1, 0.0),
            ],
            vec![[0, 1, 2]],
            0.003,
            Vec3::new(0.0, 0.0, -1.0),
        );
        let mut points = vec![
            Vec3::new(-1.0, -1.0, 0.1),
            Vec3::new(1.0, -1.0, 0.1),
            Vec3::new(0.0, 1.0, 0.1),
        ];
        assert!(points.iter().all(|&p| layer.correction(p).is_none()));
        assert!(layer.surface_residual(&points, &[[0, 1, 2]]) > 0.1);
        let mut velocities = vec![Vec3::default(); 3];
        assert!(layer.project_surface(
            &mut points,
            &mut velocities,
            &[1.0.into(); 3],
            &[[0, 1, 2]]
        ));
        assert!(layer.surface_residual(&points, &[[0, 1, 2]]) <= CLEARANCE_TOLERANCE + 1e-6);
        for ClippedSample { point, .. } in clipped(
            [points[0], points[1], points[2]],
            [
                layer.surface.positions[0],
                layer.surface.positions[1],
                layer.surface.positions[2],
            ],
            layer.inward,
        ) {
            assert!(point.z <= -0.003 + CLEARANCE_TOLERANCE);
        }
    }
}

#[cfg(test)]
mod mass_tests;
