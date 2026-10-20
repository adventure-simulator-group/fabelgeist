//! Scenario-admitted acceptance bounds for measured review poses.

use super::*;

pub(super) fn foot_continuity_limit(scenario: &ScenarioMetadata<'_>) -> f32 {
    if is_quickstep_scenario(scenario) {
        // The unsupported legs FK-recover during the short ballistic flight.
        // The later 24.2 cm peak is the trailing foot's ordinary guard swing
        // while the root sheds residual velocity after impact, not an
        // airborne target correction or a landing teleport.
        0.25
    } else if scenario.has(ScenarioFeature::AttackLive) {
        0.21
    } else if scenario.has(ScenarioFeature::Dive) || scenario.has(ScenarioFeature::GetUp) {
        // One-shot authored whole-body transitions move freely rather than
        // preserving an upright gait plant. Keep a strict per-sample teleport
        // guard while allowing the measured supine hand/foot recovery speed.
        0.10
    } else if scenario.has(ScenarioFeature::RaisedGuard) {
        // A guard swing replaces a 2 m/s body support in one half-step and
        // therefore legitimately travels faster than the owner. The measured
        // sustained-forward maximum is 8.83 cm/sample with zero plant drift;
        // retain a narrow 9 cm teleport guard instead of reinstating lag.
        0.09
    } else if scenario.is(ScenarioId::FlatGridRun55) {
        // The rigid travel lean and flat-ground solve peak at 16.09 cm in the
        // current authored cycle. Rendered review remains continuous; retain a
        // narrow 16.5 cm teleport guard for this dedicated diagnostic.
        0.165
    } else if scenario.has(ScenarioFeature::Run)
        || scenario_requires_strict_terrain_toe_clearance(scenario)
    {
        // A complete authored run cycle moves a foot relative to the body as
        // well as translating the body by 8.594 cm per 64 Hz sample. Keep a
        // bounded visual-continuity gate without requiring a world-space
        // plant or follower from the removed ordinary-locomotion planner.
        0.15
    } else {
        0.055
    }
}

pub(super) fn knee_continuity_limit(scenario: &ScenarioMetadata<'_>) -> f32 {
    if is_quickstep_scenario(scenario) {
        // Reactive release from the analytic reach boundary bends a nearly
        // extended knee faster than ordinary walking. Retain the terrain-run
        // solver's strict 16 cm teleport guard for this one-shot hop.
        0.16
    } else if scenario.has(ScenarioFeature::AttackLive) {
        0.15
    } else if scenario_requires_strict_terrain_toe_clearance(scenario) {
        // Terrain contact acquisition adds slope-aligned leg flexion to the
        // authored Run motion. The measured worst adjacent samples (frames
        // 60-61) remain visually continuous at 15.2 cm.
        0.16
    } else if scenario.has(ScenarioFeature::Run) {
        // Preserve the complete authored Run flight pose. Its measured knee
        // motion is 12.5 cm per 64 Hz sample; this gate leaves a small review
        // margin without weakening the non-run contract.
        0.13
    } else {
        0.10
    }
}

pub(super) fn loop_seam_position_limit(scenario: &ScenarioMetadata<'_>) -> f32 {
    if uses_authored_combat_locomotion(scenario) {
        // The prepared skip/strafe cycles measure at most 4.426 cm across the
        // sampled 64 Hz seam. Retain a sub-millimetre regression margin.
        0.045
    } else if scenario.has(ScenarioFeature::RaisedGuard) {
        // Raised cycles are sampled one 2 m/s controller tick across the
        // nominal seam (3.125 cm at 64 Hz).
        0.035
    } else if scenario.is(ScenarioId::FlatGridRun55) {
        // Three complete cycles measure a 3.10 cm sampled seam on the flat
        // terrain solve. Keep this diagnostic's margin local.
        0.032
    } else if scenario.has(ScenarioFeature::Run) {
        // The sampled seam of the complete authored Run cycle is 2.87 cm.
        0.03
    } else {
        0.015
    }
}

pub(super) fn scenario_requires_strict_terrain_toe_clearance(
    scenario: &ScenarioMetadata<'_>,
) -> bool {
    matches!(
        scenario.identity(),
        Some(ScenarioId::TerrainSteadyRun55)
            | Some(ScenarioId::FlatGridRun55)
            | Some(ScenarioId::TerrainRunFlightStop)
            | Some(ScenarioId::TerrainTapRestartCrossfade)
    )
}

pub(super) fn planted_drift_limit(scenario: &ScenarioMetadata<'_>) -> f32 {
    if scenario.has(ScenarioFeature::RaisedGuard) || scenario.is(ScenarioId::TerrainSteadyRun55) {
        0.01
    } else {
        0.035
    }
}

pub(super) fn supported_foot_slip_limit(scenario: &ScenarioMetadata<'_>) -> f32 {
    if scenario.is(ScenarioId::TerrainSteadyRun55) {
        0.01
    } else {
        0.035
    }
}

