//! Compare a common level courtyard with a bounded stepped courtyard.
use super::*;

/// Exact bindings, geographic observations and declared engineering bounds.
/// Selection starts at the street observation and seats floors within complete
/// source constraints, retaining every horizontal region and declared bound.
/// It neither moves a property nor increases its supplied grading reservation.
pub struct CompoundSupportRequest<'a> {
    pub property: &'a CityCompound,
    pub observations: CompoundSupportLevels,
    pub street_threshold_metres: crate::scene_coordinates::ScenePlanPoint,
    pub street_apron: CityPlotBounds,
    pub geographic: &'a GeographicSurface,
    pub limits: SupportLimits,
    pub stairs: CourtStairLimits,
    pub embedment: FoundationEmbedment,
}

impl CompoundSupportRequest<'_> {
    pub fn select(self) -> Result<CompoundSupportPlan, SupportDiagnostic> {
        let mut observations = self.observations;
        observations.gate = compound::street_gate_level(self.property, observations, self.limits)?;
        match self.candidate(observations, CourtTreatment::Level) {
            Ok(level) => return Ok(level),
            Err(error)
                if matches!(
                    error.constraint,
                    SupportConstraint::CutFill
                        | SupportConstraint::AccessGrade
                        | SupportConstraint::StairGoing
                ) => {}
            Err(error) => return Err(error),
        }
        // The level plan has validated unique member/threshold bindings. Use
        // those exact routes to reserve both landings before selecting levels.
        let route = |member: MemberSupport| {
            self.property
                .access
                .iter()
                .find(|route| route.ends_at(member.court_threshold_metres.metres()))
                .ok_or_else(|| {
                    SupportDiagnostic::new(
                        self.property,
                        SupportConstraint::ThresholdBinding,
                        SupportBoundary::CourtLanding,
                        member.court_threshold_metres.metres(),
                        0.0,
                        1.0,
                    )
                })
        };
        let front_reach = self
            .stairs
            .maximum_rise_metres(route(self.observations.front)?, self.limits);
        let rear_reach = self
            .stairs
            .maximum_rise_metres(route(self.observations.rear)?, self.limits);
        if front_reach <= 0.0 || rear_reach <= 0.0 {
            let mut error = SupportDiagnostic::new(
                self.property,
                SupportConstraint::AccessGrade,
                SupportBoundary::CourtLanding,
                self.property.court.centre_metres(),
                (self.observations.front.elevation.metres() - self.observations.court.metres())
                    .abs(),
                front_reach.min(rear_reach).max(0.0),
            );
            error.attempted_treatment = Box::new(SupportGradingAttempt::Compound(
                CourtTreatment::Terraced(self.stairs),
            ));
            return Err(error);
        }
        let mut levels = observations;
        let front = levels.front.elevation.metres();
        let court = levels
            .court
            .metres()
            .clamp(front - front_reach, front + front_reach);
        levels.court = SupportElevation(court);
        levels.rear.elevation = SupportElevation(
            levels
                .rear
                .elevation
                .metres()
                .clamp(court - rear_reach, court + rear_reach),
        );
        self.candidate(levels, CourtTreatment::Terraced(self.stairs))
    }

    fn candidate(
        &self,
        levels: CompoundSupportLevels,
        treatment: CourtTreatment,
    ) -> Result<CompoundSupportPlan, SupportDiagnostic> {
        let unbound = CompoundSupportPlan::compile(self.property, levels, self.limits, treatment)?;
        let shift = unbound.bounded_floor_shift(self.geographic)?;
        let mut selected = unbound.levels;
        selected.front.elevation.0 += shift;
        selected.rear.elevation.0 += shift;
        selected.court.0 += shift;
        let plan = CompoundSupportPlan::compile(self.property, selected, self.limits, treatment)?
            .bind_street_entry(
            self.street_threshold_metres,
            self.street_apron,
            self.geographic,
            self.stairs,
        )?;
        plan.support_surface()?
            .validate_source_controls(self.geographic)?;
        Ok(plan)
    }
}
