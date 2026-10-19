use super::*;
use adventuresim_building_generator::spatial_geometry::SignedLength;

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
        desired: f32,
        elapsed: AnimationElapsed,
    ) -> Option<SignedLength> {
        let desired =
            SignedLength::from_metres(desired).map_err(|_| AnimationValueError::NonFinite {
                quantity: AnimationQuantity::HeightOffset,
            });
        match desired.and_then(|desired| self.pelvis_shift.advance(desired, elapsed)) {
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
            Self::Tracking { metres } if !elapsed.is_zero() => SignedLength::from_metres(
                advance_pelvis_shift(metres.metres(), desired_metres.metres(), elapsed),
            )
            .map_err(|_| AnimationValueError::Overflow {
                quantity: AnimationQuantity::HeightOffset,
            })?,
            Self::Tracking { metres } => metres,
        };
        *self = Self::Tracking { metres };
        Ok(metres)
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

        let mut state = states
            .get_mut(owner)
            .map(|state| *state)
            .unwrap_or_default();
        let delta_seconds = state.sample_delta_seconds(&clock, &time);
        let authored_weights = playback.foot_ik_weights;
        let solve_weights = if enabled.is_enabled() && terrain.is_some() {
            authored_weights
        } else {
            Vec2::ZERO
        };

        let legs = [
            (
                BoneRole::ThighLeft,
                BoneRole::ShinLeft,
                BoneRole::FootLeft,
                solve_weights.x,
                LeadFoot::Left,
            ),
            (
                BoneRole::ThighRight,
                BoneRole::ShinRight,
                BoneRole::FootRight,
                solve_weights.y,
                LeadFoot::Right,
            ),
        ];
        let mut targets = [None, None];
        let mut authored = [None, None];
        let mut desired_pelvis_shift = 0.0_f32;

        for (index, (upper_role, lower_role, foot_role, weight, _)) in
            legs.iter().copied().enumerate()
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
            // Pose-buffer terrain conformity owns the upcoming sample. This
            // compatibility solve may lift the rendered sample further when
            // terrain rises, but must never lower the authored flat-ground
            // clearance toward a fixed ankle offset.
            let terrain_target = foot_position.with_y(foot_position.y.max(
                height + measured_ankle_sole_offset_metres()
                    - ik_tuning().sole_contact_margin_metres,
            ));
            let target = foot_position.lerp(terrain_target, weight.clamp(0.0, 1.0));
            targets[index] = Some(target);

            let upper_length = upper_snapshot
                .global
                .translation()
                .distance(lower_snapshot.global.translation());
            let lower_length = lower_snapshot.global.translation().distance(foot_position);
            let reach = maximum_reach(upper_length, lower_length);
            desired_pelvis_shift = desired_pelvis_shift.min(
                required_hip_shift_for_reach(upper_snapshot.global.translation(), target, reach)
                    .clamp(-ik_tuning().maximum_hip_drop_metres, 0.0)
                    * weight,
            );
        }

        let Some(pelvis_shift) = state.observe_terrain_reach(desired_pelvis_shift, delta_seconds)
        else {
            continue;
        };
        apply_root_vertical_shift(rig, pelvis_shift, &parents, &mut transforms);
        let rig_rotation = rig
            .rig_scene()
            .and_then(|entity| transforms.p0().compute_global_transform(entity).ok())
            .map(|global| global.rotation())
            .unwrap_or(Quat::IDENTITY);

        for (index, (upper_role, lower_role, foot_role, weight, foot_side)) in
            legs.iter().copied().enumerate()
        {
            let (Some(target), Some(&upper), Some(&lower), Some(&foot)) = (
                targets[index],
                rig.get(&upper_role),
                rig.get(&lower_role),
                rig.get(&foot_role),
            ) else {
                continue;
            };
            let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
                snapshot_chain(upper, lower, foot, &parents, &transforms.p0())
            else {
                continue;
            };
            if target.distance(foot_snapshot.global.translation()) <= 0.0001 {
                // Preserve an already-clear authored chain exactly. Solving an
                // identity ankle target can still rotate it through a newly
                // selected knee pole.
                continue;
            }
            let upper_length = upper_snapshot
                .global
                .translation()
                .distance(lower_snapshot.global.translation());
            let lower_length = lower_snapshot
                .global
                .translation()
                .distance(foot_snapshot.global.translation());
            let canonical = canonical_knee_pole_world(foot_side, rig_rotation);
            let pole = authored_knee_pole_world(
                upper_snapshot.global.translation(),
                lower_snapshot.global.translation(),
                target,
                canonical,
            )
            .unwrap_or(canonical);
            let pole = constrain_rendered_leg_pole(
                rig,
                foot_side,
                upper_snapshot.global.translation(),
                foot_snapshot.global.translation(),
                target,
                pole,
                &parents,
                &transforms.p0(),
            );
            if let Some(solution) = solve_two_bone_with_reach(
                TwoBoneChain::new(
                    upper_snapshot.global.translation(),
                    lower_snapshot.global.translation(),
                    foot_snapshot.global.translation(),
                    upper_length,
                    lower_length,
                    pole,
                ),
                target,
                maximum_reach(upper_length, lower_length),
            ) {
                apply_two_bone_solution(upper, lower, foot, solution, &parents, &mut transforms);
            }
            if weight > 0.001
                && let (Some(_), Some(normal), Some(sole_axis)) = (
                    terrain,
                    terrain.and_then(|terrain| terrain.normal_at(target.xz())),
                    rig.sole_axis(foot_side),
                )
            {
                align_foot_to_slope_weighted(
                    foot,
                    sole_axis,
                    normal,
                    weight,
                    &parents,
                    &mut transforms,
                );
            }
        }

        let mut memory = LegIkMemory {
            left_authored_world_target: authored[0],
            right_authored_world_target: authored[1],
            left_foot_world_target: targets[0],
            right_foot_world_target: targets[1],
            left_support_weight: Some(authored_weights.x),
            right_support_weight: Some(authored_weights.y),
            pelvis_shift: pelvis_shift.metres(),
            evaluation_tick: state.evaluation_tick,
            ..default()
        };
        memory.left_foot_target = targets[0];
        memory.right_foot_target = targets[1];
        if let Ok(mut current) = diagnostics.get_mut(owner) {
            current.0 = memory;
        } else {
            commands.entity(owner).insert(LegIkState(memory));
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
    weight: f32,
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
        .slerp(aligned_world, weight.clamp(0.0, 1.0));
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

    #[test]
    fn authored_ownership_routes_only_translating_raised_guard_to_locomotion_ik() {
        let ordinary = SkeletonState::default();
        assert!(owns(&ordinary));

        let stationary_guard = ordinary.clone().with_weapon_guard(WeaponGuardState::Raised);
        assert!(!owns(&stationary_guard));

        let guard_step = stationary_guard
            .clone()
            .with_raised_locomotion(RaisedLocomotionIntent::moving(Vec2::X, 1.0));
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
}
