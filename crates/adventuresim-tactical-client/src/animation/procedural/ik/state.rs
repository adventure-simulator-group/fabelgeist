//! Transient retained state and diagnostics. None of this tick-local animation
//! authority is persisted outside the Bevy presentation world.

use super::*;

use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::{
    AnimationSampleTick, STATIONARY_ANIMATION_SPEED_LIMIT,
};

pub(super) mod publication;
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::animation::procedural) struct LocomotionSettleState {
    pub(super) support_foot: LeadFoot,
    pub(super) swing_start: Vec3,
    pub(super) capture_point: Vec3,
    pub(super) landing_target: Vec3,
    pub(super) progress: SettleProgress,
    pub(super) elapsed_seconds: AnimationElapsed,
    pub(super) raised_handoff: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::animation::procedural) enum SlopeAlignmentMode {
    Raised,
    Ordinary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::animation::procedural) struct LegRotationChain {
    pub(super) upper: Quat,
    pub(super) lower: Quat,
    pub(super) foot: Quat,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(in crate::animation::procedural) struct LegIkMemory {
    pub(super) left_leg: Option<Vec3>,
    pub(super) right_leg: Option<Vec3>,
    pub(super) left_terrain_pole_world: Option<Vec3>,
    pub(super) right_terrain_pole_world: Option<Vec3>,
    pub(super) left_terrain_end_direction: Option<Vec3>,
    pub(super) right_terrain_end_direction: Option<Vec3>,
    pub(super) left_knee_foot_yaw_offset_degrees: AngularMagnitudeDegrees,
    pub(super) right_knee_foot_yaw_offset_degrees: AngularMagnitudeDegrees,
    pub(super) left_rotation_chain: Option<LegRotationChain>,
    pub(super) right_rotation_chain: Option<LegRotationChain>,
    pub(super) left_foot_orientation_world: Option<Quat>,
    pub(super) right_foot_orientation_world: Option<Quat>,
    pub(super) left_contact_orientation_blend_active: bool,
    pub(super) right_contact_orientation_blend_active: bool,
    pub(super) slope_alignment_mode: Option<SlopeAlignmentMode>,
    pub(super) left_foot_plant: FootPlant,
    pub(super) right_foot_plant: FootPlant,

    pub(super) left_foot_target: Option<Vec3>,
    pub(super) right_foot_target: Option<Vec3>,
    pub(super) left_foot_world_target: Option<Vec3>,
    pub(super) right_foot_world_target: Option<Vec3>,
    // The quickstep solver writes its final visible landing stance here. The
    // ordinary raised-guard follower consumes it on the first post-action
    // frame instead of reacquiring the authored feet from scratch.
    pub(super) quickstep_handoff: QuickstepContactHandoff,
    // The last propagated ankle positions are the last pose the player
    // actually saw. At the start of a stop, FK has already restored the new
    // idle sample before IK runs, so sampling globals in the IK pass would
    // mistake that authored pose for the preceding rendered run pose.
    pub(super) left_last_rendered_world: Option<Vec3>,
    pub(super) right_last_rendered_world: Option<Vec3>,
    pub(super) left_last_rendered_toe_world: Option<Vec3>,
    pub(super) right_last_rendered_toe_world: Option<Vec3>,
    pub(super) left_last_rendered_owner: Option<Vec3>,
    pub(super) right_last_rendered_owner: Option<Vec3>,
    pub(super) left_last_rendered_foot_rotation_world: Option<Quat>,
    pub(super) right_last_rendered_foot_rotation_world: Option<Quat>,
    pub(super) left_authored_world_target: Option<Vec3>,
    pub(super) right_authored_world_target: Option<Vec3>,
    pub(super) left_planned_contact: Option<Vec3>,
    pub(super) right_planned_contact: Option<Vec3>,
    pub(super) left_planned_contact_start: Option<Vec3>,
    pub(super) right_planned_contact_start: Option<Vec3>,
    pub(super) left_planned_contact_phase_start: Option<CycleDelta>,
    pub(super) right_planned_contact_phase_start: Option<CycleDelta>,
    pub(super) left_support_weight: Option<UnitInterval>,
    pub(super) right_support_weight: Option<UnitInterval>,
    // Solver ownership is separate from truthful post-propagation contact
    // diagnostics. A rendered miss may report zero without erasing the fact
    // that the next solve must release from the preceding planted chain.
    pub(super) left_transition_support_weight: Option<UnitInterval>,
    pub(super) right_transition_support_weight: Option<UnitInterval>,
    pub(super) left_support_exhausted_until_flight: bool,
    pub(super) right_support_exhausted_until_flight: bool,
    pub(super) left_release_active: bool,
    pub(super) right_release_active: bool,
    pub(super) left_release_target: Option<Vec3>,
    pub(super) right_release_target: Option<Vec3>,
    pub(super) pelvis_shift: SignedLength,
    // Terminal stop correction is an absolute offset from the local rig-root
    // pose captured when dual-contact convergence begins. Sparse idle clips
    // do not necessarily rewrite that root every tick, so adding the retained
    // ordinary pelvis scalar repeatedly stalls or double-applies correction.
    pub(super) terminal_contacts: TerminalContacts,
    pub(super) raised_pelvis_shift: SignedLength,
    pub(super) terrain_blend: UnitInterval,
    pub(super) owner_frame: Option<ObservedRigFrame>,
    pub(super) measured_owner_planar_speed: Option<NonnegativeSpeed>,
    pub(super) evaluation_tick: Option<AnimationSampleTick>,
    pub(super) recent_movement_velocity: Vec3,
    pub(super) settle: Option<LocomotionSettleState>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(in crate::animation::procedural) enum QuickstepContactHandoff {
    #[default]
    None,
    Converging {
        left: Vec3,
        right: Vec3,
    },
    Held {
        left: Vec3,
        right: Vec3,
    },
}

impl QuickstepContactHandoff {
    pub(super) fn is_pending(self) -> bool {
        matches!(self, Self::Converging { .. })
    }

    pub(super) fn is_held(self) -> bool {
        matches!(self, Self::Held { .. })
    }

    pub(super) fn targets(self) -> Option<(Vec3, Vec3)> {
        match self {
            Self::None => None,
            Self::Converging { left, right } | Self::Held { left, right } => Some((left, right)),
        }
    }

    pub(super) fn update_targets(&mut self, left: Vec3, right: Vec3) {
        *self = match self {
            Self::Converging { .. } => Self::Converging { left, right },
            Self::Held { .. } => Self::Held { left, right },
            Self::None => Self::None,
        };
    }

    pub(super) fn hold(&mut self) {
        if let Self::Converging { left, right } = *self {
            *self = Self::Held { left, right };
        }
    }
}

pub(in crate::animation::procedural) fn seed_quickstep_contact_handoff(
    memory: &mut LegIkMemory,
    frame: ObservedRigFrame,
    left_world: Vec3,
    right_world: Vec3,
) -> bool {
    if !left_world.is_finite() || !right_world.is_finite() {
        return false;
    }
    let (rig_origin, rig_rotation) = (frame.origin(), frame.rotation());
    let inverse_rotation = rig_rotation.inverse();
    let left = inverse_rotation * (left_world - rig_origin);
    let right = inverse_rotation * (right_world - rig_origin);
    if !left.is_finite() || !right.is_finite() {
        return false;
    }
    memory.quickstep_handoff = QuickstepContactHandoff::Converging { left, right };
    memory.owner_frame = Some(frame);
    true
}

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::animation::procedural) struct ArmIkMemory {
    pub(super) left_arm: Option<Vec3>,
    pub(super) right_arm: Option<Vec3>,
}

pub(in crate::animation::procedural) fn repeated_fixed_tick_skips_ik(
    fixed_tick: bool,
    evaluation_advances: bool,
) -> bool {
    fixed_tick && !evaluation_advances
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct LegIkState(pub(super) LegIkMemory);

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct LegIkDiagnostics {
    pub left_authored_target: Option<Vec3>,
    pub right_authored_target: Option<Vec3>,
    pub left_planned_contact: Option<Vec3>,
    pub right_planned_contact: Option<Vec3>,
    pub settle_capture_point: Option<Vec3>,
    pub left_solve_target: Option<Vec3>,
    pub right_solve_target: Option<Vec3>,
    pub left_support_weight: Option<UnitInterval>,
    pub right_support_weight: Option<UnitInterval>,
    pub left_release_active: bool,
    pub right_release_active: bool,
    pub left_release_target: Option<Vec3>,
    pub right_release_target: Option<Vec3>,
    pub settle_progress: Option<SettleProgress>,
    pub left_knee_foot_yaw_offset_degrees: AngularMagnitudeDegrees,
    pub right_knee_foot_yaw_offset_degrees: AngularMagnitudeDegrees,
}

impl LegIkMemory {
    /// A zero interval holds history. Otherwise admit the observed frame and
    /// displacement measurement before writing history or retiring contacts.
    pub(super) fn observe_owner_frame(
        &mut self,
        frame: ObservedRigFrame,
        elapsed: AnimationElapsed,
        evaluation_advances: bool,
    ) -> AnimationValueResult<()> {
        if elapsed.is_zero() {
            return Ok(());
        }
        let origin = frame.origin();
        let rotation = frame.rotation();
        let previous_origin = self.owner_frame.map(ObservedRigFrame::origin);
        let owner_discontinuous = previous_origin.is_some_and(|previous| {
            previous.distance(origin)
                > ik_tuning()
                    .maximum_owner_translation_per_tick_metres
                    .metres()
        }) || self
            .owner_frame
            .map(ObservedRigFrame::rotation)
            .is_some_and(|previous| {
                previous.angle_between(rotation).to_degrees()
                    > ik_tuning()
                        .maximum_owner_rotation_per_tick_degrees
                        .degrees()
            });
        self.measured_owner_planar_speed = update_measured_owner_planar_speed(
            self.measured_owner_planar_speed,
            previous_origin,
            origin,
            elapsed,
            evaluation_advances,
            owner_discontinuous,
        )?;
        if owner_discontinuous {
            self.left_foot_plant = FootPlant::Absent;
            self.right_foot_plant = FootPlant::Absent;
            self.left_foot_target = None;
            self.right_foot_target = None;
            self.left_foot_world_target = None;
            self.right_foot_world_target = None;
            self.left_last_rendered_world = None;
            self.right_last_rendered_world = None;
            self.left_last_rendered_toe_world = None;
            self.right_last_rendered_toe_world = None;
            self.left_last_rendered_owner = None;
            self.right_last_rendered_owner = None;
            self.left_last_rendered_foot_rotation_world = None;
            self.right_last_rendered_foot_rotation_world = None;
            self.left_authored_world_target = None;
            self.right_authored_world_target = None;
            clear_all_planned_contact_metadata(self);
            self.left_support_weight = None;
            self.right_support_weight = None;
            self.left_transition_support_weight = None;
            self.right_transition_support_weight = None;
            self.left_support_exhausted_until_flight = false;
            self.right_support_exhausted_until_flight = false;
            self.left_terrain_pole_world = None;
            self.right_terrain_pole_world = None;
            self.left_terrain_end_direction = None;
            self.right_terrain_end_direction = None;
            self.left_foot_orientation_world = None;
            self.right_foot_orientation_world = None;
            self.left_contact_orientation_blend_active = false;
            self.right_contact_orientation_blend_active = false;
            clear_slope_rotation_cache(self);
            self.left_release_active = false;
            self.right_release_active = false;
            self.left_release_target = None;
            self.right_release_target = None;
            self.pelvis_shift = SignedLength::ZERO;
            self.measured_owner_planar_speed = None;
            reset_terminal_settle_reach(self);
            self.recent_movement_velocity = Vec3::ZERO;
            self.settle = None;
        }
        self.owner_frame = Some(frame);
        Ok(())
    }

    /// Advance the shared root correction before publishing the retained offset.
    pub(super) fn advance_terrain_pelvis(
        &mut self,
        desired: SignedLength,
        terminal: bool,
        first_sample: bool,
        gait: LocomotionGait,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<SignedLength> {
        if terminal {
            if let Some(reach) = self.terminal_contacts.prepared_mut() {
                let target = match reach.target_shift {
                    Some(target) => target,
                    None => SignedLength::from_metres(desired.metres().clamp(-0.25, 0.0)).map_err(
                        |_| AnimationValueError::Overflow {
                            quantity: AnimationQuantity::HeightOffset,
                        },
                    )?,
                };
                let shift = if elapsed.is_zero() {
                    reach.shift
                } else {
                    advance_pelvis_shift(reach.shift, target, elapsed)?
                };
                reach.target_shift = Some(target);
                reach.shift = shift;
            }
            Ok(self.terminal_contacts.shift())
        } else {
            let shift = if first_sample {
                desired
            } else if elapsed.is_zero() {
                self.pelvis_shift
            } else if gait == LocomotionGait::Run {
                advance_scalar_at_speed(
                    self.pelvis_shift,
                    desired,
                    elapsed,
                    ik_tuning().run_pelvis_correction_speed_metres_per_second,
                )?
            } else {
                advance_pelvis_shift(self.pelvis_shift, desired, elapsed)?
            };
            self.pelvis_shift = shift;
            Ok(shift)
        }
    }

    /// Final native point results must be finite before component publication.
    /// Without an auxiliary observation, current velocity alone selects the motion budget.
    pub(super) fn effective_planar_speed(&self, current: NonnegativeSpeed) -> NonnegativeSpeed {
        self.measured_owner_planar_speed
            .map_or(current, |measured| current.max(measured))
    }

    /// A retained plant carries support only with its current contact confidence.
    pub(super) fn support_was_acquired(&self, foot: LeadFoot) -> bool {
        if foot == LeadFoot::Left {
            self.left_foot_plant.is_acquired()
                && self
                    .left_support_weight
                    .is_some_and(terrain_leg_has_support)
        } else {
            self.right_foot_plant.is_acquired()
                && self
                    .right_support_weight
                    .is_some_and(terrain_leg_has_support)
        }
    }

    /// Retain deceleration momentum and admit the existing stationary balance-capture seam.
    pub(super) fn observe_movement_for_settle(
        &mut self,
        ordinary_lowered: bool,
        raised_footwork_handoff: bool,
        speed: NonnegativeSpeed,
        planar_velocity: Vec3,
    ) -> bool {
        if ordinary_lowered && speed > STATIONARY_ANIMATION_SPEED_LIMIT && self.settle.is_none() {
            // Retain the strongest recent velocity through presentation deceleration,
            // rather than letting its final sub-threshold sample erase momentum.
            if planar_velocity.length_squared()
                >= self.recent_movement_velocity.length_squared() * 0.25
            {
                self.recent_movement_velocity = planar_velocity;
            }
            false
        } else {
            (ordinary_lowered || raised_footwork_handoff)
                && speed <= STATIONARY_ANIMATION_SPEED_LIMIT
                && self.settle.is_none()
        }
    }

    pub(super) fn finish_terrain_release(&mut self) {
        // Once the bounded release finishes, clear leg targets so a later
        // re-enable cannot resurrect stale plants. Arm pole continuity is
        // unrelated.
        self.left_foot_plant = FootPlant::Absent;
        self.right_foot_plant = FootPlant::Absent;
        self.left_foot_target = None;
        self.right_foot_target = None;
        self.left_foot_world_target = None;
        self.right_foot_world_target = None;
        self.left_last_rendered_world = None;
        self.right_last_rendered_world = None;
        self.left_last_rendered_toe_world = None;
        self.right_last_rendered_toe_world = None;
        self.left_last_rendered_owner = None;
        self.right_last_rendered_owner = None;
        self.left_last_rendered_foot_rotation_world = None;
        self.right_last_rendered_foot_rotation_world = None;
        self.left_authored_world_target = None;
        self.right_authored_world_target = None;
        self.left_planned_contact_start = None;
        self.right_planned_contact_start = None;
        self.left_planned_contact_phase_start = None;
        self.right_planned_contact_phase_start = None;
        self.left_support_weight = None;
        self.right_support_weight = None;
        self.left_transition_support_weight = None;
        self.right_transition_support_weight = None;
        self.left_support_exhausted_until_flight = false;
        self.right_support_exhausted_until_flight = false;
        self.left_terrain_pole_world = None;
        self.right_terrain_pole_world = None;
        self.left_terrain_end_direction = None;
        self.right_terrain_end_direction = None;
        self.left_foot_orientation_world = None;
        self.right_foot_orientation_world = None;
        self.left_contact_orientation_blend_active = false;
        self.right_contact_orientation_blend_active = false;
        clear_slope_rotation_cache(self);
        self.left_release_active = false;
        self.right_release_active = false;
        self.left_release_target = None;
        self.right_release_target = None;
        self.pelvis_shift = SignedLength::ZERO;
        self.measured_owner_planar_speed = None;
        reset_terminal_settle_reach(self);
        self.recent_movement_velocity = Vec3::ZERO;
        self.settle = None;
    }
}

impl LegIkState {
    pub(crate) fn diagnostics(&self) -> LegIkDiagnostics {
        let settle = self.0.settle;

        LegIkDiagnostics {
            left_authored_target: self.0.left_authored_world_target,
            right_authored_target: self.0.right_authored_world_target,
            left_planned_contact: settle
                .filter(|state| state.support_foot == LeadFoot::Right)
                .map(|state| state.landing_target)
                .or(self.0.left_planned_contact),
            right_planned_contact: settle
                .filter(|state| state.support_foot == LeadFoot::Left)
                .map(|state| state.landing_target)
                .or(self.0.right_planned_contact),
            settle_capture_point: settle.map(|state| state.capture_point),
            left_solve_target: self.0.left_foot_world_target,
            right_solve_target: self.0.right_foot_world_target,
            left_support_weight: self.0.left_support_weight,
            right_support_weight: self.0.right_support_weight,
            left_release_active: self.0.left_release_active,
            right_release_active: self.0.right_release_active,
            left_release_target: self.0.left_release_target.and_then(|target| {
                Some(self.0.owner_frame?.origin() + self.0.owner_frame?.rotation() * target)
            }),
            right_release_target: self.0.right_release_target.and_then(|target| {
                Some(self.0.owner_frame?.origin() + self.0.owner_frame?.rotation() * target)
            }),
            settle_progress: settle.map(|state| state.progress),
            left_knee_foot_yaw_offset_degrees: self.0.left_knee_foot_yaw_offset_degrees,
            right_knee_foot_yaw_offset_degrees: self.0.right_knee_foot_yaw_offset_degrees,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct ArmIkState(pub(super) ArmIkMemory);
