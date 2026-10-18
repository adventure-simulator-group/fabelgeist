//! Measure cadence and phase from each body's retargeted foot trajectories.

use super::*;
use adventuresim_tactical_core::animation::AnimationPackId;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

#[derive(Component)]
pub(in crate::animation) struct CharacterLocomotionStrides {
    pub(in crate::animation) measurements: AuthoredLocomotionStrides,
    offsets: Vec<Vec3>,
}

struct CalibrationSource {
    motion: MotionId,
    axis: usize,
}

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum CalibrationMotion {
    Walk,
    Run,
    Strafe,
    Skip,
}

impl CalibrationMotion {
    fn source(self) -> CalibrationSource {
        CalibrationSource {
            motion: MotionId::from_static(self.into()),
            axis: match self {
                Self::Strafe => 0,
                Self::Walk | Self::Run | Self::Skip => 2,
            },
        }
    }
}

pub(in crate::animation) fn calibrate_character_strides(
    mut commands: Commands,
    reference: Res<AuthoredLocomotionStrides>,
    runtime: Res<AnimationRuntime>,
    clips: Res<Assets<AnimationClip>>,
    rigs: Query<(Entity, &PoseBufferRig, Option<&CharacterLocomotionStrides>)>,
    joints: Query<&skeletal_proportions::SkeletalJointOffset>,
) {
    for (entity, rig, current) in &rigs {
        let offsets: Vec<Vec3> = rig
            .entities
            .iter()
            .map(|joint| {
                joint
                    .and_then(|joint| joints.get(joint).ok())
                    .map(|offset| offset.0)
                    .unwrap_or_default()
            })
            .collect();
        if !reference.is_changed() && current.is_some_and(|current| current.offsets == offsets) {
            continue;
        }
        let mut measurements = AuthoredLocomotionStrides::default();
        calibrate(
            &runtime,
            &clips,
            &rig.definition,
            &mut measurements,
            &offsets,
        );
        commands.entity(entity).insert(CharacterLocomotionStrides {
            measurements,
            offsets,
        });
    }
}

pub(super) fn calibrate(
    runtime: &AnimationRuntime,
    clips: &Assets<AnimationClip>,
    definition: &RigDefinition,
    strides: &mut AuthoredLocomotionStrides,
    offsets: &[Vec3],
) {
    for kind in CalibrationMotion::iter() {
        let CalibrationSource { motion, axis } = kind.source();
        let Some(loaded) = runtime
            .clips
            .get(&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &motion))
        else {
            continue;
        };
        let id = loaded.handle.id();
        if strides.measured_clips.get(&motion) == Some(&id) {
            continue;
        }
        strides.clear_motion(&motion);
        let Some(clip) = clips.get(&loaded.handle) else {
            continue;
        };
        let mut baked = bake_clip(clip, definition);
        apply_baked_offsets(&mut baked, offsets);
        let calibration = match kind {
            // Walk and run use a measured distance-domain phase curve while
            // the live sampler interpolates only their sparse semantic poses.
            CalibrationMotion::Walk | CalibrationMotion::Run => {
                measure_authored_contact_step_distance(definition, &baked, axis, -1.0)
            }
            // Combat cycles currently expose alternating contact poses but no
            // typed support interval. Retain their geometric calibration until
            // that contact timing is part of the authored motion contract.
            CalibrationMotion::Strafe | CalibrationMotion::Skip => {
                measure_authored_foot_range(definition, &baked, axis).map(|step_distance| {
                    AuthoredLocomotionCalibration {
                        stride: AuthoredStrideMeasurement {
                            step_distance,
                            maximum_stance_slip: 0.0,
                        },
                        phase_curve: None,
                    }
                })
            }
        };
        let Some(calibration) = calibration else {
            warn!(motion = %motion, "Could not infer authored locomotion stride");
            strides.measured_clips.insert(motion.clone(), id);
            continue;
        };
        let AuthoredLocomotionCalibration {
            stride,
            phase_curve,
        } = calibration;
        if let Some(phase_curve) = phase_curve {
            strides.phase_curves.insert(motion.clone(), phase_curve);
        }
        info!(
            motion = %motion,
            stride_metres = stride.step_distance,
            maximum_stance_slip_metres = stride.maximum_stance_slip,
            "Measured authored locomotion stride"
        );
        if stride.maximum_stance_slip > presentation::maximum_authored_stance_slip_metres() {
            warn!(
                motion = %motion,
                stride_metres = stride.step_distance,
                maximum_stance_slip_metres = stride.maximum_stance_slip,
                "Authored locomotion contact fit exceeds the stance-slip budget"
            );
        }
        match kind {
            CalibrationMotion::Walk => strides.walk = Some(stride),
            CalibrationMotion::Run => strides.run = Some(stride),
            CalibrationMotion::Strafe => strides.strafe = Some(stride),
            CalibrationMotion::Skip => strides.skip = Some(stride),
        }
        strides.measured_clips.insert(motion.clone(), id);
    }
}

pub(super) fn has_unmeasured_motion(
    runtime: &AnimationRuntime,
    clips: &Assets<AnimationClip>,
    strides: &AuthoredLocomotionStrides,
) -> bool {
    CalibrationMotion::iter().any(|kind| {
        let motion = kind.source().motion;
        runtime
            .clips
            .get(&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &motion))
            .is_some_and(|loaded| {
                clips.contains(loaded.handle.id())
                    && strides.measured_clips.get(&motion) != Some(&loaded.handle.id())
            })
    })
}

fn apply_baked_offsets(clip: &mut BakedClip, offsets: &[Vec3]) {
    for (track, offset) in clip.tracks.iter_mut().zip(offsets) {
        for translation in &mut track.translations {
            *translation += *offset;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletal_proportions_remeasure_stride_from_actual_limb_lengths() {
        let definition = RigDefinition {
            family: "test".into(),
            joints: ["root", "l_foot", "r_foot"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| {
                    let identity = AuthoredRigJoint::new(name.into());
                    RigJoint {
                        target: AnimationTargetId::from_name(&Name::new(name)),
                        bind: LocalPose::from_transform(Transform::IDENTITY),
                        parent: (index != 0).then_some(RigJointOrdinal::from(0usize)),
                        name: Some(identity.name),
                        role: identity.role,
                        pelvis: identity.pelvis,
                        body_region: AnimationBodyRegion::Lower,
                    }
                })
                .collect(),
            name_ordinals: Default::default(),
        };
        let mut clip = BakedClip {
            duration: 1.0,
            frame_dt: 0.5,
            frames: 3,
            tracks: (0..3)
                .map(|index| BoneTrack {
                    translations: vec![if index == 0 { Vec3::Y } else { Vec3::NEG_Y }; 3],
                    rotations: if index == 0 {
                        vec![
                            Quat::from_rotation_x(-0.4),
                            Quat::IDENTITY,
                            Quat::from_rotation_x(0.4),
                        ]
                    } else {
                        vec![Quat::IDENTITY; 3]
                    },
                    scales: vec![Vec3::ONE; 3],
                    animated: true,
                })
                .collect(),
        };
        let original = measure_authored_foot_range(&definition, &clip, 2).unwrap();
        apply_baked_offsets(
            &mut clip,
            &[Vec3::ZERO, Vec3::NEG_Y * 0.5, Vec3::NEG_Y * 0.5],
        );
        let longer = measure_authored_foot_range(&definition, &clip, 2).unwrap();
        assert!((longer / original - 1.5).abs() < 1e-5);
    }
}
