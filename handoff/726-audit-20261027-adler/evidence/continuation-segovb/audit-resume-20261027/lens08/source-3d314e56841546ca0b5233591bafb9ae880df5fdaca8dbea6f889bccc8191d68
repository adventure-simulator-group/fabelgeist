//! Narrow flights connect a level court to independently supported floors.
//! Step dimensions are supplied by the caller; no movement limit is relaxed.
use super::*;
use adventuresim_building_generator::AccessStairFlight;

/// Architectural stair bounds, distinct from the actor's maximum step height.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, bevy::reflect::Reflect)]
#[reflect(opaque)]
pub struct CourtStairLimits {
    pub(super) maximum_riser_metres:
        adventuresim_building_generator::spatial_geometry::PositiveLength,
    pub(super) minimum_going_metres:
        adventuresim_building_generator::spatial_geometry::PositiveLength,
    minimum_clear_width_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
    pub(super) minimum_floor_landing_run_metres:
        adventuresim_building_generator::spatial_geometry::PositiveLength,
    minimum_court_landing_run_metres:
        adventuresim_building_generator::spatial_geometry::PositiveLength,
}

impl CourtStairLimits {
    pub const fn new(
        maximum_riser_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
        minimum_going_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
        minimum_clear_width_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
        minimum_floor_landing_run_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
        minimum_court_landing_run_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
    ) -> Self {
        Self {
            maximum_riser_metres,
            minimum_going_metres,
            minimum_clear_width_metres,
            minimum_floor_landing_run_metres,
            minimum_court_landing_run_metres,
        }
    }

    /// Run remaining after reserving complete landings at both bound endpoints.
    pub fn available_run_metres(self, route: &CityAccessSegment) -> Option<f32> {
        let floor_landing = self
            .minimum_floor_landing_run_metres
            .metres()
            .max(route.half_width_metres());
        let court_landing = self
            .minimum_court_landing_run_metres
            .metres()
            .max(route.half_width_metres());
        let run = route.start_metres().distance(route.end_metres()) - floor_landing - court_landing;
        (run.is_finite() && run > 0.0).then_some(run)
    }

    pub(super) fn maximum_rise_metres(
        self,
        route: &CityAccessSegment,
        limits: SupportLimits,
    ) -> f32 {
        let Some(run) = self.available_run_metres(route) else {
            return 0.0;
        };
        let risers = (run / self.minimum_going_metres.metres())
            .floor()
            .min(f32::from(u16::MAX));
        (run * limits.maximum_grade.ratio()).min(risers * self.maximum_riser_metres.metres())
            - limits.contact_tolerance_metres.metres()
    }
}

#[derive(Clone, Debug)]
pub(super) struct CourtStair {
    pub flight: AccessStairFlight,
    pub half_width_metres: f32,
    pub bottom_local: Vec2,
    pub top_local: Vec2,
    floor_local: Vec2,
    floor_landing_half_length_metres: f32,
    floor_elevation_metres: f32,
}

impl CourtStair {
    pub(super) fn shift_floor(&mut self, metres: f32) {
        self.flight.top_elevation_metres += metres;
        self.flight.bottom_elevation_metres += metres;
        self.floor_elevation_metres += metres;
    }

