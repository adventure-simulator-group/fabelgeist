//! Raised-guard stepping, support handoff, and stationary-turn ownership.

use super::*;

// Cascadeur's final ankle bones sit about 15 mm inside analytic targets after
// the complete hierarchy solve. Keep a measured planning allowance so the
// rendered bones, not merely abstract targets, retain the 0.16 m contract.

/// A world-space rendering of one server-authored guard swing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::animation::procedural) struct GuardSwing {
    pub(super) start: Vec3,
    pub(super) end: Vec3,
    pub(super) progress: UnitInterval,
}

/// The complete contact topology for raised-guard locomotion. Unlike the old
/// collection of swing flags, plants, and pivot fields, every inhabited
/// variant has at least one support foot and exactly identifies the other
/// foot's role.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RaisedFootworkState {
    pub(super) step: GuardStepState,
    pub(super) step_sequence: ContactSequence,
    pub(crate) left_support_weight: UnitInterval,
    pub(crate) right_support_weight: UnitInterval,
    pub(crate) left_solve_target: Option<Vec3>,
    pub(crate) right_solve_target: Option<Vec3>,
    pub(super) left_knee_bend_world: Option<Vec3>,
    pub(super) right_knee_bend_world: Option<Vec3>,
    pub(super) left_end_direction: Option<Vec3>,
    pub(super) right_end_direction: Option<Vec3>,
}
#[derive(Debug, Clone, Copy)]
pub(in crate::animation::procedural) struct GuardTargetRequest {
    pub(super) left: Vec3,
    pub(super) right: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::animation::procedural) struct GuardLegGeometry {
    pub(super) hip: Vec3,
    pub(super) maximum_reach: ReachLimit,
}

/// A finite raised-guard IK request. A temporarily unreachable airborne target
/// is lifted inside the leg's reach without changing its horizontal swing
/// trajectory. The support target remains immutable and each leg solves
/// independently, so one exhausted leg cannot cancel the other leg's swing.
#[derive(Debug, Clone, Copy)]
pub(in crate::animation::procedural) struct ValidatedGuardTargets {
    pub(super) targets: GuardTargetRequest,
    pub(super) adjusted_for_reach: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(in crate::animation::procedural) enum GuardStepState {
    #[default]
    Uninitialized,
    Stationary {
        left: Vec3,
        right: Vec3,
        next: LeadFoot,
    },
    LeftSwing {
        right_support: Vec3,
        left: GuardSwing,
    },
    RightSwing {
        left_support: Vec3,
        right: GuardSwing,
    },
}

impl GuardStepState {
    pub(super) fn initialized(self) -> bool {
        !matches!(self, Self::Uninitialized)
    }

    pub(super) fn swing_foot(self) -> Option<LeadFoot> {
        match self {
            Self::LeftSwing { .. } => Some(LeadFoot::Left),
            Self::RightSwing { .. } => Some(LeadFoot::Right),
            Self::Uninitialized | Self::Stationary { .. } => None,
        }
    }

    pub(super) fn progress(self) -> Option<UnitInterval> {
        match self {
            Self::LeftSwing { left, .. } => Some(left.progress),
            Self::RightSwing { right, .. } => Some(right.progress),
            Self::Uninitialized | Self::Stationary { .. } => None,
        }
    }

    pub(super) fn visual_targets(self) -> Option<GuardTargetRequest> {
        match self {
            Self::Uninitialized => None,
            Self::Stationary { left, right, .. } => Some(GuardTargetRequest { left, right }),
            Self::LeftSwing {
                right_support,
                left,
            } => Some(GuardTargetRequest {
                left: guard_swing_target(left),
                right: right_support,
            }),
            Self::RightSwing {
                left_support,
                right,
            } => Some(GuardTargetRequest {
                left: left_support,
                right: guard_swing_target(right),
            }),
        }
    }

