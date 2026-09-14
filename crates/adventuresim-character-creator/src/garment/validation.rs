use super::*;

impl DrapedGarment {
    pub fn validate_contacts(&self, body: &fabelgeist_bvh::TriangleBvh) -> Result<()> {
        let sewn = SewnSurface::from_positions(&self.positions, &self.faces);
        let points = sewn
            .positions(&self.positions)
            .into_iter()
            .map(vector)
            .collect();
        let cloth = fabelgeist_bvh::TriangleBvh::new(points, sewn.faces);
        validate_surface(&cloth, body)
    }

    /// Check the outward side and requested ease against the closed wearer.
    pub fn validate_body_clearance(
        &self,
        body: &fabelgeist_bvh::TriangleBvh,
        armor: Option<&fabelgeist_armor::Armor>,
    ) -> Result<()> {
        let points: Vec<_> = self.positions.iter().copied().map(vector).collect();
        let margin =
            self.fabric.body_ease_cm(armor) * 0.01 + self.fabric.fabric().particle_radius();
        anyhow::ensure!(
            super::finish::body_residual(&points, body, margin)
                <= fabelgeist_cloth::outer_layer::CLEARANCE_TOLERANCE,
            "garment violates wearer clearance"
        );
        Ok(())
    }
}

pub(super) fn validate_surface(
    cloth: &fabelgeist_bvh::TriangleBvh,
    body: &fabelgeist_bvh::TriangleBvh,
) -> Result<()> {
    anyhow::ensure!(
        !super::armor::edges_cross(&cloth, body) && !super::armor::edges_cross(body, &cloth),
        "garment triangles intersect the wearer or an inner garment"
    );
    for (index, face) in cloth.triangles.iter().enumerate() {
        for edge in 0..3 {
            let a = cloth.positions[face[edge] as usize];
            let b = cloth.positions[face[(edge + 1) % 3] as usize];
            let delta = b - a;
            let length = delta.length();
            const ENDPOINT_TOLERANCE: f32 = 1e-6;
            if length <= ENDPOINT_TOLERANCE * 2.0 {
                continue;
            }
            let direction = delta / length;
            let ray = fabelgeist_bvh::Ray::new(a + direction * ENDPOINT_TOLERANCE, direction);
            let hit = cloth
                .bvh
                .raycast(&ray, length - ENDPOINT_TOLERANCE * 2.0, |other, limit| {
                    let other_face = cloth.triangles[other as usize];
                    if other as usize == index
                        || [face[edge], face[(edge + 1) % 3]]
                            .iter()
                            .any(|i| other_face.contains(i))
                    {
                        return None;
                    }
                    let (x, y, z) = cloth.triangle(other);
                    fabelgeist_bvh::aabb::ray_triangle(&ray, x, y, z, limit)
                });
            anyhow::ensure!(hit.is_none(), "garment surface intersects itself");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_vertex_does_not_hide_a_crossing_at_the_opposite_edge() {
        let cloth = fabelgeist_bvh::TriangleBvh::new(
            vec![
                [0., 0., 0.],
                [2., 0., 0.],
                [0., 2., 0.],
                [2., 2., -1.],
                [2., 2., 1.],
            ]
            .into_iter()
            .map(vector)
            .collect(),
            vec![[0, 1, 2], [0, 3, 4]],
        );
        let body = fabelgeist_bvh::TriangleBvh::new(
            vec![[0., 0., -10.], [2., 0., -10.], [0., 2., -10.]]
                .into_iter()
                .map(vector)
                .collect(),
            vec![[0, 1, 2]],
        );
        assert!(validate_surface(&cloth, &body).is_err());
    }

    #[test]
    fn body_clearance_rejects_wrong_side_without_a_surface_crossing() {
        let body = fabelgeist_bvh::TriangleBvh::new(
            vec![[-2., -2., 0.], [2., -2., 0.], [0., 2., 0.]]
                .into_iter()
                .map(vector)
                .collect(),
            vec![[0, 1, 2]],
        );
        let mut cloth = DrapedGarment {
            preset: GarmentPreset::Shirt,
            name: "clearance fixture".into(),
            fabric: FabricPreset::Chainmail,
            positions: vec![[-0.1, 0., -0.01], [0.1, 0., -0.01], [0., 0.1, -0.01]],
            faces: vec![[0, 1, 2]],
            normals: vec![],
            texcoords: vec![],
            indices: vec![],
            weights: vec![],
            stage: DrapeStage::Placed,
        };
        assert!(cloth.validate_contacts(&body).is_ok());
        assert!(cloth.validate_body_clearance(&body, None).is_err());
        for p in &mut cloth.positions {
            p[2] = 0.05;
        }
        assert!(cloth.validate_body_clearance(&body, None).is_ok());
    }
}
