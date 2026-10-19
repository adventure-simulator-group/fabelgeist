//! Retained run-flight and landing compression in one checked metre frame.

use super::height_values::{
    HeightDistance, HeightSpeed, add_offsets, advance_offset, subtract_offsets,
};
use super::*;
use crate::animation::presentation::playback_tuning;
use adventuresim_building_generator::spatial_geometry::SignedLength;
use adventuresim_tactical_core::animation::CyclePhase;

impl LocomotionHeightState {
    pub(super) fn advance_height(mut self, skeleton: &SkeletonState) -> AnimationValueResult<Self> {
        if skeleton.is_quickstep() {
            self.amplitude = HeightDistance::ZERO;
            self.authored_rise_compensation = HeightDistance::ZERO;
            self.displayed_wave = SignedLength::ZERO;
            self.wave_transition_offset = SignedLength::ZERO;
            self.landing_compression = HeightDistance::ZERO;
            self.landing_compression_target = HeightDistance::ZERO;
            clear_landing_foot_plants(&mut self);
            return Ok(self);
        }
        let target_wave = locomotion_height_wave(skeleton)?;
        let target_compensation = authored_height_compensation(skeleton)?;
        let previous_tick = self.history.map(|history| history.evaluation_tick);
        let mut history = match self.history {
            Some(history) => history,
            None => {
                self.amplitude = target_wave;
                self.authored_rise_compensation = target_compensation;
                // First observation seeds landing identity before landing detection.
                HeightHistory::observe(skeleton)
            }
        };
        let tick_delta = presentation_tick_delta(previous_tick, skeleton.locomotion_sample_tick)
            .unwrap_or_default();
        history.evaluation_tick = skeleton.locomotion_sample_tick;
        let elapsed =
            AnimationElapsed::from_seconds(tick_delta.samples() as f32 / locomotion_sample_hz())?;
        self.advance_amplitude(skeleton, target_wave, target_compensation, elapsed)?;
        self.advance_landing(skeleton, elapsed, &mut history)?;
        self.advance_wave(skeleton, elapsed, &mut history)?;
        self.history = Some(history);
        Ok(self)
    }

    pub(super) fn supports_foot_plants(&self, skeleton: &SkeletonState) -> bool {
        skeleton.is_grounded()
            && !skeleton.is_posture_transitioning()
            && skeleton.action_kind() == SkeletonAction::None
            && self.landing_compression.metres()
                > procedural_tuning()
                    .landing_plant_minimum_compression_metres
                    .metres()
    }