    pub(super) fn retained_targets(self) -> Option<(Vec3, Vec3)> {
        match self {
            Self::Uninitialized => None,
            Self::Stationary { left, right, .. } => Some((left, right)),
            Self::LeftSwing {
                right_support,
                left,
            } => Some((left.start, right_support)),
            Self::RightSwing {
                left_support,
                right,
            } => Some((left_support, right.start)),
        }
    }
}

impl Default for RaisedFootworkState {
    fn default() -> Self {
        Self {
            step: GuardStepState::Uninitialized,
            step_sequence: ContactSequence::ZERO,
            left_support_weight: UnitInterval::ZERO,
            right_support_weight: UnitInterval::ZERO,
            left_solve_target: None,
            right_solve_target: None,
            left_knee_bend_world: None,
            right_knee_bend_world: None,
            left_end_direction: None,
            right_end_direction: None,
        }
    }
}

impl RaisedFootworkState {
    /// Validate the completed local candidate before either retained component
    /// is published. Current native poses are independently updated by the solve.
    pub(super) fn validate_targets(&self) -> AnimationValueResult<()> {
        let step = match self.step {
            GuardStepState::Uninitialized => [None, None, None],
            GuardStepState::Stationary { left, right, .. } => [Some(left), Some(right), None],
            GuardStepState::LeftSwing {
                right_support,
                left,
            } => [Some(right_support), Some(left.start), Some(left.end)],
            GuardStepState::RightSwing {
                left_support,
                right,
            } => [Some(left_support), Some(right.start), Some(right.end)],
        };
        if step
            .into_iter()
            .chain([self.left_solve_target, self.right_solve_target])
            .flatten()
            .any(|target| !target.is_finite())
        {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::WorldPosition,
            });
        }
        Ok(())
    }

    pub(crate) fn initialized(&self) -> bool {
        self.step.initialized()
    }

    pub(crate) fn step_sequence(&self) -> ContactSequence {
        self.step_sequence
    }

    pub(crate) fn contact_foot(&self) -> Option<LeadFoot> {
        match self.step {
            GuardStepState::Uninitialized => None,
            GuardStepState::Stationary { next, .. } => Some(opposite_guard_foot(next)),
            GuardStepState::LeftSwing { .. } => Some(LeadFoot::Right),
            GuardStepState::RightSwing { .. } => Some(LeadFoot::Left),
        }
    }
}

impl GuardTargetRequest {
    /// When the body catches a visual swing, reserve its authored forward room.
    pub(super) fn keep_swing_ahead(
        &mut self,
        skeleton: &SkeletonState,
        frame: ObservedRigFrame,
        hip_center: Vec3,
    ) {
        if let Some(step) = skeleton.raised_footwork().step() {
            let local_direction = step.direction();
            let planned_direction = (frame.rotation()
                * Vec3::new(local_direction.x, 0.0, local_direction.y))
            .with_y(0.0)
            .normalize_or_zero();
            let physical_direction = skeleton.world_velocity.with_y(0.0).normalize_or_zero();
            let direction = if physical_direction == Vec3::ZERO {
                planned_direction
            } else {
                physical_direction
            };
            if direction != Vec3::ZERO {
                let support = match step.swing_foot() {
                    LeadFoot::Left => self.right,
                    LeadFoot::Right => self.left,
                };
                let swing = match step.swing_foot() {
                    LeadFoot::Left => &mut self.left,
                    LeadFoot::Right => &mut self.right,
                };
                let shortfall =
                    hip_center.dot(direction) - support.dot(direction).max(swing.dot(direction));
                if shortfall > 0.0 {
                    *swing += direction * shortfall;
                }
            }
        }
    }
}

impl ValidatedGuardTargets {
    pub(super) fn left(self) -> Vec3 {
        self.targets.left
    }

    pub(super) fn right(self) -> Vec3 {
        self.targets.right
    }

    pub(super) fn adjusted_for_reach(self) -> bool {
        self.adjusted_for_reach
    }
}

pub(in crate::animation::procedural) fn opposite_guard_foot(foot: LeadFoot) -> LeadFoot {
    match foot {
        LeadFoot::Left => LeadFoot::Right,
        LeadFoot::Right => LeadFoot::Left,
    }
}

