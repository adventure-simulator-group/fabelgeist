use super::*;

use attack::{attack_samples, guard_pose};
use locomotion::{
    combat_lower_body_samples, downed_animation_speed, gait_or_idle, quickstep_lower_body_samples,
    upright_locomotion_samples,
};

mod attack;
pub(super) mod locomotion;

/// The dive action reaches authored ground contact halfway through its timeline.
pub const DIVE_GROUND_CONTACT_PHASE: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoseSample {
    pub pose: SemanticPose,
    pub sampling: PoseSampling,
    pub weight: BlendWeight,
    /// Selects the pre-mirrored gait clip for this complete anchor pose.
    /// Mirroring is binary because fractional reflection after FK blending
    /// collapses the bilateral limbs' forward/back separation.
    pub mirror_lower_body: ClipMirror,
}

/// Client-side blend coordinates derived from authoritative state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationEvaluation {
    pub base: Vec<PoseSample>,
    /// Optional locomotion layer evaluated only on the pelvis and legs. This
    /// lets a raised upper-body guard retain an ordinary walk/run blend.
    pub lower_body: Vec<PoseSample>,
    pub action: Vec<PoseSample>,
    pub movement_speed: f32,
    pub gait_phase: CyclePhase,
    pub airborne_phase: f32,
    pub action_phase: f32,
    pub attack_target_height: f32,
}

/// One weighted authored pose contributing to the FK result.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PoseSampling {
    /// Sample the pose's authoritative catalog frame.
    Anchor,
    /// Sample the complete authored motion at a normalized timeline phase.
    /// The graph still owns this sample's blend weight; the runtime only
    /// converts phase to the motion's authored timeline.
    Cycle { phase: CyclePhase },
    /// Sample a finite authored motion from its first through final frame.
    /// Progress one holds the final frame rather than wrapping to the start.
    Timeline { progress: f32 },
    /// Blend two semantic anchor poses. The client samples both catalog frames
    /// exactly and never evaluates exported in-between keys.
    /// Both endpoints declare playback topology, including at zero or complete
    /// progress; an endpoint with zero weight contributes no pose movement.
    Span { end: SemanticPose, progress: f32 },
    /// Extrapolate the transform delta between two semantic anchors. Unlike a
    /// normal span, the coordinate is intentionally allowed outside zero to
    /// one, within the bounds enforced by [`AttackCurve`].
    CurveSpan { end: SemanticPose, coordinate: f32 },
    /// Carry an extrapolated attack tangent through the full-backswing
    /// waypoint and every authored follow-up anchor with one C2-continuous
    /// path, ending at the next guard with zero velocity.
    ContinuationSpan {
        contact: SemanticPose,
        end: SemanticPose,
        outgoing: SemanticPose,
        finish: SemanticPose,
        start_coordinate: f32,
        incoming_tangent: f32,
        /// Readiness knot strictly between zero and contact at phase 0.5,
        /// supplied by the authoritative continuation curve.
        ready_phase: f32,
        progress: f32,
    },
}

impl AnimationEvaluation {
    /// Evaluates semantic FK inputs. Clip resolution, bone masks, IK, and the
    /// final procedural pass happen after this deterministic selection stage.
    pub fn from_skeleton(state: &SkeletonState) -> AnimationValueResult<Self> {
        let speed = state.animation_speed();
        let gait_phase = CyclePhase::from_cycles(state.gait_phase)?.wrapped();
        let base = match state.action_kind() {
            SkeletonAction::Dodge => raised_guard_locomotion_samples(state),
            _ => match state.posture() {
                Posture::Prone => gait_or_idle(
                    downed_animation_speed(state),
                    gait_phase,
                    SemanticPose::ProneIdle,
                    SemanticPose::ProneCrawlContact,
                )?,
                Posture::Supine => gait_or_idle(
                    downed_animation_speed(state),
                    gait_phase,
                    SemanticPose::SupineIdle,
                    SemanticPose::SupineScamperContact,
                )?,
                Posture::Airborne => vec![airborne_sample(state.local_velocity.xz().length())],
                Posture::Ragdolled => Vec::new(),
                Posture::Upright if state.weapon_guard() == WeaponGuardState::Raised => {
                    raised_guard_locomotion_samples(state)
                }
                Posture::Upright => upright_locomotion_samples(state)?,
            },
        };
        let lower_body = if let Some(transition) = state.posture_transition()
            && let PostureTransitionKind::DiveToDowned { direction, .. } = transition.kind()
        {
            dive_lower_body_samples(state.lead_foot, direction, transition.phase())
        } else if state.is_quickstep() {
            quickstep_lower_body_samples(state)?
        } else if state.posture() == Posture::Upright
            && state.weapon_guard() == WeaponGuardState::Raised
        {
            combat_lower_body_samples(state)?
        } else {
            Vec::new()
        };
        let action = posture_transition_samples(state)
            .or_else(|| downed_facing_samples(state))
            .unwrap_or_else(|| action_samples(state));
        Ok(Self {
            base,
            lower_body,
            action,
            movement_speed: speed,
            gait_phase,
            airborne_phase: (0.5 - state.local_velocity.y * 0.2).clamp(0.0, 1.0),
            action_phase: state.action_phase().clamp(0.0, 1.0),
            attack_target_height: state.attack_target_height().clamp(0.0, 1.0),
        })
    }
}

