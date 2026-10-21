//! Observation-backed camera pose history; profile blending advances independently.

use super::values::{CameraResult, SignedBlendVelocity};
use super::{
    CameraProfile, CameraRigConfig,
    collision::BoomRecovery,
    smoothing::{blend_profile, critical_damp_scalar},
};
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_core::linear_speed::SignedSpeed;
use adventuresim_tactical_core::animation::{AnimationElapsed, UnitInterval, WeaponGuardState};
use adventuresim_tactical_core::scene_coordinates::{Scene, SceneVelocity};
use bevy::prelude::{Entity, Resource};
use fabelgeist_geometry::SignedLength;

#[derive(Resource, Debug, Clone, Default)]
pub(super) struct CameraRigState {
    history: Option<CameraPoseHistory>,
    pub(super) blend: UnitInterval,
    // Signed fraction/second derivative used only by the analytic scalar filter.
    blend_velocity: SignedBlendVelocity,
}

#[derive(Debug, Clone)]
pub(super) struct CameraPoseHistory {
    pub(super) focus: Position<Scene>,
    pub(super) focus_velocity: SceneVelocity,
    pub(super) boom: BoomRecovery,
    pub(super) shoulder_offset: SignedLength,
    pub(super) shoulder_velocity: SignedSpeed,
    last_anchor: Position<Scene>,
    subject: Entity,
}

impl CameraRigState {
    /// Prepare checked blend/profile results before committing either filter value.
    pub(super) fn advance_profile(
        &mut self,
        guard: WeaponGuardState,
        config: &CameraRigConfig,
        elapsed: AnimationElapsed,
    ) -> CameraResult<CameraProfile> {
        let target_blend = f32::from(guard == WeaponGuardState::Raised);
        let mut velocity = self.blend_velocity.fraction_per_second();
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
        let velocity = SignedBlendVelocity::from_fraction_per_second(velocity)?;
        self.blend = blend;
        self.blend_velocity = velocity;
        Ok(profile)
    }

    pub(super) fn reset_observation(&mut self) {
        self.history = None;
    }

    pub(super) fn observe(
        &self,
        subject: Entity,
        anchor: Position<Scene>,
        profile: CameraProfile,
        teleport_distance: SignedLength,
    ) -> CameraPoseHistory {
        let reset = self.history.as_ref().is_none_or(|history| {
            history.subject != subject
                || anchor.metres().distance(history.last_anchor.metres())
                    > teleport_distance.metres()
        });
        let mut history = match &self.history {
            Some(history) if !reset => history.clone(),
            _ => CameraPoseHistory::new(subject, anchor, profile),
        };
        history.last_anchor = anchor;
        history.subject = subject;
        history
    }

    pub(super) fn commit_observation(&mut self, history: CameraPoseHistory) {
        self.history = Some(history);
    }
}

impl CameraPoseHistory {
    fn new(subject: Entity, anchor: Position<Scene>, profile: CameraProfile) -> Self {
        let mut boom = BoomRecovery::default();
        boom.reset(profile.distance);
        Self {
            focus: anchor,
            focus_velocity: SceneVelocity::ZERO,
            boom,
            shoulder_offset: profile.shoulder_offset,
            shoulder_velocity: SignedSpeed::ZERO,
            last_anchor: anchor,
            subject,
        }
    }
}
