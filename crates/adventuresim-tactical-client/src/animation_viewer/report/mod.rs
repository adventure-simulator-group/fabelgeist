//! Capture report assembly and artifact writing.

use super::*;
pub(super) use bones::{
    CapturedBones, REPORT_BONES, parent_bone, report_bone_label, serialize_report_bone,
};

mod acceptance_limits;
mod body_response_validation;
mod bones;
mod html;
mod metrics;
mod model;
mod validation;

use acceptance_limits::*;
use body_response_validation::*;
use html::*;
use metrics::*;
pub(super) use model::*;
use validation::*;

pub(super) fn invalidate_previous_report(output: &std::path::Path) {
    for name in [
        "manifest.json",
        "index.html",
        "failure.txt",
        "global-bone-transforms.jsonl",
    ] {
        let path = output.join(name);
        if let Err(error) = fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            panic!("failed to invalidate previous animation report {path:?}: {error}");
        }
    }
}

fn capture_scenarios_complete(frames: &[FrameSample], plan: &[PlannedFrame]) -> bool {
    frames.len() == plan.len()
        && frames.iter().zip(plan).all(|(frame, planned)| {
            frame.scenario == planned.scenario
                && frame.scenario_frame == planned.scenario_frame
                && REPORT_BONES
                    .iter()
                    .all(|(role, _)| frame.bones.contains_key(*role))
        })
}

fn capture_artifacts_written(output: &std::path::Path, frames: &[FrameSample]) -> bool {
    frames.iter().all(|frame| {
        VIEWS.iter().all(|view| {
            frame
                .screenshots
                .get(view.slug())
                .and_then(|name| fs::metadata(output.join(name)).ok())
                .is_some_and(|metadata| metadata.len() > 0)
        })
    })
}

