//! Intersect complete bearing and exact doorway constraints before selecting a floor.
use super::*;
use crate::scene_coordinates::ScenePlanPoint;

pub(super) struct SelectedFloor {
    pub threshold: ScenePlanPoint,
    pub elevation: SupportElevation,
}

pub(super) fn select(
    request: &SingleBuildingSupportRequest<'_>,
    doors: &[&DoorwaySupportBinding],
    aprons: &[apron::PreparedApron],
    owner: &PropertySupportSurface,
) -> Result<SelectedFloor, SupportDiagnostic> {
    if doors.is_empty() {
        return Err(owner.rejection(
            SupportConstraint::ThresholdBinding,
            SupportBoundary::FrontBearing,
            request.bearing.centre_metres,
            1.0,
            0.0,
        ));
    }
    let controls = request
        .geographic
        .height_range_in_outline(
            &request
                .bearing_outline
                .vertices()
                .iter()
                .map(|point| point.metres())
                .collect::<Vec<_>>(),
        )
        .ok_or_else(|| {
            owner.rejection(
                SupportConstraint::SourceSample,
                SupportBoundary::GeographicSurface,
                request.bearing.centre_metres,
                1.0,
                0.0,
            )
        })?;
    let permitted = request.policy.limits.maximum_displacement_metres;
    let mut lower = controls.maximum.elevation.metres() - permitted;
    let mut upper = controls.minimum.elevation.metres() + permitted;
    if lower > upper {
        return Err(owner.rejection(
            SupportConstraint::CutFill,
            SupportBoundary::GeographicSurface,
            controls.maximum.point.metres(),
            (controls.maximum.elevation.metres() - controls.minimum.elevation.metres()) * 0.5,
            permitted,
        ));
    }
    for (door, apron) in doors.iter().zip(aprons) {
        let constraints = apron.floor_interval(request.policy);
        let constrained_lower = lower.max(constraints.minimum.metres());
        let constrained_upper = upper.min(constraints.maximum.metres());
        if constrained_lower > constrained_upper {
            return Err(owner.rejection(
                SupportConstraint::AccessGrade,
                SupportBoundary::StreetLanding,
                door.threshold_metres,
                constrained_lower - constrained_upper,
                0.0,
            ));
        }
        lower = constrained_lower;
        upper = constrained_upper;
    }
    let observations = doors
        .iter()
        .map(|entry| {
            request
                .geographic
                .elevation_at(entry.threshold_metres)
                .ok_or_else(|| {
                    owner.rejection(
                        SupportConstraint::SourceSample,
                        SupportBoundary::GeographicSurface,
                        entry.threshold_metres,
                        1.0,
                        0.0,
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let preferred = observations
        .iter()
        .map(|h| f64::from(h.metres()))
        .sum::<f64>()
        / observations.len() as f64;
    let tolerance = request.policy.limits.contact_tolerance_metres;
    let elevation = if lower + tolerance <= upper - tolerance {
        (preferred as f32).clamp(lower + tolerance, upper - tolerance)
    } else {
        (lower + upper) * 0.5
    };
    SelectedFloor::bind(*doors[0], elevation, owner)
}

impl SelectedFloor {
    fn bind(
        door: DoorwaySupportBinding,
        elevation_metres: f32,
        owner: &PropertySupportSurface,
    ) -> Result<Self, SupportDiagnostic> {
        Ok(SelectedFloor {
            threshold: ScenePlanPoint::from_metres(door.threshold_metres).ok_or_else(|| {
                owner.rejection(
                    SupportConstraint::ThresholdBinding,
                    SupportBoundary::FrontBearing,
                    door.threshold_metres,
                    1.0,
                    0.0,
                )
            })?,
            elevation: SupportElevation::from_metres(elevation_metres).ok_or_else(|| {
                owner.rejection(
                    SupportConstraint::CutFill,
                    SupportBoundary::FrontBearing,
                    door.threshold_metres,
                    1.0,
                    0.0,
                )
            })?,
        })
    }
}