pub(in crate::animation::procedural) fn safer_guard_reacquire_foot(
    current_left: Vec3,
    current_right: Vec3,
    desired_left: Vec3,
    desired_right: Vec3,
    fallback: LeadFoot,
) -> LeadFoot {
    let left_first_clearance = desired_left.xz().distance(current_right.xz());
    let right_first_clearance = desired_right.xz().distance(current_left.xz());
    if (left_first_clearance - right_first_clearance).abs() <= 0.001 {
        fallback
    } else if left_first_clearance > right_first_clearance {
        LeadFoot::Left
    } else {
        LeadFoot::Right
    }
}

pub(in crate::animation::procedural) fn guard_swing_target(swing: GuardSwing) -> Vec3 {
    let progress = smootherstep01(swing.progress.fraction());
    let mut target = swing.start.lerp(swing.end, progress);
    target.y += (std::f32::consts::PI * progress).sin() * 0.10;
    target
}

/// Keep stationary combat feet fixed in world space while the owner turns.
/// Once the rotating authored stance pulls a plant beyond the pole/reach
/// corridor, lift exactly one foot through the existing guard swing and place
/// it at the new authored stance target. Translating authored locomotion never
/// enters this path and is therefore allowed to slide instead of inventing
/// extra up/down foot motion.
pub(in crate::animation::procedural) fn advance_stationary_turn_step(
    current: GuardStepState,
    requested: GuardStepState,
    delta_seconds: AnimationElapsed,
) -> AnimationValueResult<GuardStepState> {
    let GuardStepState::Stationary {
        left: desired_left,
        right: desired_right,
        next: _requested_next,
    } = requested
    else {
        return Ok(requested);
    };
    Ok(match current {
        GuardStepState::Uninitialized => requested,
        GuardStepState::Stationary { left, right, next } => {
            // Pole-limit stepping responds to horizontal stance displacement.
            // Pose-buffer/terrain settling may move the requested ankle height
            // after touchdown; treating that vertical correction as another
            // reach violation immediately relaunches the foot that just landed.
            let left_exceeded = left.xz().distance(desired_left.xz())
                > ik_tuning().stationary_turn_foot_limit_metres.metres();
            let right_exceeded = right.xz().distance(desired_right.xz())
                > ik_tuning().stationary_turn_foot_limit_metres.metres();
            let swing = match (left_exceeded, right_exceeded, next) {
                (false, false, _) => return Ok(GuardStepState::Stationary { left, right, next }),
                (true, false, _) => LeadFoot::Left,
                (false, true, _) => LeadFoot::Right,
                (true, true, next) => next,
            };
            match swing {
                LeadFoot::Left => GuardStepState::LeftSwing {
                    right_support: right,
                    left: GuardSwing {
                        start: left,
                        end: desired_left,
                        progress: UnitInterval::ZERO,
                    },
                },
                LeadFoot::Right => GuardStepState::RightSwing {
                    left_support: left,
                    right: GuardSwing {
                        start: right,
                        end: desired_right,
                        progress: UnitInterval::ZERO,
                    },
                },
            }
        }
        GuardStepState::LeftSwing {
            right_support,
            mut left,
        } => {
            left.end = desired_left;
            left.progress = ik_tuning()
                .stationary_turn_step_seconds
                .advance_guard(left.progress, delta_seconds)?;
            if left.progress == UnitInterval::ONE {
                GuardStepState::Stationary {
                    left: left.end,
                    right: right_support,
                    next: LeadFoot::Right,
                }
            } else {
                GuardStepState::LeftSwing {
                    right_support,
                    left,
                }
            }
        }
        GuardStepState::RightSwing {
            left_support,
            mut right,
        } => {
            right.end = desired_right;
            right.progress = ik_tuning()
                .stationary_turn_step_seconds
                .advance_guard(right.progress, delta_seconds)?;
            if right.progress == UnitInterval::ONE {
                GuardStepState::Stationary {
                    left: left_support,
                    right: right.end,
                    next: LeadFoot::Left,
                }
            } else {
                GuardStepState::RightSwing {
                    left_support,
                    right,
                }
            }
        }
    })
}

