//! Reclaim unused legal frontage setback before declaring a block unresolved.
use super::*;

pub(super) fn seat(domains: &[PlacementDomain]) -> Result<Vec<PlacementDomain>, CityCompileError> {
    domains
        .iter()
        .map(|domain| {
            let mut seated = domain.clone();
            let frontage = domain.frontage;
            let normal = frontage.tangent().perp();
            let start = frontage.block.corners[frontage.edge].as_dvec2();
            let clearance = domain
                .proposed
                .reservation
                .corners()
                .into_iter()
                .map(|point| {
                    normal.as_dvec2().dot(point.as_dvec2() - start)
                        - f64::from(frontage.block.streets[frontage.edge].half_width())
                        + f64::from(plots::STREET_EDGE_TOLERANCE_METRES)
                })
                .fold(f64::INFINITY, f64::min);
            let shift = (clearance - CityPlotBounds::COORDINATE_TOLERANCE_METRES).max(0.0);
            seated.base_translation = PlanDisplacement::from_metres(-normal * shift as f32).ok_or_else(||domain.packing_error(domain.geometry_error(adventuresim_building_generator::plan_geometry::PlanGeometryError::NonFinite)))?;
            let reservation = seated.geometry_at_zero().map_err(|issue|domain.packing_error(issue))?.reservation;
            seated.allowed = frontage
                .available_displacement(reservation)
                .ok_or_else(|| CityCompileError::Packing {
                    property: domain.owner,
                    issue: CityPackingIssue::NoFreeFrontage {
                        block: frontage.block.id.0,
                        envelope: reservation,
                        available_displacement_metres: None,
                        blocking_properties: domains.iter().map(|d| d.owner).collect(),
                    },
                })?;
            Ok(seated)
        })
        .collect()
}
