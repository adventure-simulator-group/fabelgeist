use super::*;
use adventuresim_building_generator::spatial_geometry::SignedLength;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
use terrain_leg::OrdinaryLeg;

mod terrain_leg;

#[derive(Component, Debug, Clone, Copy, Default)]
pub(in crate::animation) struct OrdinaryLocomotionIkState {
    pelvis_shift: PelvisShiftState,
    evaluation_tick: Option<super::super::clock::AnimationSampleTick>,
}

#[derive(Debug, Clone, Copy, Default)]
enum PelvisShiftState {
    #[default]
    Pending,
    Tracking {
        /// Retained world-vertical correction, in signed metres.
        metres: SignedLength,
    },
}

impl OrdinaryLocomotionIkState {
    /// Admit the native leg-geometry correction before updating the retained tracking state.
    fn observe_terrain_reach(
        &mut self,
        desired: SignedLength,
        elapsed: AnimationElapsed,
    ) -> Option<SignedLength> {
        match self.pelvis_shift.advance(desired, elapsed) {
            Ok(shift) => Some(shift),
            Err(error) => {
                warn!(%error, "ordinary IK pelvis correction rejected");
                None
            }
        }
    }

    /// Elapsed IK interval, deduplicated by its typed sample tick.
    fn sample_delta_seconds(
        &mut self,
        clock: &ProceduralAnimationClock,
        time: &Time,
    ) -> AnimationElapsed {
        match clock.fixed_step() {
            Some(super::super::clock::FixedAnimationStep { tick, .. })
                if self.evaluation_tick == Some(tick) =>
            {
                AnimationElapsed::ZERO
            }
            Some(super::super::clock::FixedAnimationStep {
                tick,
                delta_seconds: delta,
            }) => {
                self.evaluation_tick = Some(tick);
                delta
            }
            None => AnimationElapsed::from(time.delta()),
        }
    }
}

