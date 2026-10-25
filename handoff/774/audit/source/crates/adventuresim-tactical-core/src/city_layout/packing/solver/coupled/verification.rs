//! Check the exact rounded geometry with the authoritative overlap calculation.
use super::*;

pub(super) fn validate(
    domains: &[PlacementDomain],
    positions: &FrontageDisplacements,
) -> Result<(), CoupledPackingIssue> {
    let mut geometry = Vec::with_capacity(domains.len());
    for (domain, coordinate) in domains.iter().zip(positions.iter()) {
        let frontage_displacement = coordinate.metres();
        let allowed = domain.allowed;
        if !frontage_displacement.is_finite()
            || frontage_displacement < allowed.minimum_metres
            || frontage_displacement > allowed.maximum_metres
        {
            return Err(CoupledPackingIssue::OutsideDomain {
                property: domain.owner,
                displacement_metres: frontage_displacement,
                permitted: allowed,
            });
        }
        let delta = domain.delta_at(coordinate)?;
        let shape = domain
            .proposed
            .translated(delta, domain.frontage.tangent())
            .map_err(|issue| domain.geometry_error(issue))?;
        if let Some(garden) = &shape.garden {
            garden
                .clearance_geometry()
                .map_err(|issue| CoupledPackingIssue::Garden {
                    property: domain.owner,
                    issue,
                })?;
        }
        geometry.push(shape);
    }
    for first in 0..geometry.len() {
        for second in first + 1..geometry.len() {
            if !geometry[first].clears(&geometry[second]) {
                return Err(CoupledPackingIssue::Overlap {
                    first: domains[first].owner,
                    second: domains[second].owner,
                });
            }
        }
    }
    Ok(())
}
