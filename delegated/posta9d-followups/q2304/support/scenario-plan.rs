//! Deterministic animation-review fixtures and scenario classification.

use super::*;
use admission::{authored_camera_angle, authored_frame_time};
pub(super) use admission::{authored_drive_speed, canonical_john_sprint_speed};
use adventuresim_building_generator::spatial_geometry::Radians;
pub(super) use quickstep::{
    quickstep_action_ticks, quickstep_fixture_vertical_state, quickstep_landing_frame,
    quickstep_release_frame, quickstep_scenario,
};

mod admission;
mod plan_families;
mod quickstep;

pub(super) mod terrain_policy;
pub(super) const QUICKSTEP_FIXTURE_LEG_STRENGTH: f32 = 4.0;
// The canonical default inventory currently projects 24.9 kg into tactical
// play: one catalog unit weight for each durable inventory row.
pub(super) const JOHN_CURRENT_TOTAL_BURDEN_KG: f32 = 94.9;

#[derive(Debug, Clone)]
pub(super) struct PlannedFrame {
    pub(super) scenario: ScenarioMetadata<'static>,
    pub(super) scenario_frame: usize,
    pub(super) speed: SignedSpeed,
    pub(super) time_seconds: AnimationElapsed,
    pub(super) local_direction: Vec2,
    pub(super) camera_yaw: Radians,
    pub(super) camera_pitch: Radians,
    pub(super) action: SkeletonAction,
    pub(super) weapon_guard: WeaponGuardState,
    pub(super) lead_foot: LeadFoot,
}

pub(super) fn steady_scenario(
    name: ScenarioMetadata<'static>,
    speed: SignedSpeed,
    cycles: f32,
) -> AnimationValueResult<Vec<PlannedFrame>> {
    steady_scenario_in_direction(name, speed, cycles, Vec2::NEG_Y)
}