impl PelvisShiftState {
    fn advance(
        &mut self,
        desired_metres: SignedLength,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<SignedLength> {
        let metres = match *self {
            Self::Pending => desired_metres,
            Self::Tracking { metres } if !elapsed.is_zero() => {
                advance_pelvis_shift(metres, desired_metres, elapsed)?
            }
            Self::Tracking { metres } => metres,
        };
        *self = Self::Tracking { metres };
        Ok(metres)
    }
}

/// World-space plant confidence used by diagnostics. Procedural guard movement
/// has exactly one support foot while the other follows its clearance arc.
pub(crate) fn locomotion_support_weights(
    skeleton: &SkeletonState,
    speed: NonnegativeSpeed,
) -> AnimationValueResult<(UnitInterval, UnitInterval)> {
    if !skeleton.is_grounded() {
        return Ok((UnitInterval::ZERO, UnitInterval::ZERO));
    }
    if !matches!(
        skeleton.action_kind(),
        SkeletonAction::None | SkeletonAction::Attack
    ) {
        return Ok((UnitInterval::ZERO, UnitInterval::ZERO));
    }
    if speed <= STATIONARY_ANIMATION_SPEED_LIMIT {
        return Ok((UnitInterval::ONE, UnitInterval::ONE));
    }
    if skeleton.weapon_guard() == WeaponGuardState::Raised
        && skeleton.raised_locomotion().is_moving()
    {
        Ok(match skeleton.contact_foot {
            LeadFoot::Left => (UnitInterval::ONE, UnitInterval::ZERO),
            LeadFoot::Right => (UnitInterval::ZERO, UnitInterval::ONE),
        })
    } else {
        let profile = locomotion_profile(skeleton, speed)?;
        let (left, right) = gait_support_weights(profile, skeleton.gait_phase)?;
        Ok(if profile.gait == LocomotionGait::Run {
            (contact_support_weight(left), contact_support_weight(right))
        } else {
            exclusive_ground_support(left, right, skeleton.gait_phase)
        })
    }
}

pub(in crate::animation::procedural) fn owns(skeleton: &SkeletonState) -> bool {
    skeleton.is_grounded()
        && !skeleton.is_posture_transitioning()
        && skeleton.posture() == Posture::Upright
        && !skeleton.is_quickstep()
        && ((skeleton.weapon_guard() == WeaponGuardState::Lowered
            && skeleton.action_kind() == SkeletonAction::None)
            || (skeleton.weapon_guard() == WeaponGuardState::Raised
                && skeleton.raised_locomotion().is_moving()))
}

/// Authored upright-locomotion IK: semantic evaluation supplies the complete
/// FK pose and frame-specific foot weights. This pass only conforms weighted
/// ankles to terrain, applies one shared hip correction, and solves each leg
/// once; it never plans a step or changes the authored horizontal trajectory.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each independently borrowed locomotion IK resource and query as a system parameter"
)]
pub(in crate::animation) fn apply(
    enabled: Res<super::super::super::TerrainIkMode>,
    time: Res<Time>,
    clock: Res<ProceduralAnimationClock>,
    terrain: Query<&SceneTerrain>,
    owners: Query<(&PresentedSkeleton, &AnimationPlayback)>,
    rigs: Query<(Entity, &HumanoidRig)>,
    parents: Query<&ChildOf>,
    mut states: Query<&mut OrdinaryLocomotionIkState>,
    mut diagnostics: Query<&mut LegIkState>,
    mut transforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
    mut commands: Commands,
) {
    let _spike = crate::animation::diagnostics::SpikeGuard::new("apply_ordinary_locomotion_ik");
    let terrain = terrain.single().ok();
    for (owner, rig) in &rigs {
        let Ok((skeleton, playback)) = owners.get(owner) else {
            continue;
        };
        if !owns(skeleton) {
            continue;
        }

        let frame = match ObservedRigFrame::resolve(rig, &transforms.p0()) {
            Ok(Some(frame)) => frame,
            Ok(None) => continue,
            Err(error) => {
                warn!(%error, ?owner, "Rejected ordinary IK rig frame");
                continue;
            }
        };
        let mut state = states
            .get_mut(owner)
            .map(|state| *state)
            .unwrap_or_default();
        let delta_seconds = state.sample_delta_seconds(&clock, &time);
        let authored_weights = playback.foot_ik_weights;
        let (left_authored_weight, right_authored_weight) =
            match UnitInterval::from_fraction(authored_weights.x).and_then(|left| {
                UnitInterval::from_fraction(authored_weights.y).map(|right| (left, right))
            }) {
                Ok(weights) => weights,
                Err(error) => {
                    warn!(%error, "authored IK support rejected");
                    continue;
                }
            };

        let solve_weights = if enabled.is_enabled() && terrain.is_some() {
            (left_authored_weight, right_authored_weight)
        } else {
            (UnitInterval::ZERO, UnitInterval::ZERO)
        };

        let legs = [
            OrdinaryLeg {
                upper_role: BoneRole::ThighLeft,
                lower_role: BoneRole::ShinLeft,
                foot_role: BoneRole::FootLeft,
                weight: solve_weights.0,
                foot_side: LeadFoot::Left,
            },
            OrdinaryLeg {
                upper_role: BoneRole::ThighRight,
                lower_role: BoneRole::ShinRight,
                foot_role: BoneRole::FootRight,
                weight: solve_weights.1,
                foot_side: LeadFoot::Right,
            },
        ];
        let mut targets = [None, None];
        let mut authored = [None, None];
        let mut support = if enabled.is_enabled() {
            [None, None]
        } else {
            [Some(left_authored_weight), Some(right_authored_weight)]
        };
        let mut desired_pelvis_shift = 0.0_f32;

        for (
            index,
            OrdinaryLeg {
                upper_role,
                lower_role,
                foot_role,
                weight,
                ..
            },
        ) in legs.iter().copied().enumerate()
        {
            let (Some(&upper), Some(&lower), Some(&foot), Some(terrain)) = (
                rig.get(&upper_role),
                rig.get(&lower_role),
                rig.get(&foot_role),
                terrain,
            ) else {
                continue;
            };
            let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
                snapshot_chain(upper, lower, foot, &parents, &transforms.p0())
            else {
                continue;
            };
            let foot_position = foot_snapshot.global.translation();
            authored[index] = Some(foot_position);
            let Some(height) = terrain.height_at(foot_position.xz()) else {
                continue;
            };
            let target = match checked_world_height(height)
                .and_then(|height| ordinary_terrain_target(foot_position, height, weight))
            {
                Ok(target) => target,
                Err(error) => {
                    warn!(%error, "authored terrain target rejected");
                    continue;
                }
            };
            let LimbSegments {
                upper: upper_length,
                lower: lower_length,
            } = match LimbSegments::measure(
                upper_snapshot.global.translation(),
                lower_snapshot.global.translation(),
                foot_position,
            ) {
                Ok(segments) => segments,
                Err(error) => {
                    warn!(%error, "IK segment measurement rejected");
                    continue;
                }
            };
            let reach = maximum_reach(upper_length, lower_length);
            let shift = match reach.ordinary_pelvis_shift(
                upper_snapshot.global.translation(),
                target,
                ik_tuning().maximum_hip_drop_metres,
                weight,
            ) {
                Ok(shift) => shift,
                Err(error) => {
                    warn!(%error, "ordinary pelvis reach rejected");
                    continue;
                }
            };
            targets[index] = Some(target);
            desired_pelvis_shift = desired_pelvis_shift.min(shift.metres());
        }

        let desired_pelvis_shift = match SignedLength::from_metres(desired_pelvis_shift) {
            Ok(shift) => shift,
            Err(_) => {
                warn!("ordinary IK pelvis geometry overflowed");
                continue;
            }
        };
        let Some(pelvis_shift) = state.observe_terrain_reach(desired_pelvis_shift, delta_seconds)
        else {
            continue;
        };
        apply_root_vertical_shift(rig, pelvis_shift, &parents, &mut transforms);

        for (index, leg) in legs.iter().copied().enumerate() {
            if let Some(target) = targets[index] {
                let observed = leg.solve(rig, target, frame, terrain, &parents, &mut transforms);
                if enabled.is_enabled() {
                    support[index] = observed;
                }
            }
        }

        let mut memory = LegIkMemory {
            left_authored_world_target: authored[0],
            right_authored_world_target: authored[1],
            left_foot_world_target: targets[0],
            right_foot_world_target: targets[1],
            left_support_weight: support[0],
            right_support_weight: support[1],
            pelvis_shift,
            evaluation_tick: state.evaluation_tick,
            ..default()
        };
        memory.left_foot_target = targets[0];
        memory.right_foot_target = targets[1];
        if let Err(error) = memory.publish(owner, &mut diagnostics, &mut commands) {
            warn!(%error, "ordinary IK targets rejected");
            continue;
        }
        if let Ok(mut current) = states.get_mut(owner) {
            *current = state;
        } else {
            commands.entity(owner).insert(state);
        }
    }
}

