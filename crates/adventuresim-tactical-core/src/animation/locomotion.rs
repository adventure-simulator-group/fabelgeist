use super::*;

/// One bilateral locomotion cycle hands support between two contacts.
pub const LOCOMOTION_CONTACTS_PER_CYCLE: u8 = 2;
pub const LOCOMOTION_SECOND_CONTACT_PHASE: f32 = 0.5;

/// Fixed movement-presence policy used by foot support and presentation. This
/// deliberately remains distinct from configurable playback hysteresis speeds.
pub const STATIONARY_ANIMATION_SPEED_LIMIT: NonnegativeSpeed =
    match NonnegativeSpeed::from_metres_per_second(0.05) {
        Ok(speed) => speed,
        Err(_) => panic!("stationary animation speed limit must be finite and nonnegative"),
    };

/// Turning a downed body retains the established crawl-cycle minimum.
pub(super) const MINIMUM_DOWNED_ANIMATION_SPEED: NonnegativeSpeed =
    match NonnegativeSpeed::from_metres_per_second(0.8) {
        Ok(speed) => speed,
        Err(_) => panic!("invalid downed animation speed minimum"),
    };

/// Typed locomotion families shared by authoritative cadence projection and
/// client-only presentation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum LocomotionGait {
    #[default]
    Walk,
    Run,
    Downed,
    RaisedGuard,
}

/// Compact gait dynamics metadata. Phase 0..1 is one complete left/right
/// cycle; contact phases are 0 and 0.5.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocomotionProfile {
    pub gait: LocomotionGait,
    pub reference_speed: NonnegativeSpeed,
    pub step_distance: NonnegativeLength,
    /// Radius around each contact phase that can carry support.
    pub support_phase_radius: CycleDelta,
    /// Visual grounded bounce, in metres.
    pub bounce_metres: HeightDistance,
    /// Visual unsupported apex, in metres. Zero means a grounded curve.
    pub flight_apex_metres: HeightDistance,
    pub landing: LandingProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LandingProfile {
    pub minimum_impact_speed_metres_per_second: NonnegativeSpeed,
    pub compression_per_metre_per_second: ImpactCompressionCompliance,
    pub minimum_compression_metres: HeightDistance,
    pub maximum_compression_metres: HeightDistance,
    pub recovery_seconds: AnimationElapsed,
}

impl SkeletonState {
    /// Presentation motion finishes an in-flight raised-guard step after
    /// gameplay velocity stops. Speed otherwise follows authoritative motion.
    pub fn animation_local_velocity(&self) -> Vec3 {
        let raised = self.raised_locomotion();
        if self.weapon_guard() == WeaponGuardState::Raised && raised.is_moving() {
            raised.local_velocity()
        } else {
            self.local_velocity
        }
    }

    /// Measure animation motion in the body's horizontal plane. Reject a
    /// nonfinite norm before applying the existing downed-turning minimum.
    pub fn animation_speed(&self) -> AnimationValueResult<NonnegativeSpeed> {
        let physical = self.animation_local_velocity().xz().length();
        if !physical.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::Speed,
            });
        }
        let physical = NonnegativeSpeed::from_metres_per_second(physical)?;
        Ok(if self.is_downed_turning() {
            physical.max(MINIMUM_DOWNED_ANIMATION_SPEED)
        } else {
            physical
        })
    }
}

pub fn locomotion_sample_rate() -> super::AnimationSampleRate {
    crate::combat_config::runtime_animation_config()
        .locomotion
        .sample_hz
}

pub fn walk_locomotion_profile() -> LocomotionProfile {
    let config = crate::combat_config::runtime_animation_config()
        .locomotion
        .walk;
    gait_profile(LocomotionGait::Walk, config)
}

pub fn run_locomotion_profile() -> LocomotionProfile {
    let config = crate::combat_config::runtime_animation_config()
        .locomotion
        .run;
    gait_profile(LocomotionGait::Run, config)
}

pub fn raised_guard_locomotion_profile() -> LocomotionProfile {
    let config = crate::combat_config::runtime_animation_config()
        .locomotion
        .raised_guard;
    gait_profile(LocomotionGait::RaisedGuard, config)
}

pub fn prone_locomotion_profile() -> LocomotionProfile {
    let config = crate::combat_config::runtime_animation_config()
        .locomotion
        .prone;
    gait_profile(LocomotionGait::Downed, config)
}

pub fn supine_locomotion_profile() -> LocomotionProfile {
    let config = crate::combat_config::runtime_animation_config()
        .locomotion
        .supine;
    gait_profile(LocomotionGait::Downed, config)
}