pub(super) fn write_completed_capture(completed: CompletedCapture) -> AppExit {
    let CompletedReport {
        output,
        manifest,
        global_bone_frames,
    } = match build_completed_report(completed) {
        Ok(report) => report,
        Err(error) => {
            error!(%error, "capture report admission failed");
            return AppExit::Error(std::num::NonZeroU8::MIN);
        }
    };
    let global_bone_trace_path = output.join("global-bone-transforms.jsonl");
    let trace_file = File::create(&global_bone_trace_path)
        .unwrap_or_else(|error| panic!("failed to create {global_bone_trace_path:?}: {error}"));
    let mut trace_writer = BufWriter::new(trace_file);
    for frame in &global_bone_frames {
        serde_json::to_writer(&mut trace_writer, frame).unwrap_or_else(|error| {
            panic!("failed to serialize {global_bone_trace_path:?}: {error}")
        });
        trace_writer
            .write_all(b"\n")
            .unwrap_or_else(|error| panic!("failed to write {global_bone_trace_path:?}: {error}"));
    }
    trace_writer
        .flush()
        .unwrap_or_else(|error| panic!("failed to flush {global_bone_trace_path:?}: {error}"));
    let manifest_path = output.join("manifest.json");
    fs::write(
        &manifest_path,
        match serde_json::to_string_pretty(&manifest) {
            Ok(json) => json,
            Err(error) => {
                error!(%error, "capture manifest serialization failed");
                return AppExit::Error(std::num::NonZeroU8::MIN);
            }
        },
    )
    .unwrap_or_else(|error| panic!("failed to write {manifest_path:?}: {error}"));
    let index_path = output.join("index.html");
    let html = match review_html(&manifest) {
        Ok(html) => html,
        Err(error) => {
            error!(%error, "capture HTML serialization failed");
            return AppExit::Error(std::num::NonZeroU8::MIN);
        }
    };
    fs::write(&index_path, html)
        .unwrap_or_else(|error| panic!("failed to write {index_path:?}: {error}"));
    info!(path = ?index_path, "Animation review capture completed");
    if validation_passed(&manifest.validation) {
        AppExit::Success
    } else {
        let failure_path = output.join("failure.txt");
        fs::write(
            &failure_path,
            "capture failed artifact/completeness/continuity/biomechanics/penetration/distinct-view validation; inspect manifest.json\n",
        )
        .unwrap_or_else(|error| panic!("failed to write {failure_path:?}: {error}"));
        AppExit::Error(1.try_into().expect("one is non-zero"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_output(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "adventuresim-animation-viewer-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn report_invalidation_preserves_unrelated_files() {
        let output = unique_test_output("invalidate");
        fs::create_dir_all(&output).unwrap();
        for name in [
            "manifest.json",
            "index.html",
            "failure.txt",
            "global-bone-transforms.jsonl",
            "notes.txt",
        ] {
            fs::write(output.join(name), b"old").unwrap();
        }
        invalidate_previous_report(&output);
        assert!(!output.join("manifest.json").exists());
        assert!(!output.join("index.html").exists());
        assert!(!output.join("failure.txt").exists());
        assert!(!output.join("global-bone-transforms.jsonl").exists());
        assert!(output.join("notes.txt").exists());
        fs::remove_dir_all(output).unwrap();
    }

    #[test]
    fn capture_requires_every_nonempty_view_artifact() {
        let output = unique_test_output("artifacts");
        fs::create_dir_all(&output).unwrap();
        let screenshots = VIEWS
            .into_iter()
            .map(|view| (view.slug().to_owned(), format!("{}.png", view.slug())))
            .collect::<BTreeMap<_, _>>();
        let frame = FrameSample {
            scenario: "test".into(),
            scenario_frame: 0,
            time_seconds: AnimationElapsed::from_seconds(0.0).unwrap(),
            speed_metres_per_second: SignedSpeed::from_metres_per_second(0.0).unwrap(),
            gait_phase: CyclePhase::from_cycles(0.0).unwrap(),
            locomotion_sample_tick: AnimationSampleTick::from(0u64),
            body_acceleration: Vec3::ZERO.to_array(),
            world_acceleration: Vec3::ZERO.to_array(),
            bouncy_active_springs: 0,
            bouncy_maximum_deflection_radians: 0.0,
            contact_sequence: ContactSequence::ZERO,
            contact_foot: LeadFoot::Left,
            landing_sequence: adventuresim_tactical_core::animation::LandingSequence::ZERO,
            landing_impact_speed: LandingImpactSpeed::ZERO,
            body_lean_pitch_degrees: BodyAngle::ZERO,
            body_lean_roll_degrees: BodyAngle::ZERO,
            landing_compression_metres: HeightDistance::ZERO,
            root_distance_metres: 0.0,
            root_position_metres: Vec3::ZERO.to_array(),
            world_travel_direction: Vec3::Z.to_array(),
            desired_body_forward_direction: Vec3::Z.to_array(),
            body_forward_direction: Vec3::Z.to_array(),
            body_rotation_xyzw: Quat::IDENTITY.to_array(),
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
            action: SkeletonAction::None,
            action_phase: adventuresim_tactical_core::animation::UnitInterval::ZERO,
            attack_animation: None,
            strike_family: StrikeFamily::Thrust,
            guard_action: false,
            left_support_weight: 1.0,
            right_support_weight: 1.0,
            desired_left_foot_target: None,
            desired_right_foot_target: None,
            ik_left_authored_target: None,
            ik_right_authored_target: None,
            ik_left_planned_contact: None,
            ik_right_planned_contact: None,
            ik_settle_capture_point: None,
            ik_left_solve_target: None,
            ik_right_solve_target: None,
            ik_left_support_weight: 0.0,
            ik_right_support_weight: 0.0,
            ik_left_release_active: false,
            ik_right_release_active: false,
            ik_left_release_target: None,
            ik_right_release_target: None,
            ik_settle_progress: None,
            ik_left_knee_foot_yaw_offset_degrees: 0.0,
            ik_right_knee_foot_yaw_offset_degrees: 0.0,
            semantic_route_requested_path: SemanticRoutePath::GeneralPose,
            semantic_route_selected_path: SemanticRoutePath::GeneralPose,
            semantic_route_runtime_evaluated: false,
            screenshots,
            bones: CapturedBones::default(),
        };
        for name in frame.screenshots.values() {
            fs::write(output.join(name), b"png").unwrap();
        }
        assert!(capture_artifacts_written(
            &output,
            std::slice::from_ref(&frame)
        ));
        fs::write(output.join("front.png"), b"").unwrap();
        assert!(!capture_artifacts_written(&output, &[frame]));
        fs::remove_dir_all(output).unwrap();
    }
}
