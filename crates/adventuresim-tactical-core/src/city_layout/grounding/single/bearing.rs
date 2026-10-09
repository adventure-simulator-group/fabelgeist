//! Validate complete floor containment before creating the support owner.
use super::*;
impl PropertySupportSurface {
    pub(super) fn for_single_bearing(
        request: &SingleBuildingSupportRequest<'_>,
    ) -> Result<Self, SupportDiagnostic> {
        let surface = PropertySupportSurface {
            floor_bearings: Vec::new(),
            mesh: PropertySupportMesh {
                property_id: request.property.id,
                member_building_ids: vec![request.property.building_id],
                positions: Vec::new(),
                support_triangles: Vec::new(),
                retaining_triangles: Vec::new(),
                contact_tolerance_metres: request.policy.limits.contact_tolerance_metres.metres(),
            },
            regions: vec![request.bearing],
            clipping_outlines: vec![
                request
                    .bearing_region
                    .outline()
                    .vertices()
                    .iter()
                    .map(|p| p.metres().as_dvec2())
                    .collect(),
            ],
            limits: request.policy.limits,
            treatment: SupportGradingAttempt::SingleBuildingFloorAndEntrances,
        };
        if !request.bearing.is_valid()
            || !request.property.plot.is_valid()
            || planar::signed_area(&surface.clipping_outlines[0]) <= f64::EPSILON
        {
            return Err(surface.rejection(
                SupportConstraint::Reservation,
                SupportBoundary::PropertyReservation,
                request.bearing.centre_metres(),
                1.0,
                0.0,
            ));
        }
        if let Some(point) = request
            .bearing_region
            .outline()
            .vertices()
            .iter()
            .find(|point| !request.property.plot.contains(point.metres()))
        {
            let local = request
                .property
                .plot
                .orientation()
                .world_to_local(point.metres() - request.property.plot.centre_metres())
                .abs();
            let excess = (local - request.property.plot.dimensions_metres() * 0.5).max_element();
            return Err(surface.rejection(
                SupportConstraint::Bearing,
                SupportBoundary::PropertyReservation,
                point.metres(),
                excess,
                CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32,
            ));
        }
        Ok(surface)
    }
}
