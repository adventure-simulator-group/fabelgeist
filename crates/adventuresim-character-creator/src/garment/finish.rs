use super::*;

impl DrapedGarment {
    /// Reconcile the emitted sewn surface after body clearance and seam averaging.
    /// Returns what could not be resolved; the best fit found is kept either way.
    pub fn finish_armor(
        &mut self,
        armor: &fabelgeist_armor::Armor,
        body: &fabelgeist_bvh::TriangleBvh,
        margin: f32,
        settings: &ArmorFitSettings,
    ) -> Result<Vec<String>> {
        let surface = SewnSurface::from_positions(&self.positions, &self.faces);
        let previous: Vec<_> = surface
            .positions(&self.positions)
            .into_iter()
            .map(vector)
            .collect();
        let mut points = previous.clone();
        let masses = vec![1.0; points.len()];
        let mut velocities = vec![Vec3::default(); points.len()];
        let layer = super::armor::outer_layer(armor, self.fabric.fabric())?;
        let mut contacts = fabelgeist_cloth::surface_contact::SurfaceContacts::new(
            points.len(),
            surface.faces.clone(),
        );
        contacts.set_static_surface(&body.positions, &body.triangles, margin);
        // Dense body triangles and shallow plate contacts converge slowly.
        // Stop on the measured residuals, with a bounded budget for rejection.
        for _ in 0..settings.passes {
            // Each projection follows the last collision-checked surface.
            // Replaying every sweep from the original mesh prevents curved
            // sliding paths around contacting body and cloth triangles.
            let previous = points.clone();
            layer.project(&mut points, &mut velocities, &masses, &surface.faces);
            contacts.solve(
                &mut points,
                &previous,
                &masses,
                self.fabric.fabric().thickness,
                settings.contact_iterations,
            );
            if layer.surface_residual(&points, &surface.faces)
                <= fabelgeist_cloth::outer_layer::CLEARANCE_TOLERANCE
                && body_residual(&points, body, margin)
                    <= fabelgeist_cloth::outer_layer::CLEARANCE_TOLERANCE
                && super::validation::validate_surface(
                    &fabelgeist_bvh::TriangleBvh::new(points.clone(), surface.faces.clone()),
                    body,
                )
                .is_ok()
            {
                break;
            }
        }
        let mut issues = Vec::new();
        let residual = body_residual(&points, body, margin);
        if residual > fabelgeist_cloth::outer_layer::CLEARANCE_TOLERANCE {
            issues.push(format!(
                "insufficient space between wearer and armor for this garment: {:.2} mm body residual",
                residual * 1000.0
            ));
        }
        self.positions = surface.expand(&points.into_iter().map(array).collect::<Vec<_>>());
        self.normals = self.normals_for(&self.positions);
        if let Err(error) = self.validate_armor(armor) {
            issues.push(format!("{error:#}"));
        }
        Ok(issues)
    }
}

pub(super) fn body_residual(
    points: &[Vec3],
    body: &fabelgeist_bvh::TriangleBvh,
    margin: f32,
) -> f32 {
    points
        .iter()
        .filter_map(|&point| {
            let (index, closest, _) = body.closest_point(point, f32::MAX)?;
            let (a, b, c) = body.triangle(index);
            let raw = (b - a).cross(c - a);
            (raw.length() > 1e-10).then(|| margin - (point - closest).dot(raw / raw.length()))
        })
        .fold(0.0, f32::max)
}