fn posture_transition_samples(state: &SkeletonState) -> Option<Vec<PoseSample>> {
    let transition = state.posture_transition()?;
    use PostureTransitionKind::*;
    if let DiveToDowned { direction, .. } = transition.kind() {
        return Some(dive_transition_samples(
            state.lead_foot,
            direction,
            transition.phase(),
        ));
    }
    let (start, middle, end) = match transition.kind() {
        UprightToProne => (
            SemanticPose::IdleRelaxed,
            SemanticPose::ProneTransition,
            SemanticPose::ProneIdle,
        ),
        ProneToUpright => (
            SemanticPose::ProneIdle,
            SemanticPose::ProneTransition,
            SemanticPose::IdleRelaxed,
        ),
        ProneToSupine { direction } => (
            SemanticPose::ProneIdle,
            roll_pose(direction),
            SemanticPose::SupineIdle,
        ),
        SupineToProne { direction } => (
            SemanticPose::SupineIdle,
            // Reversing a prone-to-supine roll reverses its travel too. Pick
            // the opposite authored side so input direction remains spatial.
            roll_pose(direction.opposite()),
            SemanticPose::ProneIdle,
        ),
        SupineToUpright => (
            SemanticPose::SupineIdle,
            SemanticPose::SupineTransition,
            SemanticPose::IdleRelaxed,
        ),
        DiveToDowned { .. } => unreachable!("dive transitions return above"),
    };
    let phase = transition.phase().clamp(0.0, 1.0);
    let (pose, next, progress) = if phase < 0.5 {
        (start, middle, phase * 2.0)
    } else {
        (middle, end, (phase - 0.5) * 2.0)
    };
    Some(vec![PoseSample {
        pose,
        sampling: PoseSampling::Span {
            end: next,
            progress,
        },
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }])
}

fn downed_facing_samples(state: &SkeletonState) -> Option<Vec<PoseSample>> {
    let roll = state.downed_facing()?.half_turns();
    // Whole values are already represented by the body's prone/supine base
    // pose. Leaving a zero-progress roll action active here would mask the
    // crawl/scamper base layer for as long as aim remains held.
    if (roll - roll.round()).abs() <= 1.0e-4 {
        return None;
    }
    let start_index = roll.floor() as i64;
    let progress = roll - roll.floor();
    let start_prone = start_index.rem_euclid(2) == 0;
    let (start, end) = if start_prone {
        (SemanticPose::ProneIdle, SemanticPose::SupineIdle)
    } else {
        (SemanticPose::SupineIdle, SemanticPose::ProneIdle)
    };
    let middle = if start_index.rem_euclid(2) == 0 {
        SemanticPose::ProneSupineRollRight
    } else {
        SemanticPose::ProneSupineRollLeft
    };
    let (pose, next, span) = if progress < 0.5 {
        (start, middle, progress * 2.0)
    } else {
        (middle, end, (progress - 0.5) * 2.0)
    };
    Some(vec![PoseSample {
        pose,
        sampling: PoseSampling::Span {
            end: next,
            progress: span,
        },
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }])
}

