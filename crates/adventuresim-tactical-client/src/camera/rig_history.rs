//! Observation-backed camera pose history; profile blending advances independently.

use super::{
    CameraProfile, CameraRigConfig,
    collision::BoomRecovery,
    smoothing::{blend_profile, critical_damp_scalar},
};
use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult, UnitInterval,
    WeaponGuardState,
};
use bevy::prelude::{Entity, Resource, Vec3};

#[derive(Resource, Debug, Clone, Default)]
pub(super) struct CameraRigState {
    history: Option<CameraPoseHistory>,
    pub(super) blend: UnitInterval,
    // Signed fraction/second derivative used only by the analytic scalar filter.
    blend_velocity: f32,
}

#[derive(Debug, Clone)]
pub(super) struct CameraPoseHistory {
    pub(super) focus: Vec3,
    pub(super) focus_velocity: Vec3,
    pub(super) boom: BoomRecovery,
    pub(super) shoulder_offset: f32,
    pub(super) shoulder_velocity: f32,
    last_anchor: Vec3,
    subject: Entity,
}

impl CameraRigState {
    /// Prepare checked blend/profile results before committing either filter value.
    pub(super) fn advance_profile(
        &mut self,
        guard: WeaponGuardState,
        config: &CameraRigConfig,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<CameraProfile> {
        let target_blend = f32::from(guard == WeaponGuardState::Raised);
        let mut velocity = self.blend_velocity;
        let value = critical_damp_scalar(
            self.blend.fraction(),
            target_blend,
            &mut velocity,
            config.transition_time,
            elapsed,
        )
        .clamp(0.0, 1.0);
        let blend = UnitInterval::from_fraction(value)?;
        let profile = blend_profile(config.lowered, config.raised, blend)?;
        if !velocity.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::CameraBlendVelocity,
            });
        }
        self.blend = blend;
        self.blend_velocity = velocity;
        Ok(profile)
    }

    pub(super) fn reset_observation(&mut self) {
        self.history = None;
    }

    pub(super) fn observe(
        &mut self,
        subject: Entity,
        anchor: Vec3,
        profile: CameraProfile,
        teleport_distance: f32,
    ) -> &mut CameraPoseHistory {
        let reset = self.history.as_ref().is_none_or(|history| {
            history.subject != subject || anchor.distance(history.last_anchor) > teleport_distance
        });
        if reset {
            self.history = None;
        }
        let history = self
            .history
            .get_or_insert_with(|| CameraPoseHistory::new(subject, anchor, profile));
        history.last_anchor = anchor;
        history.subject = subject;
        history
    }
}

impl CameraPoseHistory {
    fn new(subject: Entity, anchor: Vec3, profile: CameraProfile) -> Self {
        let mut boom = BoomRecovery::default();
        boom.reset(profile.distance);
        Self {
            focus: anchor,
            focus_velocity: Vec3::ZERO,
            boom,
            shoulder_offset: profile.shoulder_offset,
            shoulder_velocity: 0.0,
            last_anchor: anchor,
            subject,
        }
    }
}
