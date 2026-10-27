use super::{AnimationLandingConfig, TacticalCombatConfigError};
use crate::animation::{AnimationSampleRate, CycleDelta, HeightDistance, UnitInterval};
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use adventuresim_core::linear_speed::{NonnegativeSpeed, PositiveSpeed};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationLocomotionConfig {
    pub sample_hz: AnimationSampleRate,
    pub landing: AnimationLandingConfig,
    pub walk: AnimationGaitConfig,
    pub run: AnimationGaitConfig,
    pub raised_guard: AnimationGaitConfig,
    pub prone: AnimationGaitConfig,
    pub supine: AnimationGaitConfig,
    pub blend_speed: PositiveSpeed,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationGaitConfig {
    pub reference_speed_metres_per_second: PositiveSpeed,
    pub step_distance_metres: PositiveLength,
    pub support_phase_radius: CycleDelta,
    pub bounce_metres: HeightDistance,
    pub flight_apex_metres: HeightDistance,
}

impl AnimationLocomotionConfig {
    pub(super) fn validate(
        &self,
        initial_locomotion_speed: NonnegativeSpeed,
    ) -> Result<(), TacticalCombatConfigError> {
        let gaits = [
            self.walk,
            self.run,
            self.raised_guard,
            self.prone,
            self.supine,
        ];
        if gaits.into_iter().any(|gait| {
            gait.support_phase_radius < CycleDelta::ZERO
                || gait.support_phase_radius >= CycleDelta::from_unit_cycles(UnitInterval::HALF)
        }) {
            return Err(TacticalCombatConfigError::Validation(
                "invalid animation locomotion tuning",
            ));
        }
        if self.run.reference_speed_metres_per_second <= self.walk.reference_speed_metres_per_second
        {
            return Err(TacticalCombatConfigError::Validation(
                "run reference speed must exceed walk reference speed",
            ));
        }
        if self.blend_speed.nonnegative() <= initial_locomotion_speed {
            return Err(TacticalCombatConfigError::Validation(
                "locomotion blend speed must exceed initial locomotion speed",
            ));
        }
        self.landing.validate()
    }
}