pub fn locomotion_profile(
    state: &SkeletonState,
    speed: NonnegativeSpeed,
) -> AnimationValueResult<LocomotionProfile> {
    match state.body() {
        BodyState::Prone => return Ok(prone_locomotion_profile()),
        BodyState::Supine => return Ok(supine_locomotion_profile()),
        _ => {}
    }
    if state.weapon_guard() == WeaponGuardState::Raised && !state.is_guarded_sprinting() {
        return Ok(LocomotionProfile {
            step_distance: guard_step_length()?,
            ..raised_guard_locomotion_profile()
        });
    }
    let walk = walk_locomotion_profile();
    let run_profile = run_locomotion_profile();
    let run = speed.clamped_inverse_lerp(walk.reference_speed, run_profile.reference_speed)?;
    Ok(LocomotionProfile {
        gait: if run >= UnitInterval::HALF {
            LocomotionGait::Run
        } else {
            LocomotionGait::Walk
        },
        reference_speed: walk
            .reference_speed
            .checked_lerp(run_profile.reference_speed, run)?,
        step_distance: ordinary_step_distance(speed)?,
        support_phase_radius: walk
            .support_phase_radius
            .checked_lerp(run_profile.support_phase_radius, run)?,
        bounce_metres: walk.bounce_metres.scaled_by(run.complement())?,
        flight_apex_metres: run_profile.flight_apex_metres.scaled_by(run)?,
        landing: landing_profile(),
    })
}

/// Shared distance model through authored walk/run reference points.
pub fn ordinary_step_distance(speed: NonnegativeSpeed) -> AnimationValueResult<NonnegativeLength> {
    let walk = walk_locomotion_profile();
    let run = run_locomotion_profile();
    if speed <= walk.reference_speed {
        let zero_speed_step_distance = NonnegativeLength::from(
            crate::combat_config::runtime_animation_config()
                .locomotion
                .zero_speed_step_distance_metres,
        );
        zero_speed_step_distance.checked_lerp(
            walk.step_distance,
            speed.clamped_inverse_lerp(NonnegativeSpeed::ZERO, walk.reference_speed)?,
        )
    } else {
        walk.step_distance.checked_lerp(
            run.step_distance,
            speed.clamped_inverse_lerp(walk.reference_speed, run.reference_speed)?,
        )
    }
}

pub fn gait_cycle_phase_delta(
    profile: LocomotionProfile,
    speed: NonnegativeSpeed,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<CycleDelta> {
    // Speed and elapsed seconds meet only in this physical cadence kernel.
    CycleDelta::from_arithmetic(
        speed.metres_per_second().max(0.0) * elapsed.seconds().max(0.0)
            / (profile.step_distance.metres().max(0.01) * 2.0),
    )
}

pub fn gait_support_weights(
    profile: LocomotionProfile,
    phase: CyclePhase,
) -> AnimationValueResult<(UnitInterval, UnitInterval)> {
    if profile.gait == LocomotionGait::RaisedGuard {
        return Ok((UnitInterval::ONE, UnitInterval::ONE));
    }
    let phase = phase.cycles();
    let support = |contact: f32| {
        let distance = {
            let delta = (phase - contact).abs();
            delta.min(1.0 - delta)
        };
        (1.0 - smoothstep(
            profile.support_phase_radius.cycles() * 0.45,
            profile.support_phase_radius.cycles(),
            distance,
        ))
        .clamp(0.0, 1.0)
    };
    Ok((
        UnitInterval::from_fraction(support(0.0))?,
        UnitInterval::from_fraction(support(0.5))?,
    ))
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    if edge1 <= edge0 {
        return f32::from(value >= edge1);
    }
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn landing_profile() -> LandingProfile {
    let config = crate::combat_config::runtime_animation_config()
        .locomotion
        .landing;
    LandingProfile {
        minimum_impact_speed_metres_per_second: config.minimum_impact_speed_metres_per_second,
        compression_per_metre_per_second: config.compression_per_metre_per_second,
        minimum_compression_metres: config.minimum_compression_metres,
        maximum_compression_metres: config.maximum_compression_metres,
        recovery_seconds: config.recovery_seconds.elapsed(),
    }
}

fn gait_profile(
    gait: LocomotionGait,
    config: crate::combat_config::AnimationGaitConfig,
) -> LocomotionProfile {
    LocomotionProfile {
        gait,
        reference_speed: config.reference_speed_metres_per_second.into(),
        step_distance: config.step_distance_metres.into(),
        support_phase_radius: config.support_phase_radius,
        bounce_metres: config.bounce_metres,
        flight_apex_metres: config.flight_apex_metres,
        landing: landing_profile(),
    }
}