pub(in crate::animation::procedural) fn validate_guard_frame_targets(
    requested: GuardTargetRequest,
    geometry: [GuardLegGeometry; 2],
    swing_foot: Option<LeadFoot>,
) -> Option<ValidatedGuardTargets> {
    if !requested.left.is_finite()
        || !requested.right.is_finite()
        || geometry.iter().any(|leg| {
            !leg.hip.is_finite()
                || leg
                    .maximum_reach
                    .native_metres()
                    .is_none_or(|reach| !reach.is_finite())
        })
    {
        return None;
    }
    let targets = GuardTargetRequest {
        left: if swing_foot == Some(LeadFoot::Left) {
            constrain_guard_swing_to_reach(requested.left, geometry[0])?
        } else {
            requested.left
        },
        right: if swing_foot == Some(LeadFoot::Right) {
            constrain_guard_swing_to_reach(requested.right, geometry[1])?
        } else {
            requested.right
        },
    };
    if !targets.left.is_finite() || !targets.right.is_finite() {
        return None;
    }
    Some(ValidatedGuardTargets {
        targets,
        adjusted_for_reach: targets.left != requested.left || targets.right != requested.right,
    })
}

/// Keeps the horizontal swing path monotonic even when the current hip has not
/// yet travelled close enough to reach a ground-level sample. Radial projection
/// shortens XZ as well as Y, which makes an ankle pause and then catch up near
/// contact. Lifting the airborne sample spends the available reach vertically
/// and lets it descend naturally as the hip approaches. Only a target whose
/// horizontal offset alone exceeds the whole leg falls back to radial clamping.
pub(in crate::animation::procedural) fn constrain_guard_swing_to_reach(
    target: Vec3,
    geometry: GuardLegGeometry,
) -> Option<Vec3> {
    let maximum_reach = geometry.maximum_reach.native_metres()?.max(0.0);
    let offset = target - geometry.hip;
    let horizontal_distance_squared = offset.x * offset.x + offset.z * offset.z;
    let reach_squared = maximum_reach * maximum_reach;
    if horizontal_distance_squared > reach_squared {
        let vertical = target.y - geometry.hip.y;
        let horizontal_reach = (maximum_reach * maximum_reach - vertical * vertical)
            .max(0.0)
            .sqrt();
        let horizontal = (target - geometry.hip)
            .xz()
            .clamp_length_max(horizontal_reach);
        return Some(Vec3::new(
            geometry.hip.x + horizontal.x,
            target.y,
            geometry.hip.z + horizontal.y,
        ));
    }

    let vertical_reach = (reach_squared - horizontal_distance_squared)
        .max(0.0)
        .sqrt();
    Some(Vec3::new(
        target.x,
        target.y.clamp(
            geometry.hip.y - vertical_reach,
            geometry.hip.y + vertical_reach,
        ),
        target.z,
    ))
}

pub(in crate::animation::procedural) fn anatomical_side(
    rig_rotation: Quat,
    rig_origin: Vec3,
    hip: Vec3,
    foot_side: LeadFoot,
) -> FootTrackSide {
    let hip_x = (rig_rotation.inverse() * (hip - rig_origin)).x;
    if hip_x.abs() > 0.001 {
        FootTrackSide::from_native_x(hip_x)
    } else {
        FootTrackSide::for_foot(foot_side)
    }
}

pub(in crate::animation::procedural) fn constrain_foot_to_track(
    world: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    side: FootTrackSide,
) -> Vec3 {
    let mut local = rig_rotation.inverse() * (world - rig_origin);
    let signed_x = (local.x * side.coefficient()).clamp(
        foot_track_inner_metres().metres(),
        ik_tuning().outer_foot_track_metres.metres(),
    );
    local.x = signed_x * side.coefficient();
    rig_origin + rig_rotation * local
}
pub(in crate::animation::procedural) fn terrain_conformed_guard_target(
    mut flat_target: Vec3,
    terrain_height: Option<f32>,
) -> Vec3 {
    if let Some(height) = terrain_height {
        flat_target.y = height + measured_ankle_sole_offset_metres().metres();
    }
    flat_target
}
