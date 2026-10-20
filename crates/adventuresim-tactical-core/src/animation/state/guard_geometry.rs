//! Leg-scaled guard stance and contact travel in rig-local metres.

use super::*;
use adventuresim_building_generator::spatial_geometry::SignedLength;

// Measured hip-knee-ankle chain of the current humanoid rig. The animation
// client uses its live rig measurement; this is the server-side fallback until
// anatomical dimensions become part of character state.
/// Ground distance covered by one procedural combat-stance contact interval.
pub fn guard_step_length() -> AnimationValueResult<NonnegativeLength> {
    guard_contact_travel_distance(
        guard_footwork_config().reference_leg_length_metres.into(),
        Vec2::NEG_Y,
    )
}

/// Maximum lateral open stance. Like the longitudinal opening, this is the
/// leg-length-scaled one-yard reference stance: the moving-side foot reaches
/// outward while the support foot remains behind the projected COM.
pub fn guard_maximum_lateral_foot_separation(
    leg_length: SignedLength,
) -> AnimationValueResult<NonnegativeLength> {
    guard_maximum_foot_separation(leg_length)
}

/// Direction-specific open stance, blended continuously for diagonals.
pub fn guard_open_foot_separation(
    leg_length: SignedLength,
    local_direction: Vec2,
) -> AnimationValueResult<NonnegativeLength> {
    let direction = local_direction.normalize_or_zero();
    let longitudinal = direction.y.abs();
    NonnegativeLength::from_arithmetic(
        guard_maximum_lateral_foot_separation(leg_length)?
            .metres()
            .lerp(
                guard_maximum_foot_separation(leg_length)?.metres(),
                longitudinal,
            ),
    )
}

/// Direction-specific closed stance. Lateral shuffles retain normal
/// anatomical width; the three-inch contract applies along the guard's
/// longitudinal axis.
pub fn guard_closed_foot_separation(
    leg_length: SignedLength,
    local_direction: Vec2,
) -> AnimationValueResult<NonnegativeLength> {
    let direction = local_direction.normalize_or_zero();
    let longitudinal = direction.y.abs();
    NonnegativeLength::from_arithmetic((leg_length.metres().max(0.0) * 0.25).lerp(
        guard_rear_contact_separation(leg_length)?.metres(),
        longitudinal,
    ))
}

/// COM travel between centered open and closed contacts.
pub fn guard_contact_travel_distance(
    leg_length: SignedLength,
    local_direction: Vec2,
) -> AnimationValueResult<NonnegativeLength> {
    NonnegativeLength::from_arithmetic(
        (guard_open_foot_separation(leg_length, local_direction)?.metres()
            - guard_closed_foot_separation(leg_length, local_direction)?.metres())
        .max(0.0)
            * 0.5,
    )
}

/// Maximum fore-aft guard stance immediately before the following foot lifts.
/// The ratio maps a 0.851688 m average-male thigh-plus-shank chain at 5'11" to
/// the requested one-yard stance, then scales directly with the actual rig.
pub fn guard_maximum_foot_separation(
    leg_length: SignedLength,
) -> AnimationValueResult<NonnegativeLength> {
    NonnegativeLength::from_arithmetic(leg_length.metres().max(0.0) * 1.073_632_5)
}

/// Maximum fore-aft separation when the rear foot returns beside the front
/// foot. This maps the same 5'11" reference leg to the requested three inches.
pub fn guard_rear_contact_separation(
    leg_length: SignedLength,
) -> AnimationValueResult<NonnegativeLength> {
    NonnegativeLength::from_arithmetic(leg_length.metres().max(0.0) * 0.089_469_37)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guard_stride_scales_with_leg_length_from_the_five_eleven_reference() {
        let reference_leg_length = 1.8034 * 0.860 / 1.821;
        assert!(
            guard_maximum_foot_separation(SignedLength::from_metres(0.75).unwrap())
                .unwrap()
                .metres()
                < guard_maximum_foot_separation(SignedLength::from_metres(0.90).unwrap())
                    .unwrap()
                    .metres()
        );
        assert!(
            (guard_maximum_foot_separation(
                SignedLength::from_metres(reference_leg_length).unwrap()
            )
            .unwrap()
            .metres()
                - 0.9144)
                .abs()
                < 0.0001
        );
        assert!(
            (guard_rear_contact_separation(
                SignedLength::from_metres(reference_leg_length).unwrap()
            )
            .unwrap()
            .metres()
                - 0.0762)
                .abs()
                < 0.0001
        );
    }
}