pub(super) fn full_ragdoll_scenario() -> Vec<PlannedFrame> {
    // The first logical sample receives the viewer's sixty-frame load/settle
    // window, so nine captured samples already cover a stable physics result.
    (0..=8)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::FullRagdoll),
            scenario_frame,
            speed: authored_drive_speed(0.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::ZERO,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn steady_scenario_in_direction(
    name: ScenarioMetadata<'static>,
    speed: SignedSpeed,
    cycles: f32,
    local_direction: Vec2,
) -> AnimationValueResult<Vec<PlannedFrame>> {
    let cycle_duration = ordinary_step_distance(speed.nonnegative_part())?.metres() * 2.0
        / speed.metres_per_second();
    let duration = cycles * cycle_duration;
    // Include the first authoritative tick after the requested final cycle.
    // Fixed-rate sampling rarely lands on the mathematical wrap exactly; the
    // post-wrap sample makes every steady scenario exercise its real loop
    // transition instead of silently reporting no seam.
    let last_frame = (duration * locomotion_sample_rate().hertz()).ceil() as usize + 1;
    Ok((0..=last_frame)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed,
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect())
}

pub(super) fn transition_scenario() -> Vec<PlannedFrame> {
    let duration = 4.0;
    let last_frame = (duration * locomotion_sample_rate().hertz()) as usize;
    (0..=last_frame)
        .map(|frame| {
            let t = frame as f32 / locomotion_sample_rate().hertz();
            let speed = if t < 0.5 {
                2.0 * smoothstep01(t / 0.5)
            } else if t < 1.0 {
                2.0
            } else if t < 1.75 {
                2.0 + 3.5 * smoothstep01((t - 1.0) / 0.75)
            } else if t < 2.5 {
                5.5
            } else if t < 3.5 {
                5.5 * (1.0 - smoothstep01((t - 2.5) / 1.0))
            } else {
                0.0
            };
            PlannedFrame {
                scenario: ScenarioMetadata::authored(ScenarioId::StartStopTransition),
                scenario_frame: frame,
                speed: authored_drive_speed(speed),
                time_seconds: authored_frame_time(t),
                local_direction: Vec2::NEG_Y,
                camera_yaw: authored_camera_angle(0.0),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn terrain_toggle_scenario() -> Vec<PlannedFrame> {
    (0..=112)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::TerrainToggleMidStride),
            scenario_frame,
            speed: authored_drive_speed(2.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::X,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn terrain_half_turn_reversal_scenario() -> Vec<PlannedFrame> {
    (0..=192)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::TerrainHalfTurnReversal),
            scenario_frame,
            speed: authored_drive_speed(2.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::NEG_Y,
            camera_yaw: authored_camera_angle(if scenario_frame >= 128 {
                std::f32::consts::PI
            } else {
                0.0
            }),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn dynamics_speed_scenario(
    name: ScenarioMetadata<'static>,
    hard_stop: bool,
) -> Vec<PlannedFrame> {
    (0..=256)
        .map(|scenario_frame| {
            let speed = if hard_stop {
                if scenario_frame < 96 { 5.5 } else { 0.0 }
            } else if scenario_frame < 32 {
                5.5 * scenario_frame as f32 / 32.0
            } else if scenario_frame < 128 {
                5.5
            } else if scenario_frame < 160 {
                5.5 * (160 - scenario_frame) as f32 / 32.0
            } else {
                0.0
            };
            PlannedFrame {
                scenario: name.clone(),
                scenario_frame,
                speed: authored_drive_speed(speed),
                time_seconds: authored_frame_time(
                    scenario_frame as f32 / locomotion_sample_rate().hertz(),
                ),
                local_direction: Vec2::NEG_Y,
                camera_yaw: authored_camera_angle(0.0),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn flat_grid_walk_stop_scenario() -> Vec<PlannedFrame> {
    (0..=96)
        .map(|scenario_frame| {
            let speed = if scenario_frame < 48 {
                2.0
            } else if scenario_frame < 56 {
                2.0 * (56 - scenario_frame) as f32 / 8.0
            } else {
                0.0
            };
            PlannedFrame {
                scenario: ScenarioMetadata::authored(ScenarioId::FlatGridWalkStop),
                scenario_frame,
                speed: authored_drive_speed(speed),
                time_seconds: authored_frame_time(
                    scenario_frame as f32 / locomotion_sample_rate().hertz(),
                ),
                local_direction: Vec2::NEG_Y,
                camera_yaw: authored_camera_angle(0.0),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn terrain_tap_stop_scenario(
    name: ScenarioMetadata<'static>,
    speed: SignedSpeed,
    moving_frames: std::ops::Range<usize>,
    local_direction: Vec2,
) -> Vec<PlannedFrame> {
    (0..=96)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: if moving_frames.contains(&scenario_frame) {
                speed
            } else {
                SignedSpeed::ZERO
            },
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn terrain_tap_restart_scenario() -> Vec<PlannedFrame> {
    (0..=96)
        .map(|scenario_frame| {
            let moving = matches!(scenario_frame, 8..=17 | 24..=33 | 40..=49);
            PlannedFrame {
                scenario: ScenarioMetadata::authored(ScenarioId::TerrainTapRestartCrossfade),
                scenario_frame,
                speed: authored_drive_speed(if moving { 5.5 } else { 0.0 }),
                time_seconds: authored_frame_time(
                    scenario_frame as f32 / locomotion_sample_rate().hertz(),
                ),
                local_direction: Vec2::NEG_Y,
                camera_yaw: authored_camera_angle(0.0),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn terrain_threshold_chatter_scenario() -> Vec<PlannedFrame> {
    (0..=128)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::TerrainSpeedThresholdChatter),
            scenario_frame,
            speed: authored_drive_speed(match scenario_frame {
                8..=39 => {
                    if scenario_frame % 2 == 0 {
                        0.079
                    } else {
                        0.081
                    }
                }
                40 => 0.09,
                41..=71 => {
                    if scenario_frame % 2 == 0 {
                        0.029
                    } else {
                        0.031
                    }
                }
                72 => 0.02,
                73..=103 => {
                    if scenario_frame % 2 == 0 {
                        0.079
                    } else {
                        0.081
                    }
                }
                _ => 0.0,
            }),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::NEG_Y,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn raised_guard_lateral_tap_stop_scenario(
    name: ScenarioMetadata<'static>,
    direction: Vec2,
) -> Vec<PlannedFrame> {
    (0..=96)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: authored_drive_speed(if (32..38).contains(&scenario_frame) {
                1.0
            } else {
                0.0
            }),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: direction,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Raised,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn airborne_landing_scenario() -> Vec<PlannedFrame> {
    (0..=96)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::AirborneLanding),
            scenario_frame,
            speed: authored_drive_speed(0.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::ZERO,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn smoothstep01(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub(super) fn attack_live_scenario(
    name: ScenarioMetadata<'static>,
    speed: SignedSpeed,
    initial_direction: Vec2,
    lead_foot: LeadFoot,
    reverse_velocity: bool,
) -> Vec<PlannedFrame> {
    const START: usize = 8;
    // Keep sampling after the authored action ends so procedural recovery can
    // take as many bounded guard-convergence steps as it needs.
    (0..=127)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed,
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            // Deliberately reverse velocity and yaw after attack start in the
            // stress fixture. The live movement input must remain the one
            // selected on frame zero.
            local_direction: if (reverse_velocity || name.has(ScenarioFeature::HighSpeed))
                && scenario_frame > START
            {
                -initial_direction
            } else {
                initial_direction
            },
            camera_yaw: authored_camera_angle(
                if name.has(ScenarioFeature::YawOnly) && scenario_frame > START {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                },
            ),
            camera_pitch: authored_camera_angle(0.0),
            action: if scenario_frame < START {
                SkeletonAction::None
            } else {
                SkeletonAction::Attack
            },
            weapon_guard: WeaponGuardState::Raised,
            lead_foot,
        })
        .collect()
}

pub(super) fn capture_plan() -> AnimationValueResult<Vec<PlannedFrame>> {
    Ok([
        downed_contact_scenario(
            ScenarioMetadata::authored(ScenarioId::DownedProneCrawl),
            BodyState::Prone,
        ),
        downed_contact_scenario(
            ScenarioMetadata::authored(ScenarioId::DownedSupineScamper),
            BodyState::Supine,
        ),
        downed_look_scenario(),
        ordinary_camera_pitch_scenario(),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::DiveForward),
            BodyState::Grounded(GroundedPosture::Upright),
        ),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::DiveBackward),
            BodyState::Grounded(GroundedPosture::Upright),
        ),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::DiveLeft),
            BodyState::Grounded(GroundedPosture::Upright),
        ),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::DiveRight),
            BodyState::Grounded(GroundedPosture::Upright),
        ),
        dive_impact_scenario(ScenarioMetadata::authored(ScenarioId::DiveLeftImpact)),
        dive_impact_scenario(ScenarioMetadata::authored(ScenarioId::DiveRightImpact)),
        dive_impact_scenario(ScenarioMetadata::authored(ScenarioId::DiveBackwardImpact)),
        aimed_dive_impact_scenario(ScenarioMetadata::authored(
            ScenarioId::DiveForwardAimedImpact,
        )),
        aimed_dive_impact_scenario(ScenarioMetadata::authored(ScenarioId::DiveLeftAimedImpact)),
        aimed_dive_impact_scenario(ScenarioMetadata::authored(ScenarioId::DiveRightAimedImpact)),
        aimed_dive_impact_scenario(ScenarioMetadata::authored(
            ScenarioId::DiveBackwardAimedImpact,
        )),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::ProneGetUp),
            BodyState::Prone,
        ),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::SupineGetUp),
            BodyState::Supine,
        ),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::ProneRollLeft),
            BodyState::Prone,
        ),
        posture_transition_scenario(
            ScenarioMetadata::authored(ScenarioId::ProneRollRight),
            BodyState::Prone,
        ),
        jump_charge_scenario(),
        quickstep_scenario(
            ScenarioMetadata::authored(ScenarioId::QuickstepRight),
            Vec2::X,
        ),
        quickstep_scenario(
            ScenarioMetadata::authored(ScenarioId::QuickstepLeft),
            Vec2::NEG_X,
        ),
        steady_scenario(
            ScenarioMetadata::authored(ScenarioId::SteadyWalk20),
            authored_drive_speed(2.0),
            2.0,
        )?,
        steady_scenario(
            ScenarioMetadata::authored(ScenarioId::WalkRunBlend375),
            authored_drive_speed(3.75),
            2.0,
        )?,
        steady_scenario(
            ScenarioMetadata::authored(ScenarioId::SteadyRun55),
            authored_drive_speed(5.5),
            2.0,
        )?,
        steady_scenario_in_direction(
            ScenarioMetadata::authored(ScenarioId::LateralWalk20),
            authored_drive_speed(2.0),
            1.0,
            Vec2::X,
        )?,
        steady_scenario_in_direction(
            ScenarioMetadata::authored(ScenarioId::ReverseWalk20),
            authored_drive_speed(2.0),
            1.0,
            Vec2::Y,
        )?,
        turning_scenario(
            ScenarioMetadata::authored(ScenarioId::GradualCameraTurn),
            false,
        ),
        turning_scenario(
            ScenarioMetadata::authored(ScenarioId::HalfTurnReversal),
            true,
        ),
        guard_plant_turn_scenario(),
        plan_families::raised_guard()?,
        plan_families::right_support()?,
        plan_families::attacks(),
        dynamics_speed_scenario(
            ScenarioMetadata::authored(ScenarioId::SpeedRampUpDown),
            false,
        ),
        dynamics_speed_scenario(ScenarioMetadata::authored(ScenarioId::HardStop), true),
        dynamics_turn_scenario(
            ScenarioMetadata::authored(ScenarioId::DynamicsTurn90),
            std::f32::consts::FRAC_PI_2,
        ),
        dynamics_turn_scenario(
            ScenarioMetadata::authored(ScenarioId::DynamicsTurn180),
            std::f32::consts::PI,
        ),
        airborne_landing_scenario(),
        steady_scenario(
            ScenarioMetadata::authored(ScenarioId::CadenceContact),
            authored_drive_speed(2.0),
            2.0,
        )?,
        // Terrain IK captures include a complete calibration cycle plus two
        // review cycles. The old one-cycle probe discarded its only full cycle
        // as warmup and judged a tiny post-wrap tail.
        steady_scenario_in_direction(
            ScenarioMetadata::authored(ScenarioId::CrossSlopeWalk),
            authored_drive_speed(2.0),
            3.0,
            Vec2::X,
        )?,
        steady_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainUphillWalk),
            authored_drive_speed(2.0),
            3.0,
        )?,
        steady_scenario_in_direction(
            ScenarioMetadata::authored(ScenarioId::TerrainDownhillWalk),
            authored_drive_speed(2.0),
            3.0,
            Vec2::Y,
        )?,
        steady_scenario_in_direction(
            ScenarioMetadata::authored(ScenarioId::TerrainDiagonalWalk),
            authored_drive_speed(2.0),
            3.0,
            Vec2::new(1.0, -1.0).normalize(),
        )?,
        terrain_toggle_scenario(),
        dynamics_speed_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainHardStop),
            true,
        ),
        terrain_tap_stop_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainTapStopForward),
            authored_drive_speed(0.8),
            8..20,
            Vec2::NEG_Y,
        ),
        terrain_tap_stop_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainStopMidSwing),
            authored_drive_speed(2.0),
            8..28,
            Vec2::NEG_Y,
        ),
        terrain_tap_stop_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainRunFlightStop),
            authored_drive_speed(5.5),
            8..18,
            Vec2::NEG_Y,
        ),
        terrain_tap_restart_scenario(),
        terrain_threshold_chatter_scenario(),
        steady_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainSteadyRun55),
            authored_drive_speed(5.5),
            3.0,
        )?,
        dynamics_turn_scenario(
            ScenarioMetadata::authored(ScenarioId::TerrainTurn90),
            std::f32::consts::FRAC_PI_2,
        ),
        terrain_half_turn_reversal_scenario(),
        transition_scenario(),
    ]
    .into_iter()
    .flatten()
    .collect())
}