fn apply_root_vertical_shift(
    rig: &HumanoidRig,
    shift: SignedLength,
    parents: &Query<&ChildOf>,
    transforms: &mut ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    let shift = shift.metres();
    if shift.abs() <= 0.0001 {
        return;
    }
    let Some(&root) = rig.get(&BoneRole::Root) else {
        return;
    };
    let local_delta = parents
        .get(root)
        .ok()
        .and_then(|parent| {
            transforms
                .p0()
                .compute_global_transform(parent.parent())
                .ok()
        })
        .map(|parent| parent.affine().inverse().transform_vector3(Vec3::Y * shift))
        .unwrap_or(Vec3::Y * shift);
    if local_delta.is_finite()
        && let Ok(mut transform) = transforms.p1().get_mut(root)
    {
        transform.translation += local_delta;
    }
}

fn align_foot_to_slope_weighted(
    foot: Entity,
    sole_up_local: Vec3,
    normal: Vec3,
    weight: UnitInterval,
    parents: &Query<&ChildOf>,
    transforms: &mut ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    let Some(snapshot) = snapshot(foot, parents, &transforms.p0()) else {
        return;
    };
    let Some(aligned_world) =
        slope_aligned_world_rotation(snapshot.global.rotation(), sole_up_local, normal)
    else {
        return;
    };
    let desired_world = snapshot
        .global
        .rotation()
        .slerp(aligned_world, weight.fraction().clamp(0.0, 1.0));
    let Some(local) = local_rotation_for_world(snapshot.parent_rotation, desired_world) else {
        return;
    };
    if let Ok(mut transform) = transforms.p1().get_mut(foot) {
        transform.rotation = local;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_tactical_core::animation::{AttackAnimation, AttackSpec};

    #[test]
    fn authored_ownership_routes_only_translating_raised_guard_to_locomotion_ik() {
        let ordinary = SkeletonState::default();
        assert!(owns(&ordinary));

        let stationary_guard = ordinary.clone().with_weapon_guard(WeaponGuardState::Raised);
        assert!(!owns(&stationary_guard));

        let guard_step =
            stationary_guard
                .clone()
                .with_raised_locomotion(RaisedLocomotionIntent::moving(
                    Vec2::X,
                    adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(1.0)
                        .unwrap(),
                ));
        assert!(owns(&guard_step));

        let guarded_sprint = guard_step
            .clone()
            .with_guarded_locomotion_mode(GuardedLocomotionMode::Sprint);
        assert!(owns(&guarded_sprint));

        let mut attacking = guard_step;
        attacking
            .begin_attack(
                AttackSpec::default(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(1u64),
            )
            .unwrap();
        assert!(owns(&attacking));

        let mut quickstep = stationary_guard;
        quickstep
            .begin_dodge(
                DodgeSpec::quickstep(Vec2::X).unwrap(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(100u64),
            )
            .unwrap();
        assert!(!owns(&quickstep));
    }

    #[test]
    fn authored_posture_transition_excludes_ordinary_locomotion_ik() {
        let mut skeleton = SkeletonState::default();
        assert!(owns(&skeleton));
        assert!(skeleton.begin_posture_transition(
            PostureTransitionKind::UprightToProne,
            adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
            adventuresim_tactical_core::animation::AnimationSampleSpan::from(10u64),
        ));
        assert!(!owns(&skeleton));
    }

    #[test]
    fn raised_guard_movement_reports_exactly_one_procedural_support_foot() {
        for contact in [LeadFoot::Left, LeadFoot::Right] {
            let mut skeleton = SkeletonState::default()
                .with_local_velocity(Vec3::NEG_Z * 2.0)
                .with_weapon_guard(WeaponGuardState::Raised)
                .with_raised_locomotion(RaisedLocomotionIntent::moving(
                    Vec2::NEG_Y,
                    adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(2.0)
                        .unwrap(),
                ));
            skeleton.contact_foot = contact;
            let support =
                locomotion_support_weights(&skeleton, skeleton.animation_speed().unwrap()).unwrap();
            assert_eq!(support.0.fraction() + support.1.fraction(), 1.0);
            assert_eq!(
                support,
                match contact {
                    LeadFoot::Left => (UnitInterval::ONE, UnitInterval::ZERO),
                    LeadFoot::Right => (UnitInterval::ZERO, UnitInterval::ONE),
                }
            );
        }
        let idle = SkeletonState::default()
            .with_local_velocity(Vec3::ZERO)
            .with_weapon_guard(WeaponGuardState::Raised);
        assert_eq!(
            locomotion_support_weights(&idle, idle.animation_speed().unwrap()).unwrap(),
            (UnitInterval::ONE, UnitInterval::ONE)
        );
    }

    #[test]
    fn ordinary_swing_support_is_not_filled_in_by_low_speed() {
        let idle = SkeletonState::default().with_gait_phase(CyclePhase::from_cycles(0.25).unwrap());
        assert_eq!(
            locomotion_support_weights(&idle, idle.animation_speed().unwrap()).unwrap(),
            (UnitInterval::ONE, UnitInterval::ONE)
        );

        let stopping = SkeletonState::default()
            .with_gait_phase(CyclePhase::from_cycles(0.25).unwrap())
            .with_local_velocity(Vec3::NEG_Z * 0.2);
        let (left, right) =
            locomotion_support_weights(&stopping, stopping.animation_speed().unwrap()).unwrap();
        let (raw_left, raw_right) = gait_support_weights(
            locomotion_profile(&stopping, stopping.animation_speed().unwrap()).unwrap(),
            stopping.gait_phase,
        )
        .unwrap();
        assert!(left <= raw_left && right <= raw_right);
        assert!(left.fraction() <= 0.05 || right.fraction() <= 0.05);
    }

    #[test]
    fn attack_uses_the_live_guard_support_weights() {
        let mut skeleton = SkeletonState::default()
            .with_weapon_guard(WeaponGuardState::Raised)
            .with_local_velocity(Vec3::NEG_Z * 2.0)
            .with_raised_locomotion(RaisedLocomotionIntent::moving(
                Vec2::NEG_Y,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(2.0)
                    .unwrap(),
            ));
        let guard_weights =
            locomotion_support_weights(&skeleton, skeleton.animation_speed().unwrap()).unwrap();
        skeleton
            .begin_attack(
                AttackSpec::new(AttackAnimation::Swing),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        assert_eq!(
            locomotion_support_weights(&skeleton, skeleton.animation_speed().unwrap()).unwrap(),
            guard_weights
        );
    }
}
