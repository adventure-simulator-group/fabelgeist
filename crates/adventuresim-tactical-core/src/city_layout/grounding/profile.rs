use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProfilePoint {
    pub distance_metres: f32,
    pub height_metres: f32,
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
        location_at: impl Fn(f32) -> Vec2,
    ) -> Result<Self, SupportDiagnostic> {
        for pair in points.windows(2) {
            let run = pair[1].distance_metres - pair[0].distance_metres;
            let rise = (pair[1].height_metres - pair[0].height_metres).abs();
            let permitted = run.max(0.0) * limits.maximum_grade;
            if run <= 0.0 || rise > permitted + limits.contact_tolerance_metres {
                return Err(SupportDiagnostic::new(
                    property,
                    SupportConstraint::AccessGrade,
                    boundary,
                    location_at(pair[0].distance_metres),
                    rise,
                    permitted,
                ));
            }
        }
        Ok(Self { points })
    }

    pub(super) fn height_at(&self, distance: f32) -> f32 {
        let first = self.points[0];
        if distance <= first.distance_metres {
            return first.height_metres;
        }
        for pair in self.points.windows(2) {
            if distance <= pair[1].distance_metres {
                let fraction = (distance - pair[0].distance_metres)
                    / (pair[1].distance_metres - pair[0].distance_metres);
                return pair[0].height_metres
                    + fraction * (pair[1].height_metres - pair[0].height_metres);
            }
        }
        self.points
            .last()
            .expect("validated nonempty profile")
            .height_metres
    }
}
