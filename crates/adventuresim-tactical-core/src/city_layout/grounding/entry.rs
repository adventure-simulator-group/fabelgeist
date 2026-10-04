//! A street approach preserves source ground and a complete doorway landing.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct StreetEntryApron {
    pub reservation: CityPlotBounds,
    pub support_region: CityPlotBounds,
    pub mesh: PropertySupportMesh,
}

impl CompoundSupportPlan {
    /// Bind the exact street door to a supplied external apron. Support extends
    /// through the owned setback, stopping at the bearing edge. The doorway
    /// landing is level before the actor reaches the wall. Compare a ramp with
    /// a flight meeting the supplied riser/going bounds; never extend the
    /// external reservation or alter actor movement limits.
    pub fn bind_street_entry(
        mut self,
        threshold: Vec2,
        reservation: CityPlotBounds,
        source: &GeographicSurface,
        stairs: CourtStairLimits,
    ) -> Result<Self, SupportDiagnostic> {
        let region = self.street_entry_region(threshold, reservation)?;
        let corners = region.corners();
        let mut points = corners.map(|p| Vec3::new(p.x, self.levels.front.elevation.metres(), p.y));
        for i in 0..2 {
            points[i].y = source
                .elevation_at(corners[i])
                .ok_or_else(|| {
                    self.street_rejection(SupportConstraint::SourceSample, corners[i], 1.0, 0.0)
                })?
                .metres();
        }
        let threshold_local = region
            .orientation
            .world_to_local(threshold - region.centre_metres);
        let landing_begin = threshold_local.y - stairs.minimum_floor_landing_run_metres;
        let run = landing_begin + region.dimensions_metres.y * 0.5;
        if run <= 0.0 {
            return Err(self.street_rejection(
                SupportConstraint::StairClearance,
                threshold,
                stairs.minimum_floor_landing_run_metres,
                threshold_local.y + region.dimensions_metres.y * 0.5,
            ));
        }
        let landing_fraction = run / region.dimensions_metres.y;
        let floor = self.levels.front.elevation.metres();
        let edge = |t: f32, heights: [f32; 2]| {
            let a = points[0].lerp(points[3], t).with_y(heights[0]);
            let b = points[1].lerp(points[2], t).with_y(heights[1]);
            [a, b]
        };
        let end = edge(landing_fraction, [floor; 2]);
        let mut mesh = mesh::empty(&self);
        mesh.quad([points[0], points[1], end[1], end[0]], false);
        if mesh.maximum_grade() > self.limits.maximum_grade {
            mesh = mesh::empty(&self);
            let rise = [points[0].y, points[1].y]
                .map(|h| (h - floor).abs())
                .into_iter()
                .fold(0.0, f32::max);
            let count = (rise / stairs.maximum_riser_metres).ceil();
            let required = count * stairs.minimum_going_metres;
            if count > f32::from(u16::MAX) || required > run + self.limits.contact_tolerance_metres
            {
                return Err(self.street_rejection(
                    SupportConstraint::StairGoing,
                    reservation.centre_metres,
                    required,
                    run,
                ));
            }
            let heights = |t: f32| [points[0].y, points[1].y].map(|h| h + (floor - h) * t);
            for i in 0..count as u16 {
                let t = f32::from(i) / count;
                let next = f32::from(i + 1) / count;
                let a = edge(landing_fraction * t, heights(t));
                let b = edge(landing_fraction * next, heights(t));
                let c = edge(landing_fraction * next, heights(next));
                mesh.quad([a[0], a[1], b[1], b[0]], false);
                mesh.quad([b[0], b[1], c[1], c[0]], true);
            }
        }
        mesh.quad([end[0], end[1], points[2], points[3]], false);
        if mesh.maximum_grade() > self.limits.maximum_grade {
            return Err(self.street_rejection(
                SupportConstraint::AccessGrade,
                reservation.centre_metres,
                mesh.maximum_grade() * run,
                self.limits.maximum_grade * run,
            ));
        }
        self.street_entry = Some(StreetEntryApron {
            reservation,
            support_region: region,
            mesh,
        });
        Ok(self)
    }

    fn street_entry_region(
        &self,
        threshold: Vec2,
        reservation: CityPlotBounds,
    ) -> Result<CityPlotBounds, SupportDiagnostic> {
        let local = |point| {
            self.property
                .plot
                .orientation
                .world_to_local(point - self.property.plot.centre_metres)
        };
        let centre = local(reservation.centre_metres);
        let half = reservation.dimensions_metres * 0.5;
        let plot_half = self.property.plot.dimensions_metres * 0.5;
        let threshold_local = local(threshold);
        if self.street_entry.is_some()
            || !reservation.is_valid()
            || reservation.orientation != self.property.plot.orientation
            || !self.property.plot.contains(threshold)
            || centre.x.abs() + half.x > plot_half.x
            || (threshold_local.x - centre.x).abs() > self.limits.contact_tolerance_metres
            || (centre.y + half.y + plot_half.y).abs() > self.limits.contact_tolerance_metres
        {
            return Err(self.street_rejection(
                SupportConstraint::ThresholdBinding,
                threshold,
                1.0,
                0.0,
            ));
        }
        let bearing_begin = self
            .levels
            .front
            .contact
            .corners()
            .into_iter()
            .map(|point| local(point).y)
            .fold(f32::INFINITY, f32::min);
        let setback = bearing_begin + plot_half.y;
        if setback < -self.limits.contact_tolerance_metres || threshold_local.y < bearing_begin {
            return Err(self.street_rejection(SupportConstraint::Bearing, threshold, 1.0, 0.0));
        }
        Ok(CityPlotBounds {
            centre_metres: reservation.centre_metres
                + reservation.orientation.local_to_world(Vec2::Y) * setback.max(0.0) * 0.5,
            dimensions_metres: reservation.dimensions_metres + Vec2::Y * setback.max(0.0),
            orientation: reservation.orientation,
        })
    }

    fn street_rejection(
        &self,
        constraint: SupportConstraint,
        point: Vec2,
        measured: f32,
        permitted: f32,
    ) -> SupportDiagnostic {
        let mut error = SupportDiagnostic::new(
            &self.property,
            constraint,
            SupportBoundary::StreetLanding,
            point,
            measured,
            permitted,
        );
        error.attempted_treatment = SupportGradingAttempt::Compound(self.treatment);
        error
    }
}