    fn advance_amplitude(
        &mut self,
        skeleton: &SkeletonState,
        wave: HeightDistance,
        compensation: HeightDistance,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<()> {
        let ordinary_stop = skeleton.is_grounded()
            && skeleton.action_kind() == SkeletonAction::None
            && skeleton.posture() == Posture::Upright
            && skeleton.animation_speed()?
                <= playback_tuning().initial_locomotion_speed_metres_per_second;
        self.amplitude = if ordinary_stop {
            self.amplitude.advance_towards(
                HeightDistance::ZERO,
                procedural_tuning()
                    .locomotion_stop_height_speed_metres_per_second
                    .over(elapsed)?,
            )?
        } else {
            wave
        };
        self.authored_rise_compensation = self.authored_rise_compensation.advance_towards(
            compensation,
            procedural_tuning()
                .height_transition_speed_metres_per_second
                .over(elapsed)?,
        )?;
        Ok(())
    }

    fn advance_landing(
        &mut self,
        skeleton: &SkeletonState,
        elapsed: AnimationElapsed,
        history: &mut HeightHistory,
    ) -> AnimationValueResult<()> {
        let landing_delta = skeleton
            .landing_sequence
            .events_since(history.landing_sequence)
            .filter(|delta| *delta <= runtime_animation_config().playback.maximum_coalesced_events);
        let landed = landing_delta.is_some_and(|delta| delta > 0);
        if skeleton.landing_sequence != history.landing_sequence {
            history.landing_sequence = skeleton.landing_sequence;
        }
        if landed {
            self.begin_landing(skeleton, history.action)?;
        }
        if self.landing_compression < self.landing_compression_target {
            self.landing_compression = self.landing_compression.advance_towards(
                self.landing_compression_target,
                self.landing_absorption_metres_per_second.over(elapsed)?,
            )?;
            if self
                .landing_compression
                .difference(self.landing_compression_target)?
                .metres()
                .abs()
                <= procedural_tuning()
                    .landing_compression_arrival_tolerance_metres
                    .metres()
            {
                self.landing_compression_target = HeightDistance::ZERO;
            }
        } else if !landed {
            self.landing_compression = self.landing_compression.advance_towards(
                HeightDistance::ZERO,
                self.landing_recovery_metres_per_second.over(elapsed)?,
            )?;
        }
        if !self.supports_foot_plants(skeleton) {
            clear_landing_foot_plants(self);
        }
        Ok(())
    }

    fn begin_landing(
        &mut self,
        skeleton: &SkeletonState,
        previous_action: SkeletonAction,
    ) -> AnimationValueResult<()> {
        if skeleton.is_posture_transitioning() {
            self.landing_compression = HeightDistance::ZERO;
            self.landing_compression_target = HeightDistance::ZERO;
            self.landing_absorption_metres_per_second = HeightSpeed::ZERO;
            self.landing_recovery_metres_per_second = HeightSpeed::ZERO;
        } else {
            self.landing_compression_target = landing_compression_for_action(
                locomotion_profile(skeleton, skeleton.animation_speed()?)?,
                skeleton.landing_impact_speed,
                Some(previous_action),
            )?;
            self.landing_absorption_metres_per_second = HeightSpeed::from_metres_per_second(
                skeleton.landing_impact_speed.metres_per_second().max(
                    procedural_tuning()
                        .minimum_landing_absorption_speed_metres_per_second
                        .metres_per_second(),
                ),
            )?;
            self.landing_recovery_metres_per_second = HeightSpeed::recovery(
                self.landing_compression_target,
                locomotion_profile(skeleton, skeleton.animation_speed()?)?
                    .landing
                    .recovery_seconds,
            )?;
        }
        clear_landing_foot_plants(self);
        Ok(())
    }

    fn advance_wave(
        &mut self,
        skeleton: &SkeletonState,
        elapsed: AnimationElapsed,
        history: &mut HeightHistory,
    ) -> AnimationValueResult<()> {
        let compensation =
            grounded_height_wave(skeleton.gait_phase, self.authored_rise_compensation)?;
        let raw_wave = self.amplitude.difference(compensation)?;
        let state_changed = history.guard != skeleton.weapon_guard()
            || history.posture != skeleton.posture()
            || history.action != skeleton.action_kind()
            || history.ground_contact != GroundContact::from(skeleton);
        self.wave_transition_offset = if state_changed {
            subtract_offsets(self.displayed_wave, raw_wave)?
        } else {
            advance_offset(
                self.wave_transition_offset,
                procedural_tuning()
                    .height_transition_speed_metres_per_second
                    .over(elapsed)?,
            )?
        };
        self.displayed_wave = add_offsets(raw_wave, self.wave_transition_offset)?;
        history.guard = skeleton.weapon_guard();
        history.posture = skeleton.posture();
        history.action = skeleton.action_kind();
        history.ground_contact = GroundContact::from(skeleton);
        Ok(())
    }
}

/// Additive run-flight waveform with contacts at 0/.5 and flight peaks at
/// .25/.75. Authored joint translation remains the base pose. This is
/// presentation-only and is never applied to the authoritative owner/controller
/// transform.
pub(super) fn locomotion_height_wave(
    skeleton: &SkeletonState,
) -> AnimationValueResult<HeightDistance> {
    if !skeleton.is_grounded()
        || skeleton.is_posture_transitioning()
        || skeleton.action_kind() != SkeletonAction::None
    {
        return Ok(HeightDistance::ZERO);
    }
    if skeleton.weapon_guard() == WeaponGuardState::Raised && !skeleton.is_guarded_sprinting() {
        return Ok(HeightDistance::ZERO);
    }
    let speed = skeleton.animation_speed()?;
    if speed <= playback_tuning().initial_locomotion_speed_metres_per_second {
        return Ok(HeightDistance::ZERO);
    }
    let profile = locomotion_profile(skeleton, speed)?;
    let moving_weight = smoothstep(
        playback_tuning()
            .initial_locomotion_speed_metres_per_second
            .metres_per_second(),
        runtime_animation_config()
            .locomotion
            .blend_speed
            .metres_per_second(),
        speed.metres_per_second(),
    );
    if profile.flight_apex_metres <= 0.0 {
        return Ok(HeightDistance::ZERO);
    }
    let half_step = (skeleton.gait_phase.cycles().rem_euclid(0.5) * 2.0).clamp(0.0, 1.0);
    let flight = (half_step * std::f32::consts::PI).sin().powi(2) * profile.flight_apex_metres;
    HeightDistance::from_metres(flight * moving_weight)
}
pub(super) fn authored_height_compensation(
    skeleton: &SkeletonState,
) -> AnimationValueResult<HeightDistance> {
    if !skeleton.is_grounded()
        || skeleton.action_kind() != SkeletonAction::None
        || skeleton.animation_pack != AnimationPackId::HUMANOID_UNARMED
        || skeleton.posture() != Posture::Upright
        || skeleton.weapon_guard() != WeaponGuardState::Lowered
    {
        return Ok(HeightDistance::ZERO);
    }
    let run_weight = locomotion_profile(skeleton, skeleton.animation_speed()?)?.flight_apex_metres
        / run_locomotion_profile().flight_apex_metres;
    if !run_weight.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::HeightDistance,
        });
    }
    HeightDistance::from_metres(
        procedural_tuning()
            .authored_ordinary_passing_rise_metres
            .metres()
            * run_weight.clamp(0.0, 1.0)
            * smoothstep(
                playback_tuning()
                    .initial_locomotion_speed_metres_per_second
                    .metres_per_second(),
                runtime_animation_config()
                    .locomotion
                    .blend_speed
                    .metres_per_second(),
                skeleton.animation_speed()?.metres_per_second(),
            ),
    )
}
pub(super) fn landing_compression_for_impact(
    profile: LocomotionProfile,
    impact_speed: LandingImpactSpeed,
) -> AnimationValueResult<HeightDistance> {
    let impact_speed = impact_speed.metres_per_second();
    if impact_speed
        < profile
            .landing
            .minimum_impact_speed_metres_per_second
            .metres_per_second()
    {
        return Ok(HeightDistance::ZERO);
    }
    let compression = impact_speed * profile.landing.compression_per_metre_per_second;
    if !compression.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::HeightDistance,
        });
    }
    HeightDistance::from_metres(compression.clamp(
        profile.landing.minimum_compression_metres,
        profile.landing.maximum_compression_metres,
    ))
}
pub(super) fn landing_compression_for_action(
    profile: LocomotionProfile,
    impact_speed: LandingImpactSpeed,
    _previous_action: Option<SkeletonAction>,
) -> AnimationValueResult<HeightDistance> {
    landing_compression_for_impact(profile, impact_speed)
}
pub(super) fn clear_landing_foot_plants(height: &mut LocomotionHeightState) {
    height.landing_left_foot_target = None;
    height.landing_right_foot_target = None;
    height.landing_plant_owner_position = None;
    height.landing_plant_tick = None;
    height.landing_plant_resync_tick = None;
}

fn grounded_height_wave(
    phase: CyclePhase,
    amplitude: HeightDistance,
) -> AnimationValueResult<HeightDistance> {
    // A native periodic metre-amplitude kernel; the signed offset is computed afterward.
    let sine = (phase.wrapped().cycles() * std::f32::consts::TAU).sin();
    HeightDistance::from_metres(amplitude.metres() * sine * sine)
}