fn dive_transition_samples(
    _lead: LeadFoot,
    direction: DiveDirection,
    phase: f32,
) -> Vec<PoseSample> {
    let dive = dive_pose(direction);
    let phase = phase.clamp(0.0, 1.0);
    if phase < DIVE_GROUND_CONTACT_PHASE {
        return vec![PoseSample {
            pose: SemanticPose::GuardThrust,
            sampling: PoseSampling::Span {
                end: dive,
                progress: phase * 2.0,
            },
            weight: BlendWeight::ONE,
            mirror_lower_body: ClipMirror::Original,
        }];
    }
    let contact = match direction {
        DiveDirection::Forward => SemanticPose::ProneIdle,
        DiveDirection::Backward => SemanticPose::SupineIdle,
        DiveDirection::Left => SemanticPose::ProneSupineRollLeft,
        DiveDirection::Right => SemanticPose::ProneSupineRollRight,
    };
    // Contact begins and ends at zero blend velocity, matching the procedural
    // pelvis release and the authoritative directional root handoff. Starting
    // this span linearly made the complete upper-body chain visibly twist on
    // the first grounded frame, including on forward dives with no root yaw.
    let recovery = dive_contact_pose_progress(phase);
    vec![PoseSample {
        pose: dive,
        sampling: PoseSampling::Span {
            end: contact,
            progress: recovery,
        },
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }]
}

/// The dive clip contributes only above the pelvis. The lower body holds the
/// guard through takeoff, then blends into the appropriate ground contact.
fn dive_lower_body_samples(
    _lead: LeadFoot,
    direction: DiveDirection,
    phase: f32,
) -> Vec<PoseSample> {
    let phase = phase.clamp(0.0, 1.0);
    let sampling = if phase < DIVE_GROUND_CONTACT_PHASE {
        PoseSampling::Anchor
    } else {
        let contact = match direction {
            DiveDirection::Forward => SemanticPose::ProneIdle,
            DiveDirection::Backward => SemanticPose::SupineIdle,
            DiveDirection::Left => SemanticPose::ProneSupineRollLeft,
            DiveDirection::Right => SemanticPose::ProneSupineRollRight,
        };
        PoseSampling::Span {
            end: contact,
            progress: dive_contact_pose_progress(phase),
        }
    };
    vec![PoseSample {
        pose: SemanticPose::GuardThrust,
        sampling,
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }]
}

fn dive_contact_pose_progress(phase: f32) -> f32 {
    smoothstep01((phase - DIVE_GROUND_CONTACT_PHASE) * 2.0)
}

fn dive_pose(direction: DiveDirection) -> SemanticPose {
    match direction {
        DiveDirection::Forward => SemanticPose::DiveForward,
        DiveDirection::Backward => SemanticPose::DiveBackward,
        DiveDirection::Left => SemanticPose::DiveLeft,
        DiveDirection::Right => SemanticPose::DiveRight,
    }
}

fn roll_pose(direction: RollDirection) -> SemanticPose {
    match direction {
        RollDirection::Left => SemanticPose::ProneSupineRollLeft,
        RollDirection::Right => SemanticPose::ProneSupineRollRight,
    }
}

fn raised_guard_locomotion_samples(state: &SkeletonState) -> Vec<PoseSample> {
    let preparation = state.attack_preparation();
    let sampling = if preparation.from == preparation.to {
        PoseSampling::Anchor
    } else {
        PoseSampling::Span {
            end: guard_pose(preparation.to),
            progress: preparation.progress,
        }
    };
    vec![PoseSample {
        pose: guard_pose(preparation.from),
        sampling,
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }]
}

