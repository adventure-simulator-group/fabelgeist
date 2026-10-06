//! Mesh index bounds fail before geometry is appended.
use super::*;
impl PropertySupportMesh {
    pub(super) fn vertex_start(&self, added: usize, point: Vec2) -> Result<u32, SupportDiagnostic> {
        let total = self.positions.len().checked_add(added);
        let within = total.and_then(|count| u32::try_from(count).ok());
        let reject = |actual: Option<usize>| {
            let mut error = SupportDiagnostic::for_mesh(
                self,
                SupportConstraint::MeshIndexCapacity,
                point,
                0.0,
                0.0,
            );
            error.violation = match actual.and_then(|n| u64::try_from(n).ok()) {
                Some(actual) => SupportViolation::Count(BoundViolation::Maximum {
                    actual: DiagnosticCount::new(actual),
                    permitted: DiagnosticCount::new(u64::from(u32::MAX)),
                }),
                None => SupportViolation::Unmeasurable {
                    unit: SupportDiagnosticUnit::Count,
                    bound: SupportBound::Maximum,
                },
            };
            error
        };
        if within.is_none() {
            return Err(reject(total));
        }
        u32::try_from(self.positions.len()).map_err(|_| reject(Some(self.positions.len())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oversized_mesh_and_degenerate_doorway_report_exact_owners_without_appending() {
        let fixture = super::super::super::tests::Fixture::load();
        let plan = fixture.selected_plan(&fixture.source());
        let mut mesh = PropertySupportMesh::empty_for_compound(&plan);
        let original_positions = vec![Vec3::ZERO; 4];
        mesh.positions = original_positions.clone();
        let point = plan.property.plot.centre_metres();
        let error = mesh.vertex_start(usize::MAX, point).unwrap_err();
        assert_eq!(error.constraint, SupportConstraint::MeshIndexCapacity);
        assert_eq!(error.property_id, plan.property_id());
        assert_eq!(
            error.member_building_ids,
            vec![
                plan.property.front_building_id,
                plan.property.rear_building_id
            ]
        );
        assert_eq!(error.location_metres.attempted_metres(), point);
        assert!(error.violation.discrepancy_value() > 0.0);
        assert_eq!(mesh.positions, original_positions);
        let mut invalid = mesh.clone();
        invalid.positions = vec![Vec3::ZERO; 3];
        invalid.support_triangles = vec![[0, 1, 2]];
        let error = mesh.append_outside_floor(&invalid, &[]).unwrap_err();
        assert_eq!(error.constraint, SupportConstraint::Reservation);
        assert_eq!(error.property_id, mesh.property_id);
        assert_eq!(mesh.positions, original_positions);
        assert!(mesh.support_triangles.is_empty());
    }
}
