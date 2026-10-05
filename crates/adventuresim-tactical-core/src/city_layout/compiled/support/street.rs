//! Prefer the property's near street half within the declared maximum apron.
use super::*;
use crate::city_layout::grounding::access::{access_regions, available_run};

pub(super) fn select(
    property: &CityCompound,
    observations: CompoundSupportLevels,
    threshold: Vec2,
    geographic: &GeographicSurface,
    streets: &[CityStreetPatch],
    policy: CompoundGradingPolicy,
) -> Result<CompoundSupportPlan, CitySupportError> {
    let plot = property.plot;
    let local = plot
        .orientation
        .world_to_local(threshold - plot.centre_metres);
    let edge = plot.centre_metres
        + plot
            .orientation
            .local_to_world(Vec2::new(local.x, -plot.dimensions_metres.y * 0.5));
    let direction = plot.orientation.local_to_world(-Vec2::Y);
    let offset = plot.orientation.local_to_world(Vec2::X) * policy.street_apron.0.x * 0.5;
    let maximum = policy.street_apron.0.y;
    let regions = access_regions(plot, streets, edge);
    let preferred = (available_run(
        edge,
        direction,
        offset,
        &regions,
        maximum,
        policy.limits.contact_tolerance_metres(),
    ) - policy.limits.contact_tolerance_metres())
    .max(0.0);
    let candidate = |run: f32| {
        CompoundSupportRequest {
            property,
            observations,
            street_threshold_metres: threshold,
            street_apron: CityPlotBounds {
                centre_metres: edge + direction * run * 0.5,
                dimensions_metres: Vec2::new(policy.street_apron.0.x, run),
                orientation: plot.orientation,
            },
            geographic,
            limits: policy.limits,
            stairs: policy.stairs,
            embedment: policy.embedment,
        }
        .select()
    };
    // A steep approach may require more of the existing external reservation.
    // Both candidates retain the same members and complete source constraints;
    // final composition still rejects any intersection with another property.
    if preferred > 0.0 && preferred < maximum {
        match candidate(preferred) {
            Ok(plan) => return Ok(plan),
            Err(error)
                if matches!(
                    error.constraint,
                    SupportConstraint::StairGoing
                        | SupportConstraint::StairClearance
                        | SupportConstraint::AccessGrade
                        | SupportConstraint::CutFill
                ) => {}
            Err(error) => return Err(CitySupportError::Support(error)),
        }
    }
    candidate(maximum).map_err(CitySupportError::Support)
}