fn smoothstep01(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn airborne_sample(horizontal_speed: f32) -> PoseSample {
    PoseSample {
        pose: SemanticPose::AirborneCenter,
        sampling: PoseSampling::Span {
            end: SemanticPose::AirborneTravel,
            progress: smoothstep01(horizontal_speed / walk_locomotion_profile().reference_speed),
        },
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }
}

fn action_samples(state: &SkeletonState) -> Vec<PoseSample> {
    match state.action_kind() {
        SkeletonAction::None => Vec::new(),
        SkeletonAction::Dodge => Vec::new(),
        SkeletonAction::Attack => attack_samples(state),
        SkeletonAction::Block => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dive_blends_from_guard_to_directional_airborne_pose() {
        let loading = dive_transition_samples(LeadFoot::Left, DiveDirection::Right, 0.1);
        assert_eq!(loading[0].pose, SemanticPose::GuardThrust);
        assert_eq!(
            loading[0].sampling,
            PoseSampling::Span {
                end: SemanticPose::DiveRight,
                progress: 0.2,
            }
        );

        let contact = dive_transition_samples(LeadFoot::Left, DiveDirection::Right, 0.5);
        assert_eq!(contact[0].pose, SemanticPose::DiveRight);
        assert_eq!(
            contact[0].sampling,
            PoseSampling::Span {
                end: SemanticPose::ProneSupineRollRight,
                progress: 0.0,
            }
        );

        let recovery = dive_transition_samples(LeadFoot::Left, DiveDirection::Right, 0.75);
        assert_eq!(recovery.len(), 1);
        assert_eq!(recovery[0].pose, SemanticPose::DiveRight);
        assert_eq!(
            recovery[0].sampling,
            PoseSampling::Span {
                end: SemanticPose::ProneSupineRollRight,
                progress: 0.5,
            }
        );
    }

    #[test]
    fn lateral_dives_recover_directly_into_the_matching_half_roll() {
        for (direction, contact) in [
            (DiveDirection::Left, SemanticPose::ProneSupineRollLeft),
            (DiveDirection::Right, SemanticPose::ProneSupineRollRight),
        ] {
            let sample = dive_transition_samples(LeadFoot::Left, direction, 1.0);
            assert_eq!(sample[0].pose, dive_pose(direction));
            assert_eq!(
                sample[0].sampling,
                PoseSampling::Span {
                    end: contact,
                    progress: 1.0,
                }
            );
        }
    }

    #[test]
    fn dive_lower_body_holds_guard_until_ground_contact() {
        let contact = dive_lower_body_samples(LeadFoot::Left, DiveDirection::Backward, 0.5);
        assert_eq!(contact[0].pose, SemanticPose::GuardThrust);
        assert_eq!(
            contact[0].sampling,
            PoseSampling::Span {
                end: SemanticPose::SupineIdle,
                progress: 0.0,
            }
        );

        let recovery = dive_lower_body_samples(LeadFoot::Left, DiveDirection::Backward, 0.75);
        assert_eq!(recovery[0].pose, SemanticPose::GuardThrust);
        assert_eq!(
            recovery[0].sampling,
            PoseSampling::Span {
                end: SemanticPose::SupineIdle,
                progress: 0.5,
            }
        );
    }

    #[test]
    fn dive_contact_recovery_starts_with_zero_slope() {
        let sample = dive_transition_samples(LeadFoot::Left, DiveDirection::Forward, 0.525);
        let PoseSampling::Span { progress, .. } = sample[0].sampling else {
            panic!("dive recovery must remain an authored pose span");
        };
        assert!((progress - 0.00725).abs() < 0.000_01);

        let lower = dive_lower_body_samples(LeadFoot::Left, DiveDirection::Forward, 0.525);
        let PoseSampling::Span { progress, .. } = lower[0].sampling else {
            panic!("dive lower-body recovery must remain an authored pose span");
        };
        assert!((progress - 0.00725).abs() < 0.000_01);
    }

    #[test]
    fn forward_dive_airborne_pose_is_independent_of_guard_lead() {
        let sample = dive_transition_samples(LeadFoot::Right, DiveDirection::Forward, 0.6);
        assert_eq!(sample[0].pose, SemanticPose::DiveForward);
    }

    #[test]
    fn diagonal_quickstep_blends_directional_authored_timelines() {
        let mut state = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        state
            .begin_dodge(DodgeSpec::quickstep(Vec2::new(1.0, 1.0)).unwrap(), 0, 100)
            .unwrap();
        state.advance_action(150);
        let samples = quickstep_lower_body_samples(&state).unwrap();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].pose, SemanticPose::QuickstepForwardTakeoff);
        assert_eq!(samples[0].weight.factor(), 0.5);
        assert_eq!(samples[1].pose, SemanticPose::QuickstepRightTakeoff);
        assert_eq!(samples[1].weight.factor(), 0.5);
        assert!(samples.iter().all(|sample| {
            matches!(sample.sampling, PoseSampling::Timeline { progress: 0.75 })
        }));
    }

    #[test]
    fn attacks_keep_authored_weapon_upper_body_and_combat_lower_body_separate() {
        let mut state = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        state.begin_attack(AttackSpec::default(), 0, 100).unwrap();
        state.advance_action(50);
        let evaluation = AnimationEvaluation::from_skeleton(&state).unwrap();
        assert_eq!(evaluation.action[0].pose, SemanticPose::GuardThrust);
        assert_eq!(evaluation.lower_body[0].pose, SemanticPose::CombatStance);
    }
}
