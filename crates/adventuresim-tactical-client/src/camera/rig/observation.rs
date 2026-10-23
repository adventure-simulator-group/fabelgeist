//! Complete checked camera-frame preparation, before retained history is replaced.
use super::*;
use crate::camera::{
    collision::{CameraFrame, CameraVolume},
    rig_history::CameraPoseHistory,
    smoothing::{camera_view_metrics, critical_damp_scalar, damp_focus, sweet_spot_target},
};
use adventuresim_tactical_core::animation::UnitInterval;

pub(super) struct CameraSubjectObservation<'a> {
    pub(super) entity: Entity,
    pub(super) transform: &'a Transform,
    pub(super) controller: &'a CharacterControllerState,
    pub(super) skeleton: Option<&'a SkeletonState>,
}

pub(super) struct CameraRigInput {
    subject: Entity,
    origin: Position<Scene>,
    pub(super) anchor: Position<Scene>,
    rotation: RigidRotation,
    profile: CameraProfile,
    raised_blend: UnitInterval,
    elapsed: AnimationElapsed,
}

impl CameraRigInput {
    pub(super) fn observe(
        subject: CameraSubjectObservation<'_>,
        camera: &Transform,
        profile: CameraProfile,
        raised_blend: UnitInterval,
        config: &CameraRigConfig,
        elapsed: AnimationElapsed,
    ) -> CameraResult<Self> {
        let crouch_adjustment = match subject.skeleton.map(SkeletonState::body) {
            Some(BodyState::Prone) => config.prone_focus_adjustment,
            Some(BodyState::Supine) => config.supine_focus_adjustment,
            _ if subject.controller.crouching => config.crouch_focus_adjustment,
            _ => SignedLength::ZERO,
        };
        Ok(Self {
            subject: subject.entity,
            origin: Position::from_metres(subject.transform.translation)?,
            anchor: Position::from_metres(
                subject.transform.translation
                    + Vec3::Y * (profile.focus_height.metres() + crouch_adjustment.metres()),
            )?,
            rotation: RigidRotation::from_quaternion(camera.rotation)?,
            profile,
            raised_blend,
            elapsed,
        })
    }

    pub(super) fn prepare(
        self,
        history: &mut CameraPoseHistory,
        projection: &Projection,
        spatial: &SpatialQuery,
        soft_occluders: &Query<(), With<CameraSoftOccluder>>,
        config: &CameraRigConfig,
    ) -> CameraResult<CameraRigObservation> {
        let screen_error = self.prepare_follow(history, projection, config)?;
        let frame = CameraFrame {
            origin: self.origin,
            anchor: self.anchor,
            focus: history.focus,
            rotation: self.rotation,
            shoulder_offset: history.shoulder_offset,
            distance: self.profile.distance,
        };
        let volume = CameraVolume::new(projection, config.collision_margin)?;
        let placement = history.boom.place(
            &frame,
            &volume,
            spatial,
            &SpatialQueryFilter::from_excluded_entities([self.subject]),
            &|entity| !soft_occluders.contains(entity),
            config,
            self.elapsed,
        )?;
        let subject_delta = self.anchor.metres() - placement.position.metres();
        let subject_distance = subject_delta.length();
        let subject_direction = Dir3::new(subject_delta).unwrap_or(Dir3::NEG_Z);
        let soft_occlusion = spatial.cast_ray_predicate(
            placement.position.metres(),
            subject_direction,
            subject_distance,
            true,
            &SpatialQueryFilter::from_excluded_entities([self.subject]),
            &|entity| soft_occluders.contains(entity),
        );
        let soft_occluder = soft_occlusion
            .map(|hit| {
                Ok::<_, CameraError>(SoftOcclusion {
                    entity: hit.entity,
                    point: Position::from_metres(
                        placement.position.metres() + *subject_direction * hit.distance,
                    )?,
                })
            })
            .transpose()?;
        Ok(CameraRigObservation {
            raised_blend: self.raised_blend,
            subject: self.anchor,
            focus: history.focus,
            collision_origin: frame.origin,
            collision_volume: volume.debug_volume(placement.position, self.rotation)?,
            desired_endpoint: placement.desired,
            final_endpoint: placement.position,
            collision: placement.hit,
            soft_occluder,
            desired_distance: self.profile.distance,
            limited_distance: placement.limited_distance,
            focus_velocity: history.focus_velocity,
            boom_velocity: history.boom.velocity,
            screen_error,
            sweet_spot: self.profile.sweet_spot,
        })
    }

    fn prepare_follow(
        &self,
        history: &mut CameraPoseHistory,
        projection: &Projection,
        config: &CameraRigConfig,
    ) -> CameraResult<super::super::values::ScreenError> {
        let sweet_spot = sweet_spot_target(
            self.anchor,
            history.focus,
            self.rotation,
            history.boom.distance,
            self.profile.sweet_spot,
            camera_view_metrics(projection),
        )?;
        let focus = damp_focus(
            history.focus,
            sweet_spot.target,
            history.focus_velocity,
            self.profile,
            self.elapsed,
        )?;
        history.focus = focus.position;
        history.focus_velocity = focus.velocity;
        let follow_error = self.anchor.metres() - history.focus.metres();
        if follow_error.length() > self.profile.maximum_follow_error.metres() {
            history.focus = Position::from_metres(
                self.anchor.metres()
                    - follow_error.normalize() * self.profile.maximum_follow_error.metres(),
            )?;
            history.focus_velocity = SceneVelocity::from_metres_per_second(
                history
                    .focus_velocity
                    .metres_per_second()
                    .reject_from_normalized(follow_error.normalize()),
            )?;
        }
        let mut shoulder_velocity = history.shoulder_velocity.metres_per_second();
        history.shoulder_offset = SignedLength::from_metres(critical_damp_scalar(
            history.shoulder_offset.metres(),
            self.profile.shoulder_offset.metres(),
            &mut shoulder_velocity,
            config.transition_time,
            self.elapsed,
        ))?;
        history.shoulder_velocity = SignedSpeed::from_metres_per_second(shoulder_velocity)?;
        Ok(sweet_spot.error)
    }
}