pub(super) fn downed_contact_scenario(
    name: ScenarioMetadata<'static>,
    body: BodyState,
) -> Vec<PlannedFrame> {
    let speed = match body {
        BodyState::Prone => 2.0,
        BodyState::Supine => 0.8,
        _ => 0.0,
    };
    // Include a full review cycle after the pose-buffer startup settles. The
    // shorter probe ended just before the first loop seam, hiding precisely
    // the kind of crawl discontinuity this scenario is meant to diagnose.
    (0..=148)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: authored_drive_speed(speed),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::NEG_Y,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn downed_look_scenario() -> Vec<PlannedFrame> {
    (0..=64)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::DownedProneLookAt),
            scenario_frame,
            speed: authored_drive_speed(0.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::ZERO,
            camera_yaw: authored_camera_angle(std::f32::consts::FRAC_PI_2),
            camera_pitch: authored_camera_angle(0.6),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Raised,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn ordinary_camera_pitch_scenario() -> Vec<PlannedFrame> {
    (0..=64)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::OrdinaryCameraPitch),
            scenario_frame,
            speed: authored_drive_speed(0.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::ZERO,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(if scenario_frame < 16 {
                0.0
            } else if scenario_frame < 40 {
                0.6
            } else {
                -0.6
            }),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn posture_transition_scenario(
    name: ScenarioMetadata<'static>,
    _start: BodyState,
) -> Vec<PlannedFrame> {
    // Stop just before the runtime-owned endpoint handoff. The viewer is
    // validating the authored transition arc; ordinary base-pose captures
    // validate the contact endpoint independently.
    (0..=80)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: authored_drive_speed(0.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::ZERO,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn dive_impact_scenario(name: ScenarioMetadata<'static>) -> Vec<PlannedFrame> {
    dive_impact_scenario_with_aim(name, false)
}

pub(super) fn aimed_dive_impact_scenario(name: ScenarioMetadata<'static>) -> Vec<PlannedFrame> {
    dive_impact_scenario_with_aim(name, true)
}

pub(super) fn dive_impact_scenario_with_aim(
    name: ScenarioMetadata<'static>,
    aimed: bool,
) -> Vec<PlannedFrame> {
    let local_direction = match name.dive_direction() {
        Some(DiveDirection::Forward) => Vec2::NEG_Y,
        Some(DiveDirection::Backward) => Vec2::Y,
        Some(DiveDirection::Left) => Vec2::NEG_X,
        Some(DiveDirection::Right) => Vec2::X,
        None => Vec2::ZERO,
    };
    let final_frame = if name.dive_direction() == Some(DiveDirection::Backward) {
        56
    } else {
        48
    };
    (0..=final_frame)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            // Match the server's camera-relative launch rather than judging a
            // directional pose on a stationary root. Retain velocity through
            // the first terrain-contact sample so travel and body orientation
            // can be compared across the complete airborne arc.
            speed: if scenario_frame <= 17 {
                SignedSpeed::from(TACTICAL_DIVE_HORIZONTAL_SPEED_METRES_PER_SECOND)
            } else {
                SignedSpeed::ZERO
            },
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction,
            camera_yaw: authored_camera_angle(if aimed { 0.85 } else { 0.0 }),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: if aimed {
                WeaponGuardState::Raised
            } else {
                WeaponGuardState::Lowered
            },
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn jump_charge_scenario() -> Vec<PlannedFrame> {
    (0..=64)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::JumpChargeAnticipation),
            scenario_frame,
            speed: authored_drive_speed(0.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::ZERO,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn turning_scenario(
    name: ScenarioMetadata<'static>,
    reversal: bool,
) -> Vec<PlannedFrame> {
    (0..=64)
        .map(|frame| {
            let progress = frame as f32 / 64.0;
            PlannedFrame {
                scenario: name.clone(),
                scenario_frame: frame,
                speed: authored_drive_speed(2.0),
                time_seconds: authored_frame_time(progress),
                local_direction: Vec2::NEG_Y,
                camera_yaw: authored_camera_angle(if reversal && frame > 0 {
                    std::f32::consts::PI
                } else if reversal {
                    0.0
                } else {
                    std::f32::consts::FRAC_PI_2 * progress
                }),
                camera_pitch: authored_camera_angle(0.55 * progress),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn dynamics_turn_scenario(
    name: ScenarioMetadata<'static>,
    angle_radians: f32,
) -> Vec<PlannedFrame> {
    (0..=64)
        .map(|frame| {
            let progress = frame as f32 / 64.0;
            PlannedFrame {
                scenario: name.clone(),
                scenario_frame: frame,
                speed: authored_drive_speed(5.5),
                time_seconds: authored_frame_time(progress),
                local_direction: Vec2::NEG_Y,
                camera_yaw: authored_camera_angle(angle_radians * progress),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn guard_plant_turn_scenario() -> Vec<PlannedFrame> {
    (0..=64)
        .map(|frame| {
            let progress = frame as f32 / 64.0;
            PlannedFrame {
                scenario: ScenarioMetadata::authored(ScenarioId::PlantedGuardTurn),
                scenario_frame: frame,
                speed: authored_drive_speed(0.35),
                time_seconds: authored_frame_time(progress),
                local_direction: Vec2::X,
                camera_yaw: authored_camera_angle(std::f32::consts::FRAC_PI_2 * progress),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::Block,
                weapon_guard: WeaponGuardState::Lowered,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn raised_guard_stationary_turn_scenario() -> Vec<PlannedFrame> {
    (0..=127)
        .map(|scenario_frame| {
            let turn_progress = (scenario_frame as f32 / 64.0).clamp(0.0, 1.0);
            PlannedFrame {
                scenario: ScenarioMetadata::authored(ScenarioId::RaisedGuardStationaryTurn),
                scenario_frame,
                speed: authored_drive_speed(0.0),
                time_seconds: authored_frame_time(
                    scenario_frame as f32 / locomotion_sample_rate().hertz(),
                ),
                local_direction: Vec2::ZERO,
                camera_yaw: authored_camera_angle(
                    std::f32::consts::FRAC_PI_2 * smoothstep01(turn_progress),
                ),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Raised,
                lead_foot: LeadFoot::Left,
            }
        })
        .collect()
}

pub(super) fn raised_guard_scenario(
    name: ScenarioMetadata<'static>,
    direction: Vec2,
) -> AnimationValueResult<Vec<PlannedFrame>> {
    let direction = direction.normalize_or_zero();
    // Leave enough post-startup time for both local support identities to
    // complete. One semantic cycle could end before the client-owned first
    // landing after asset/pose readiness and therefore never test alternation.
    raised_guard_steady_scenario(name, authored_drive_speed(2.0), 2.0, direction)
}

pub(super) fn raised_guard_steady_scenario(
    name: ScenarioMetadata<'static>,
    speed: SignedSpeed,
    cycles: f32,
    direction: Vec2,
) -> AnimationValueResult<Vec<PlannedFrame>> {
    raised_guard_steady_scenario_with_lead(name, speed, cycles, direction, LeadFoot::Left)
}

pub(super) fn raised_guard_steady_scenario_with_lead(
    name: ScenarioMetadata<'static>,
    speed: SignedSpeed,
    cycles: f32,
    direction: Vec2,
    lead_foot: LeadFoot,
) -> AnimationValueResult<Vec<PlannedFrame>> {
    let duration = cycles * guard_step_length()?.metres() * 2.0 / speed.metres_per_second();
    let last_frame = (duration * locomotion_sample_rate().hertz()).ceil() as usize + 1;
    Ok((0..=last_frame)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed,
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: direction.normalize_or_zero(),
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Raised,
            lead_foot,
        })
        .collect())
}

pub(super) fn raised_guard_acceleration_scenario() -> Vec<PlannedFrame> {
    raised_guard_acceleration_scenario_with_lead(
        ScenarioMetadata::authored(ScenarioId::RaisedGuardAccelerateFromRest),
        LeadFoot::Left,
    )
}

pub(super) fn raised_guard_acceleration_scenario_with_lead(
    name: ScenarioMetadata<'static>,
    lead_foot: LeadFoot,
) -> Vec<PlannedFrame> {
    (0..=96)
        .map(|scenario_frame| {
            let time_seconds = scenario_frame as f32 / locomotion_sample_rate().hertz();
            PlannedFrame {
                scenario: name.clone(),
                scenario_frame,
                speed: authored_drive_speed((time_seconds / 0.5).clamp(0.0, 1.0) * 2.0),
                time_seconds: authored_frame_time(time_seconds),
                local_direction: Vec2::X,
                camera_yaw: authored_camera_angle(0.0),
                camera_pitch: authored_camera_angle(0.0),
                action: SkeletonAction::None,
                weapon_guard: WeaponGuardState::Raised,
                lead_foot,
            }
        })
        .collect()
}

pub(super) fn raised_guard_transition_scenario() -> Vec<PlannedFrame> {
    (0..=64)
        .map(|scenario_frame| PlannedFrame {
            scenario: ScenarioMetadata::authored(ScenarioId::RaisedGuardTransition),
            scenario_frame,
            speed: authored_drive_speed(2.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::NEG_Y,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: if scenario_frame < 16 {
                WeaponGuardState::Lowered
            } else {
                WeaponGuardState::Raised
            },
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(super) fn raised_guard_release_scenario() -> AnimationValueResult<Vec<PlannedFrame>> {
    raised_guard_release_scenario_with_lead(
        ScenarioMetadata::authored(ScenarioId::RaisedGuardReleaseAtPeak),
        LeadFoot::Left,
    )
}

pub(super) fn raised_guard_release_scenario_with_lead(
    name: ScenarioMetadata<'static>,
    lead_foot: LeadFoot,
) -> AnimationValueResult<Vec<PlannedFrame>> {
    let release_frame =
        (guard_step_length()?.metres() * 0.75 * locomotion_sample_rate().hertz()).round() as usize;
    Ok((0..=64)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: authored_drive_speed(if scenario_frame <= release_frame {
                2.0
            } else {
                0.0
            }),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: Vec2::NEG_Y,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Raised,
            lead_foot,
        })
        .collect())
}

pub(super) fn raised_guard_reversal_scenario() -> Vec<PlannedFrame> {
    raised_guard_reversal_scenario_with_lead(
        ScenarioMetadata::authored(ScenarioId::RaisedGuardLeftRightReversal),
        LeadFoot::Left,
    )
}

pub(super) fn raised_guard_reversal_scenario_with_lead(
    name: ScenarioMetadata<'static>,
    lead_foot: LeadFoot,
) -> Vec<PlannedFrame> {
    (0..=64)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: authored_drive_speed(2.0),
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction: if scenario_frame < 16 {
                Vec2::NEG_X
            } else {
                Vec2::X
            },
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::None,
            weapon_guard: WeaponGuardState::Raised,
            lead_foot,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_directional_dive_scenario_requires_the_shared_motion() {
        for scenario in [
            "dive-forward",
            "dive-backward-impact",
            "dive-left-aimed-impact",
            "dive-right",
        ] {
            assert_eq!(
                ScenarioMetadata::from_label(scenario)
                    .required_motion()
                    .cloned(),
                Some(crate::animation::MotionId::DIVE)
            );
        }
    }

    #[test]
    fn steady_scenarios_use_authoritative_fixed_tick_samples() {
        for (name, speed) in [
            ("walk", authored_drive_speed(2.0)),
            ("blend", authored_drive_speed(3.75)),
            ("run", authored_drive_speed(5.5)),
        ] {
            let frames =
                steady_scenario(ScenarioMetadata::from_label(name).into_owned(), speed, 2.0)
                    .unwrap();
            assert_eq!(frames.first().unwrap().time_seconds.seconds(), 0.0);
            assert!(frames.len() > 30);
            for pair in frames.windows(2) {
                assert!(pair[1].time_seconds > pair[0].time_seconds);
                assert!(
                    (pair[1].time_seconds.seconds()
                        - pair[0].time_seconds.seconds()
                        - locomotion_sample_rate().interval().seconds())
                    .abs()
                        < 0.0001
                );
            }
        }
    }

    #[test]
    fn raised_tap_stop_adversaries_enable_terrain_ik_without_changing_gate_kind() {
        for (name, direction) in [
            ("raised-guard-tap-stop-left", Vec2::NEG_X),
            ("raised-guard-tap-stop-right", Vec2::X),
        ] {
            let frames = raised_guard_lateral_tap_stop_scenario(
                ScenarioMetadata::from_label(name).into_owned(),
                direction,
            );
            assert_eq!(
                ScenarioMetadata::from_label(name).kind(),
                ScenarioKind::RaisedGuard
            );
            assert!(scenario_uses_terrain_ik(&ScenarioMetadata::from_label(
                name
            )));
            assert!(!raised_scenario_requires_zero_flight(
                &ScenarioMetadata::from_label(name)
            ));
            assert!(
                frames
                    .iter()
                    .all(|frame| terrain_ik_mode_for_frame(frame) == TerrainIkMode::Enabled)
            );
        }
    }

    #[test]
    fn terrain_and_steady_raised_gate_classification_remain_distinct() {
        assert!(scenario_uses_terrain_ik(&ScenarioMetadata::authored(
            ScenarioId::CrossSlopeWalk
        )));
        assert!(!scenario_uses_terrain_ik(&ScenarioMetadata::authored(
            ScenarioId::RaisedGuardForward
        )));
        assert!(!scenario_uses_terrain_ik(&ScenarioMetadata::authored(
            ScenarioId::RaisedGuardStationaryTurn
        )));
        assert!(!scenario_uses_terrain_ik(&ScenarioMetadata::authored(
            ScenarioId::SteadyWalk20
        )));
        assert!(raised_scenario_requires_zero_flight(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardForward)
        ));
        assert!(raised_scenario_requires_zero_flight(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardStationaryTurn)
        ));
        assert!(!raised_scenario_requires_zero_flight(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardTapStopRight)
        ));
        assert!(!raised_scenario_requires_zero_flight(
            &ScenarioMetadata::authored(ScenarioId::CrossSlopeWalk)
        ));

        let stationary_turn = ScenarioMetadata::authored(ScenarioId::RaisedGuardStationaryTurn);
        assert_eq!(stationary_turn.kind(), ScenarioKind::RaisedGuard);
        assert!(!stationary_turn.repeatable());
        assert!(stationary_turn.procedural_solver());
    }

    #[test]
    fn transition_uses_server_stride_formula_without_non_finite_state() {
        let frames = transition_scenario();
        assert_eq!(frames.len(), 257);
        assert!(
            frames
                .iter()
                .all(|frame| frame.speed.metres_per_second().is_finite()
                    && frame.time_seconds.seconds().is_finite()
                    && frame.local_direction.is_finite())
        );
        assert_eq!(frames.first().unwrap().speed, SignedSpeed::ZERO);
        assert_eq!(frames.last().unwrap().speed, SignedSpeed::ZERO);
    }

    #[test]
    fn threshold_stress_hits_both_hysteresis_edges_and_crossings() {
        let speeds = terrain_threshold_chatter_scenario()
            .into_iter()
            .map(|frame| frame.speed)
            .collect::<Vec<_>>();
        for expected in [0.079, 0.081, 0.029, 0.031, 0.02, 0.09] {
            assert!(
                speeds.contains(&authored_drive_speed(expected)),
                "missing threshold sample {expected}"
            );
        }
    }

    #[test]
    fn capture_plan_covers_raised_guard_directions_and_gameplay_transition() {
        let plan = capture_plan().unwrap();
        for scenario in [
            "raised-guard-forward",
            "raised-guard-backward",
            "raised-guard-left",
            "raised-guard-right",
            "raised-guard-forward-left",
            "raised-guard-forward-right",
            "raised-guard-backward-left",
            "raised-guard-backward-right",
        ] {
            assert!(plan.iter().any(|frame| {
                frame.scenario.label() == scenario && frame.weapon_guard == WeaponGuardState::Raised
            }));
        }
        let stationary_turn = plan
            .iter()
            .filter(|frame| frame.scenario.label() == "raised-guard-stationary-turn")
            .collect::<Vec<_>>();
        assert_eq!(stationary_turn.len(), 128);
        assert!(stationary_turn.iter().all(|frame| {
            frame.speed == SignedSpeed::ZERO
                && frame.local_direction == Vec2::ZERO
                && frame.weapon_guard == WeaponGuardState::Raised
        }));
        assert!(stationary_turn.first().unwrap().camera_yaw.radians().abs() < 0.0001);
        assert!(
            (stationary_turn.last().unwrap().camera_yaw.radians() - std::f32::consts::FRAC_PI_2)
                .abs()
                < 0.0001
        );
        let transition = plan
            .iter()
            .filter(|frame| frame.scenario.label() == "raised-guard-transition")
            .collect::<Vec<_>>();
        assert!(
            transition
                .iter()
                .any(|frame| { frame.weapon_guard == WeaponGuardState::Lowered })
        );
        assert!(
            transition
                .iter()
                .any(|frame| { frame.weapon_guard == WeaponGuardState::Raised })
        );
        assert_eq!(
            transition.first().unwrap().weapon_guard,
            WeaponGuardState::Lowered
        );
        assert_eq!(
            transition.last().unwrap().weapon_guard,
            WeaponGuardState::Raised
        );
        for scenario in [
            "raised-guard-right-support-left",
            "raised-guard-right-support-right",
            "raised-guard-right-support-forward-right",
            "raised-guard-right-support-accelerate",
            "raised-guard-right-support-release",
            "raised-guard-right-support-reversal",
        ] {
            assert!(plan.iter().any(|frame| {
                frame.scenario.label() == scenario
                    && frame.weapon_guard == WeaponGuardState::Raised
                    && frame.lead_foot == LeadFoot::Right
            }));
        }
    }

    #[test]
    fn raised_guard_viewer_scenarios_cross_complete_step_cycle_with_fixed_lead() {
        for scenario in [
            "raised-guard-forward",
            "raised-guard-backward",
            "raised-guard-left",
            "raised-guard-right",
            "raised-guard-forward-left",
            "raised-guard-forward-right",
            "raised-guard-backward-left",
            "raised-guard-backward-right",
        ] {
            let mut skeleton = SkeletonState::default();
            set_weapon_guard(&mut skeleton, WeaponGuardState::Raised);
            let mut phases = Vec::new();
            for frame in capture_plan()
                .unwrap()
                .into_iter()
                .filter(|frame| frame.scenario.label() == scenario)
            {
                project_skeleton_locomotion(
                    &mut skeleton,
                    SkeletonLocomotionInput::from_controller(
                        Quat::IDENTITY,
                        Vec3::new(frame.local_direction.x, 0.0, frame.local_direction.y)
                            * frame.speed.metres_per_second(),
                        GroundContact::Grounded,
                        AnimationElapsed::from_seconds(if frame.scenario_frame == 0 {
                            0.0
                        } else {
                            locomotion_sample_rate().interval().seconds()
                        })
                        .unwrap(),
                        AnimationSampleTick::from(frame.scenario_frame as u64),
                    )
                    .unwrap(),
                )
                .unwrap();
                assert_eq!(skeleton.lead_foot, frame.lead_foot);
                let evaluation = AnimationEvaluation::from_skeleton(&skeleton).unwrap();
                assert_eq!(evaluation.base.len(), 1);
                assert_eq!(evaluation.base[0].pose, SemanticPose::GuardThrust);
                assert_eq!(evaluation.base[0].sampling, PoseSampling::Anchor);
                phases.push(skeleton.gait_phase);
            }
            assert!(
                phases
                    .windows(2)
                    .any(|pair| pair[1].cycles() < pair[0].cycles())
            );
            assert!(phases.iter().any(|&phase| phase.cycles() >= 0.5));
        }
    }

    #[test]
    fn raised_guard_viewer_finishes_release_and_updates_reversal_without_phase_reset() {
        let replay = |scenario: &str| {
            let mut skeleton = SkeletonState::default();
            set_weapon_guard(&mut skeleton, WeaponGuardState::Raised);
            let mut samples = Vec::new();
            for frame in capture_plan()
                .unwrap()
                .into_iter()
                .filter(|frame| frame.scenario.label() == scenario)
            {
                project_skeleton_locomotion(
                    &mut skeleton,
                    SkeletonLocomotionInput::from_controller(
                        Quat::IDENTITY,
                        Vec3::new(frame.local_direction.x, 0.0, frame.local_direction.y)
                            * frame.speed.metres_per_second(),
                        GroundContact::Grounded,
                        AnimationElapsed::from_seconds(if frame.scenario_frame == 0 {
                            0.0
                        } else {
                            locomotion_sample_rate().interval().seconds()
                        })
                        .unwrap(),
                        AnimationSampleTick::from(frame.scenario_frame as u64),
                    )
                    .unwrap(),
                )
                .unwrap();
                samples.push((
                    frame.scenario_frame,
                    skeleton.gait_phase,
                    skeleton.raised_locomotion(),
                ));
            }
            samples
        };

        let release = replay("raised-guard-release-at-peak");
        let release_frame =
            (guard_step_length().unwrap().metres() * 0.75 * locomotion_sample_rate().hertz())
                .round() as usize;
        assert!(
            release
                .iter()
                .any(|(frame, phase, intent)| *frame > release_frame
                    && phase.cycles() > 0.5
                    && intent.is_moving())
        );
        assert!(!release.last().unwrap().2.is_moving());
        assert_eq!(release.last().unwrap().1.cycles(), 0.0);

        let reversal = replay("raised-guard-left-right-reversal");
        let changed = reversal
            .iter()
            .find(|(_, _, intent)| intent.local_direction() == Vec2::X)
            .expect("reversal observation is accepted immediately");
        let previous_phase = reversal
            .iter()
            .find(|(frame, _, _)| *frame == 15)
            .expect("pre-reversal sample")
            .1;
        assert_eq!(changed.0, 16);
        let phase_delta = (changed.1.cycles() - previous_phase.cycles()).rem_euclid(1.0);
        // A reversal may make the currently committed contact immediately
        // due and begin the opposite half-step. The authoritative guard plan
        // therefore does not promise constant phase velocity across this
        // tick; it promises forward continuity without skipping more than one
        // contact interval (one half of the normalized cycle).
        const MAXIMUM_REVERSAL_PHASE_ADVANCE: f32 = 0.5;
        assert!(
            phase_delta > 0.0 && phase_delta <= MAXIMUM_REVERSAL_PHASE_ADVANCE + 0.001,
            "reversal phase advanced by {phase_delta}, expected (0, {MAXIMUM_REVERSAL_PHASE_ADVANCE}]"
        );
    }

    #[test]
    fn guard_and_attack_captures_use_prepared_runtime_pose_assets() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for name in ["swing.glb", "thrust.glb", "offhand.glb"] {
            let source = root
                .join("assets_src/biped/unarmed")
                .join(name.replace(".glb", ".casc"));
            let runtime = root.join("assets/animations/biped/unarmed").join(name);
            assert!(source.is_file(), "missing authored source {source:?}");
            let runtime_bytes = fs::read(&runtime).unwrap_or_else(|error| {
                panic!("missing prepared runtime asset {runtime:?}: {error}")
            });
            assert!(
                runtime_bytes.starts_with(b"glTF") && runtime_bytes.len() > 20,
                "prepared runtime asset is not a non-empty binary glTF: {runtime:?}"
            );
        }
    }
}