    pub fn compile(
        property: &CityCompound,
        route: &CityAccessSegment,
        floor: SupportElevation,
        court: SupportElevation,
        bounds: CourtStairLimits,
        limits: SupportLimits,
    ) -> Result<Option<Self>, SupportDiagnostic> {
        let delta = route.end_metres() - route.start_metres();
        let direction = delta.normalize_or_zero();
        let floor_landing = bounds
            .minimum_floor_landing_run_metres
            .metres()
            .max(route.half_width_metres());
        let court_landing = bounds
            .minimum_court_landing_run_metres
            .metres()
            .max(route.half_width_metres());
        let width =
            (bounds.minimum_clear_width_metres.metres() * 0.5).max(route.half_width_metres());
        let court_end = route.start_metres() + direction * court_landing;
        let floor_end = route.end_metres() - direction * floor_landing;
        let run = delta.length() - floor_landing - court_landing;
        let rise = (floor.metres() - court.metres()).abs();
        if run <= 0.0
            || rise > run * limits.maximum_grade.ratio() + limits.contact_tolerance_metres.metres()
        {
            return Err(SupportDiagnostic::new(
                property,
                SupportConstraint::AccessGrade,
                SupportBoundary::CourtLanding,
                court_end,
                rise,
                run.max(0.0) * limits.maximum_grade.ratio(),
            ));
        }
        if rise <= limits.contact_tolerance_metres.metres() {
            return Ok(None);
        }
        validate_reservation(
            property,
            route,
            direction,
            width,
            floor_landing,
            court_landing,
        )?;
        let count = (rise / bounds.maximum_riser_metres.metres()).ceil();
        if count > f32::from(u16::MAX) {
            return Err(SupportDiagnostic::new(
                property,
                SupportConstraint::StairGoing,
                SupportBoundary::CourtLanding,
                court_end,
                count * bounds.minimum_going_metres.metres(),
                run,
            ));
        }
        let required_run = count * bounds.minimum_going_metres.metres();
        if required_run > run + limits.contact_tolerance_metres.metres() {
            return Err(SupportDiagnostic::new(
                property,
                SupportConstraint::StairGoing,
                SupportBoundary::CourtLanding,
                court_end,
                required_run,
                run,
            ));
        }
        let (top, bottom, top_height, bottom_height) = if floor.metres() > court.metres() {
            (floor_end, court_end, floor.metres(), court.metres())
        } else {
            (court_end, floor_end, court.metres(), floor.metres())
        };
        let local = |point| {
            property
                .plot
                .orientation()
                .world_to_local(point - property.plot.centre_metres())
        };
        Ok(Some(Self {
            flight: AccessStairFlight {
                top,
                bottom,
                top_elevation_metres: top_height,
                bottom_elevation_metres: bottom_height,
                riser_count: count as u16,
                going_metres: run / count,
                nosing_metres: 0.0,
            },
            half_width_metres: width,
            top_local: local(top),
            bottom_local: local(bottom),
            floor_local: local(route.end_metres()),
            floor_landing_half_length_metres: floor_landing,
            floor_elevation_metres: floor.metres(),
        }))
    }

    pub fn height_at(&self, local: Vec2) -> Option<f32> {
        if (local - self.floor_local)
            .abs()
            .cmple(Vec2::new(
                self.half_width_metres,
                self.floor_landing_half_length_metres,
            ))
            .all()
        {
            return Some(self.floor_elevation_metres);
        }
        let axis = self.top_local - self.bottom_local;
        let length = axis.length();
        let distance = (local - self.bottom_local).dot(axis / length);
        let across = (local - self.bottom_local).perp_dot(axis / length).abs();
        if !(0.0..=length).contains(&distance) || across > self.half_width_metres {
            return None;
        }
        let count = f32::from(self.flight.riser_count);
        let tread = (distance / length * count).ceil().clamp(0.0, count);
        Some(
            self.flight.bottom_elevation_metres
                + (self.flight.top_elevation_metres - self.flight.bottom_elevation_metres) * tread
                    / count,
        )
    }

    pub fn local_cuts(&self) -> (Vec<f32>, Vec<f32>) {
        let width = self.half_width_metres;
        let x = self.top_local.x;
        let count = self.flight.riser_count;
        let mut z: Vec<_> = (0..=count)
            .map(|i| {
                self.bottom_local.y
                    + (self.top_local.y - self.bottom_local.y) * f32::from(i) / f32::from(count)
            })
            .collect();
        z.extend([
            self.floor_local.y - self.floor_landing_half_length_metres,
            self.floor_local.y + self.floor_landing_half_length_metres,
        ]);
        (vec![x - width, x + width], z)
    }
}

fn validate_reservation(
    property: &CityCompound,
    route: &CityAccessSegment,
    direction: Vec2,
    half_width: f32,
    floor_landing: f32,
    court_landing: f32,
) -> Result<(), SupportDiagnostic> {
    let across = Vec2::new(-direction.y, direction.x);
    for (anchor, landing) in [
        (route.start_metres(), court_landing),
        (route.end_metres(), floor_landing),
    ] {
        for along in [-landing, landing] {
            for side in [-half_width, half_width] {
                let point = anchor + direction * along + across * side;
                let local = property
                    .plot
                    .orientation()
                    .world_to_local(point - property.plot.centre_metres());
                let outside =
                    (local.abs() - property.plot.dimensions_metres() * 0.5).max(Vec2::ZERO);
                if outside.length_squared() > 0.0 {
                    return Err(SupportDiagnostic::new(
                        property,
                        SupportConstraint::StairClearance,
                        SupportBoundary::PropertyReservation,
                        point,
                        outside.length(),
                        0.0,
                    ));
                }
            }
        }
    }
    Ok(())
}
