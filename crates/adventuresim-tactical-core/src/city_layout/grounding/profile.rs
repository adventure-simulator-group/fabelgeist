use super::*;

/// Finite signed ordinate along a bound profile, in metres. Court ordinates
/// run on both sides of the property origin; this is not a positive length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProfileCoordinate(f32);
impl ProfileCoordinate {
    pub fn from_metres(metres: f32) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProfilePoint {
    pub coordinate: ProfileCoordinate,
    pub elevation: SupportElevation,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SupportProfile {
    pub points: Vec<ProfilePoint>,
}

impl SupportProfile {
    pub(super) fn checked(
        property: &CityCompound,
        points: Vec<ProfilePoint>,
        limits: SupportLimits,
        boundary: SupportBoundary,
        location_at: impl Fn(ProfileCoordinate) -> Vec2,
    ) -> Result<Self, SupportDiagnostic> {
        for pair in points.windows(2) {
            let run = pair[1].coordinate.metres() - pair[0].coordinate.metres();
            let rise = (pair[1].elevation.metres() - pair[0].elevation.metres()).abs();
            let permitted = run.max(0.0) * limits.maximum_grade;
            if run <= 0.0 || rise > permitted + limits.contact_tolerance_metres {
                return Err(SupportDiagnostic::new(
                    property,
                    SupportConstraint::AccessGrade,
                    boundary,
                    location_at(pair[0].coordinate),
                    rise,
                    permitted,
                ));
            }
        }
        Ok(Self { points })
    }

    pub(super) fn height_at(&self, coordinate: ProfileCoordinate) -> SupportElevation {
        let distance = coordinate.metres();
        let first = self.points[0];
        if distance <= first.coordinate.metres() {
            return first.elevation;
        }
        for pair in self.points.windows(2) {
            if distance <= pair[1].coordinate.metres() {
                let fraction = (distance - pair[0].coordinate.metres())
                    / (pair[1].coordinate.metres() - pair[0].coordinate.metres());
                return SupportElevation(
                    pair[0].elevation.metres()
                        + fraction * (pair[1].elevation.metres() - pair[0].elevation.metres()),
                );
            }
        }
        self.points
            .last()
            .expect("validated nonempty profile")
            .elevation
    }
}

impl ProfilePoint {
    pub fn at_metres(coordinate: f32, elevation: SupportElevation) -> Self {
        Self {
            coordinate: ProfileCoordinate::from_metres(coordinate)
                .expect("validated property profile has finite coordinates"),
            elevation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_profile_interpolates_elevations_and_clamps_endpoints() {
        assert!(ProfileCoordinate::from_metres(f32::INFINITY).is_none());
        let profile = SupportProfile {
            points: vec![
                ProfilePoint::at_metres(-2.0, SupportElevation(10.0)),
                ProfilePoint::at_metres(2.0, SupportElevation(12.0)),
            ],
        };
        let at = |x| {
            profile
                .height_at(ProfileCoordinate::from_metres(x).unwrap())
                .metres()
        };
        assert_eq!(at(-3.0), 10.0);
        assert_eq!(at(0.0), 11.0);
        assert_eq!(at(3.0), 12.0);
    }
}
