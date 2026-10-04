use super::*;

type Sample = (Vec3, [f32; 3]);

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
                for (point, _) in clipped(points, [a, b, c], self.inward) {
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
        masses: &[f32],
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
                for (_, weights) in clipped(points, [a, b, c], self.inward) {
                    let point = (0..3).fold(Vec3::default(), |p, i| {
                        p + positions[face[i] as usize] * weights[i]
                    });
                    let depth = self.clearance - (point - a).dot(normal);
                    if depth <= PROJECTION_TOLERANCE {
                        continue;
                    }
                    let denominator: f32 = (0..3)
                        .map(|i| weights[i].powi(2) * masses[face[i] as usize])
                        .sum();
                    if denominator <= 0.0 {
                        continue;
                    }
                    let incoming = (0..3)
                        .map(|i| velocities[face[i] as usize].dot(normal) * weights[i])
                        .sum::<f32>()
                        .min(0.0);
                    for i in 0..3 {
                        let vertex = face[i] as usize;
                        let share = weights[i] * masses[vertex] / denominator;
                        positions[vertex] += normal * (depth * share);
                        velocities[vertex] -= normal * (incoming * share);
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

fn clipped(cloth: [Vec3; 3], plate: [Vec3; 3], inward: Vec3) -> Vec<Sample> {
    let mut polygon: Vec<Sample> = cloth
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let mut w = [0.0; 3];
            w[i] = 1.0;
            (p, w)
        })
        .collect();
    for edge in 0..3 {
        let a = plate[edge];
        let direction = plate[(edge + 1) % 3] - a;
        let signed = |point: Vec3| direction.cross(point - a).dot(inward);
        let mut output = Vec::new();
        for i in 0..polygon.len() {
            let (p, pw) = polygon[i];
            let (q, qw) = polygon[(i + 1) % polygon.len()];
            let (dp, dq) = (signed(p), signed(q));
            if dp >= 0.0 {
                output.push((p, pw));
            }
            if (dp < 0.0) != (dq < 0.0) {
                let t = dp / (dp - dq);
                output.push((
                    p + (q - p) * t,
                    std::array::from_fn(|k| pw[k] + (qw[k] - pw[k]) * t),
                ));
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
        assert!(layer.project_surface(&mut points, &mut velocities, &[1.0; 3], &[[0, 1, 2]]));
        assert!(layer.surface_residual(&points, &[[0, 1, 2]]) <= CLEARANCE_TOLERANCE + 1e-6);
        for (point, _) in clipped(
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
