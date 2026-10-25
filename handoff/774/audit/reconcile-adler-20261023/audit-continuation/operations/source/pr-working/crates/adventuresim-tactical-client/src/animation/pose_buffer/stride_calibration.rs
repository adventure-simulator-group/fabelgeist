//! Measure cadence and phase from each body's retargeted foot trajectories.

use super::phase_calibration::StanceDirection;
use super::*;
use adventuresim_tactical_core::animation::AnimationPackId;
use strum::IntoEnumIterator;

#[derive(Component)]
pub(in crate::animation) struct CharacterLocomotionStrides {
    pub(in crate::animation) measurements: AuthoredLocomotionStrides,
    offsets: Vec<Vec3>,
    definition: Arc<RigDefinition>,
}

struct CalibrationSource {
    motion: MotionId,
    axis: CalibrationAxis,
}

/// Travel component in the authored foot-trajectory frame.
#[derive(Clone, Copy)]
pub(super) enum CalibrationAxis {
    Lateral,
    Longitudinal,
}

impl CharacterLocomotionStrides {
    pub(super) fn matches_offsets(
        &self,
        rig: &PoseBufferRig,
        joints: &Query<&skeletal_proportions::SkeletalJointOffset>,
    ) -> bool {
        Arc::ptr_eq(&self.definition, &rig.definition)
            && self.offsets.len() == rig.entities.len()
            && self
                .offsets
                .iter()
                .zip(rig.entities.iter())
                .all(|(previous, joint)| {
                    let current = joint
                        .and_then(|entity| joints.get(entity).ok())
                        .map(|offset| offset.0)
                        .unwrap_or_default();
                    *previous == current
                })
    }
}

impl CalibrationAxis {
    pub(super) fn component(
        self,
        position: Vec3,
    ) -> AnimationValueResult<adventuresim_building_generator::spatial_geometry::SignedLength> {
        let component = match self {
            Self::Lateral => position.x,
            Self::Longitudinal => position.z,
        };
        adventuresim_building_generator::spatial_geometry::SignedLength::from_metres(component)
            .map_err(|_| AnimationValueError::NonFinite {
                quantity: AnimationQuantity::CalibrationPosition,
            })
    }
}

impl CalibrationMotion {
    fn source(self) -> CalibrationSource {
        CalibrationSource {
            motion: self.motion(),
            axis: match self {
                Self::Strafe => CalibrationAxis::Lateral,
                Self::Walk | Self::Run | Self::Skip => CalibrationAxis::Longitudinal,
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
        if !reference.is_changed()
            && current.is_some_and(|current| {
                Arc::ptr_eq(&current.definition, &rig.definition) && current.offsets == offsets
            })
        {
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
            definition: Arc::clone(&rig.definition),
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
    strides.invalidate_configuration();
    for kind in CalibrationMotion::iter() {
        let CalibrationSource { motion, axis } = kind.source();
        let Some(loaded) = runtime
            .clips
            .get(&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &motion))
        else {
            continue;
        };
        let id = loaded.handle.id();
        if strides.measured_clips.get(&kind) == Some(&id) {
            continue;
        }
        let Some(clip) = clips.get(&loaded.handle) else {
            continue;
        };
        let mut baked = match bake_clip(clip, definition) {
            Ok(baked) => baked,
            Err(error) => {
                warn!(%error, motion = %motion, "Rejected stride bake grid");
                continue;
            }
        };
        baked.offset_translations(offsets);
        let calibration = match kind {
            // Walk and run use a measured distance-domain phase curve while
            // the live sampler interpolates only their sparse semantic poses.
            CalibrationMotion::Walk | CalibrationMotion::Run => {
                measure_authored_contact_step_distance(
                    definition,
                    &baked,
                    axis,
                    StanceDirection::Decreasing,
                )
            }
            // Combat cycles currently expose alternating contact poses but no
            // typed support interval. Retain their geometric calibration until
            // that contact timing is part of the authored motion contract.
            CalibrationMotion::Strafe | CalibrationMotion::Skip => {
                measure_authored_foot_range(definition, &baked, axis).map(|range| {
                    range.map(|step_distance| AuthoredLocomotionCalibration {
                        stride: AuthoredStrideMeasurement {
                            step_distance,
                            maximum_stance_slip: NonnegativeLength::ZERO,
                        },
                        phase_curve: None,
                    })
                })
            }
        };
        let calibration = match calibration {
            Ok(calibration) => calibration,
            Err(error) => {
                warn!(%error, motion = %motion, "Rejected authored phase calibration");
                continue;
            }
        };
        // Prepare first. F08 revision invalidation separately retires changed sources.
        strides.clear_motion(kind);
        let Some(calibration) = calibration else {
            warn!(motion = %motion, "Could not infer authored locomotion stride");
            strides.measured_clips.insert(kind, id);
            continue;
        };
        let AuthoredLocomotionCalibration {
            stride,
            phase_curve,
        } = calibration;
        if let Some(phase_curve) = phase_curve {
            strides.phase_curves.insert(kind, phase_curve);
        }
        info!(
            motion = %motion,
            stride_metres = stride.step_distance.metres(),
            maximum_stance_slip_metres = stride.maximum_stance_slip.metres(),
            "Measured authored locomotion stride"
        );
        if stride.maximum_stance_slip > presentation::maximum_authored_stance_slip_metres() {
            warn!(
                motion = %motion,
                stride_metres = stride.step_distance.metres(),
                maximum_stance_slip_metres = stride.maximum_stance_slip.metres(),
                "Authored locomotion contact fit exceeds the stance-slip budget"
            );
        }
        match kind {
            CalibrationMotion::Walk => strides.walk = Some(stride),
            CalibrationMotion::Run => strides.run = Some(stride),
            CalibrationMotion::Strafe => strides.strafe = Some(stride),
            CalibrationMotion::Skip => strides.skip = Some(stride),
        }
        strides.measured_clips.insert(kind, id);
    }
}

pub(super) fn has_unmeasured_motion(
    runtime: &AnimationRuntime,
    clips: &Assets<AnimationClip>,
    strides: &AuthoredLocomotionStrides,
) -> bool {
    !strides.has_current_configuration()
        || CalibrationMotion::iter().any(|kind| {
            let motion = kind.source().motion;
            runtime
                .clips
                .get(&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &motion))
                .is_some_and(|loaded| {
                    clips.contains(loaded.handle.id())
                        && strides.measured_clips.get(&kind) != Some(&loaded.handle.id())
                })
        })
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
                        name: Some(identity.name().clone()),
                        role: identity.role(),
                        body_region: AnimationBodyRegion::Lower,
                    }
                })
                .collect(),
            name_ordinals: Default::default(),
        };
        let mut clip = BakedClip::new(
            ClipDuration::from_seconds(1.0).unwrap(),
            adventuresim_tactical_core::animation::PositiveAnimationDuration::try_from(0.5)
                .unwrap(),
            BakedFrameCount::try_from(3).unwrap(),
            (0..3)
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
                    source: BoneTrackSource::Authored,
                })
                .collect(),
        )
        .unwrap();
        let original =
            measure_authored_foot_range(&definition, &clip, CalibrationAxis::Longitudinal)
                .unwrap()
                .unwrap();
        clip.offset_translations(&[Vec3::ZERO, Vec3::NEG_Y * 0.5, Vec3::NEG_Y * 0.5]);
        let longer = measure_authored_foot_range(&definition, &clip, CalibrationAxis::Longitudinal)
            .unwrap()
            .unwrap();
        assert!((longer.metres() / original.metres() - 1.5).abs() < 1e-5);
    }
}
