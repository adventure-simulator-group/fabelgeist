//! Check the exact rounded geometry with the authoritative overlap calculation.
use super::*;

pub(super) fn validate(
    domains: &[PlacementDomain],
    positions: &FrontageCoordinates,
) -> Result<(), CoupledPackingIssue> {
    let mut geometry = Vec::with_capacity(domains.len());
    for (domain, coordinate) in domains.iter().zip(positions.iter()) {
        let position = coordinate.metres();
        let allowed = domain.allowed;
        if !position.is_finite()
            || position < allowed.minimum_metres
            || position > allowed.maximum_metres
        {
            return Err(CoupledPackingIssue::OutsideDomain {
                property: domain.owner,
                displacement_metres: position,
                permitted: allowed,
            });
        }
        let delta = domain.delta_at(coordinate);
        let shape = domain
            .proposed
            .translated(delta.metres(), domain.frontage.tangent());
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
