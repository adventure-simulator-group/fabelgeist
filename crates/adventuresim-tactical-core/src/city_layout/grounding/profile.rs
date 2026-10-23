//! Ordered support sequences distinguish walkable grades from retaining steps.
use super::*;

/// Finite signed ordinate along a bound profile, in metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProfileCoordinate(f32);
impl ProfileCoordinate {
    pub const ZERO: Self = Self(0.0);
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
impl ProfilePoint {
    pub fn at_metres(coordinate: f32, elevation: SupportElevation) -> Option<Self> {
        elevation.metres().is_finite().then_some(Self {
            coordinate: ProfileCoordinate::from_metres(coordinate)?,
            elevation,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ProfileError {
    Empty,
    InvalidElevation { coordinate: ProfileCoordinate },
    Reversed { coordinate: ProfileCoordinate },
}
impl ProfileError {
    pub fn coordinate(self) -> ProfileCoordinate {
        match self {
            Self::Empty => ProfileCoordinate::ZERO,
            Self::InvalidElevation { coordinate } | Self::Reversed { coordinate } => coordinate,
        }
    }
}

/// Nonempty finite ordered storage. Repeated ordinates represent vertical
/// retaining faces. Walkability is additionally checked at passage construction.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SupportProfile {
    first: ProfilePoint,
    remainder: Vec<ProfilePoint>,
}
#[derive(Clone, Copy)]
pub(super) enum FloorBasisRole {
    Court,
    PassageTerminalLanding,
}
impl SupportProfile {
    pub fn stepped(points: Vec<ProfilePoint>) -> Result<Self, ProfileError> {
        let mut points = points.into_iter();
        let first = points.next().ok_or(ProfileError::Empty)?;
        let remainder: Vec<_> = points.collect();
        let profile = Self { first, remainder };
        let mut previous = first.coordinate;
        for point in profile.points() {
            if !point.elevation.metres().is_finite() {
                return Err(ProfileError::InvalidElevation {
                    coordinate: point.coordinate,
                });
            }
            if point.coordinate.metres() < previous.metres() {
                return Err(ProfileError::Reversed {
                    coordinate: point.coordinate,
                });
            }
            previous = point.coordinate;
        }
        Ok(profile)
    }

    pub fn points(&self) -> impl DoubleEndedIterator<Item = &ProfilePoint> {
        std::iter::once(&self.first).chain(self.remainder.iter())
    }

    pub fn checked(
        property: &CityCompound,
        points: Vec<ProfilePoint>,
        limits: SupportLimits,
        boundary: SupportBoundary,
        location_at: impl Fn(ProfileCoordinate) -> Vec2,
    ) -> Result<Self, SupportDiagnostic> {
        // Preserve the passage's strict ordering/grade rejection separately
        // from the stepped court's nondecreasing-order storage invariant.
        for pair in points.windows(2) {
            let run = pair[1].coordinate.metres() - pair[0].coordinate.metres();
            let rise = (pair[1].elevation.metres() - pair[0].elevation.metres()).abs();
            let permitted = run.max(0.0) * limits.maximum_grade.ratio();
            if run <= 0.0 || rise > permitted + limits.contact_tolerance_metres.metres() {
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
        Self::stepped(points).map_err(|error| {
            SupportDiagnostic::new(
                property,
                SupportConstraint::Reservation,
                boundary,
                location_at(error.coordinate()),
                1.0,
                0.0,
            )
        })
    }

    pub fn height_at(&self, coordinate: ProfileCoordinate) -> SupportElevation {
        let distance = coordinate.metres();
        if distance <= self.first.coordinate.metres() {
            return self.first.elevation;
        }
        let mut previous = self.first;
        for next in &self.remainder {
            if distance <= next.coordinate.metres() {
                let run = next.coordinate.metres() - previous.coordinate.metres();
                let fraction = (distance - previous.coordinate.metres()) / run;
                let value = previous.elevation.metres()
                    + fraction * (next.elevation.metres() - previous.elevation.metres());
                if run.is_finite() && value.is_finite() {
                    return SupportElevation(value);
                }
                let fraction = (f64::from(distance) - f64::from(previous.coordinate.metres()))
                    / (f64::from(next.coordinate.metres())
                        - f64::from(previous.coordinate.metres()));
                let begin = f64::from(previous.elevation.metres());
                let end = f64::from(next.elevation.metres());
                return SupportElevation((begin * (1.0 - fraction) + end * fraction) as f32);
            }
            previous = *next;
        }
        previous.elevation
    }

    /// Affine-response probes preserve the sequence invariant. They do not
    /// certify a changed passage grade and cannot bypass final plan acceptance.
    pub fn translated_for_floor_basis(&self, role: FloorBasisRole) -> Result<Self, ProfileError> {
        let mut points: Vec<_> = self.points().copied().collect();
        let begin = match role {
            FloorBasisRole::Court => 0,
            FloorBasisRole::PassageTerminalLanding => points.len().saturating_sub(2),
        };
        for point in &mut points[begin..] {
            point.elevation = SupportElevation::from_metres(point.elevation.metres() + 1.0).ok_or(
                ProfileError::InvalidElevation {
                    coordinate: point.coordinate,
                },
            )?;
        }
        Self::stepped(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(coordinate: f32, elevation: f32) -> ProfilePoint {
        ProfilePoint::at_metres(coordinate, SupportElevation(elevation)).unwrap()
    }
    fn height(profile: &SupportProfile, coordinate: f32) -> f32 {
        profile
            .height_at(ProfileCoordinate::from_metres(coordinate).unwrap())
            .metres()
    }
    #[test]
    fn empty_invalid_and_reversed_profiles_reject_at_construction() {
        assert_eq!(SupportProfile::stepped(vec![]), Err(ProfileError::Empty));
        assert!(ProfilePoint::at_metres(f32::INFINITY, SupportElevation(0.0)).is_none());
        assert!(ProfilePoint::at_metres(0.0, SupportElevation(f32::NAN)).is_none());
        assert!(matches!(
            SupportProfile::stepped(vec![point(1.0, 0.0), point(0.0, 0.0)]),
            Err(ProfileError::Reversed { .. })
        ));
    }
    #[test]
    fn stepped_court_preserves_retaining_faces_and_endpoint_interpolation() {
        let profile = SupportProfile::stepped(vec![
            point(-2.0, 10.0),
            point(0.0, 10.0),
            point(0.0, 12.0),
            point(2.0, 12.0),
        ])
        .unwrap();
        assert_eq!(height(&profile, -3.0), 10.0);
        assert_eq!(height(&profile, 0.0), 10.0);
        assert_eq!(height(&profile, 0.25), 12.0);
        assert_eq!(height(&profile, 3.0), 12.0);
        let translated = profile
            .translated_for_floor_basis(FloorBasisRole::Court)
            .unwrap();
        assert_eq!(height(&translated, 0.25), 13.0);
        assert_eq!(
            profile.points().map(|p| p.coordinate).collect::<Vec<_>>(),
            translated
                .points()
                .map(|p| p.coordinate)
                .collect::<Vec<_>>()
        );
        let sloping = SupportProfile::stepped(vec![point(-2.0, 10.0), point(2.0, 12.0)]).unwrap();
        assert_eq!(height(&sloping, 0.0), 11.0);
    }
    #[test]
    fn passage_requires_strict_order_and_bounded_grade() {
        let fixture = super::super::tests::Fixture::load();
        let limits = SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.2).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        );
        let checked = |points| {
            SupportProfile::checked(
                &fixture.property,
                points,
                limits,
                SupportBoundary::GateLanding,
                |c| Vec2::Y * c.metres(),
            )
        };
        assert!(checked(vec![]).is_err());
        assert!(checked(vec![point(0.0, 0.0), point(0.0, 0.0)]).is_err());
        assert!(checked(vec![point(0.0, 0.0), point(1.0, 1.0)]).is_err());
        let profile = checked(vec![
            point(0.0, 0.0),
            point(1.0, 0.1),
            point(2.0, 0.2),
            point(3.0, 0.3),
        ])
        .unwrap();
        assert_eq!(height(&profile, 0.5), 0.05);
        let basis = profile
            .translated_for_floor_basis(FloorBasisRole::PassageTerminalLanding)
            .unwrap();
        assert_eq!(height(&basis, 0.0), 0.0);
        assert_eq!(height(&basis, 3.0), 1.3);
    }
    #[test]
    fn interpolation_of_finite_extreme_endpoints_stays_finite() {
        let profile =
            SupportProfile::stepped(vec![point(-2.0, -f32::MAX), point(2.0, f32::MAX)]).unwrap();
        assert_eq!(height(&profile, 0.0), 0.0);
        let wide =
            SupportProfile::stepped(vec![point(-f32::MAX, -1.0), point(f32::MAX, 1.0)]).unwrap();
        assert_eq!(height(&wide, 0.0), 0.0);
    }
}