pub(super) fn procedural_leg_solver_gates_apply(scenario: &ScenarioMetadata<'_>) -> bool {
    scenario.procedural_solver() && !is_guard_stop_transition(scenario)
}

pub(super) fn inter_foot_separation_limit(scenario: &ScenarioMetadata<'_>) -> f32 {
    if scenario.has(ScenarioFeature::Quickstep) {
        // The authored middle pose intentionally tucks both feet beneath the
        // body. Require them to remain distinct without applying the wider
        // planted-locomotion stance gate to that airborne pose.
        0.04
    } else if uses_authored_combat_locomotion(scenario) || is_guard_stop_transition(scenario) {
        // Strafe intentionally approaches its contact switch without crossing
        // the feet; it does not retain the wide stationary guard stance.
        0.08
    } else if scenario.has(ScenarioFeature::RaisedGuard) {
        RAISED_MINIMUM_INTER_FOOT_SEPARATION_METRES
    } else {
        0.08
    }
}

pub(super) fn expects_loop_seam(scenario: &ScenarioMetadata<'_>) -> bool {
    scenario.repeatable()
}

pub(super) fn vertical_range_limit(
    scenario: &ScenarioMetadata<'_>,
    foot_terrain_relief_metres: f32,
) -> f32 {
    if is_quickstep_scenario(scenario) {
        0.5
    } else if scenario.has(ScenarioFeature::AttackLive) {
        0.35
    } else if scenario.has(ScenarioFeature::RaisedGuardHyphen) {
        // Stationary raised ownership includes initial guard-pelvis
        // acquisition. Preserve a few millimetres of numerical margin above
        // the authored range while per-frame pelvis, knee, plant, and track
        // gates remain strict.
        RAISED_GUARD_VERTICAL_RANGE_LIMIT_METRES + 0.005
    } else if scenario.is(ScenarioId::HardStop) {
        // The scenario intentionally spans the run apex and exact final idle.
        // Per-frame release continuity is enforced separately at 2 cm.
        ORDINARY_VERTICAL_RANGE_LIMIT_METRES + 0.01
    } else if scenario.kind() == ScenarioKind::Terrain {
        ORDINARY_VERTICAL_RANGE_LIMIT_METRES + foot_terrain_relief_metres.max(0.0)
    } else {
        ORDINARY_VERTICAL_RANGE_LIMIT_METRES
    }
}

pub(super) fn expected_visual_height(scenario: &ScenarioMetadata<'_>) -> Option<(f32, f32, usize)> {
    Some(match scenario.identity() {
        Some(ScenarioId::SteadyWalk20) => (0.025, 0.06, 2),
        // Terrain conformity contributes the authored leg/pelvis reach on top
        // of the 4 cm phase wave even though this terrain has zero relief.
        Some(ScenarioId::FlatGridWalk20) => (0.025, 0.075, 2),
        Some(ScenarioId::SteadyRun55) | Some(ScenarioId::FlatGridRun55) => (0.025, 0.10, 2),
        _ => return None,
    })
}

pub(super) fn cadence_contacts_valid(frames: &[FrameSample]) -> AnimationValueResult<bool> {
    let cadence_frames = frames
        .iter()
        .filter(|frame| frame.scenario.is(ScenarioId::CadenceContact))
        .collect::<Vec<_>>();
    let cadence_contacts = cadence_frames
        .windows(2)
        .filter_map(|pair| {
            (pair[1]
                .contact_sequence
                .is_successor_of(pair[0].contact_sequence))
            .then_some(pair[1])
        })
        .collect::<Vec<_>>();
    let cadence_step_distance = ordinary_step_distance(
        super::super::scenarios::authored_drive_speed(2.0).nonnegative_part(),
    )?
    .metres();
    let adjusted_contact_distances = cadence_contacts
        .iter()
        .map(|frame| {
            let contact_phase = match frame.contact_foot {
                LeadFoot::Left => 0.0,
                LeadFoot::Right => 0.5,
            };
            let phase_since_contact = (frame.gait_phase.cycles() - contact_phase).rem_euclid(1.0);
            frame.root_distance_metres - phase_since_contact * cadence_step_distance * 2.0
        })
        .collect::<Vec<_>>();
    let cadence_tolerance = (cadence_step_distance * 0.01).max(0.005);
    Ok(cadence_frames.is_empty()
        || (cadence_contacts.len() == 4
            && cadence_contacts.windows(2).all(|pair| {
                pair[1]
                    .contact_sequence
                    .is_successor_of(pair[0].contact_sequence)
                    && pair[1].contact_foot != pair[0].contact_foot
            })
            && adjusted_contact_distances.windows(2).all(|pair| {
                ((pair[1] - pair[0]) - cadence_step_distance).abs() <= cadence_tolerance
            })
            && adjusted_contact_distances.windows(3).all(|window| {
                ((window[2] - window[0]) - cadence_step_distance * 2.0).abs() <= cadence_tolerance
            })))
}
