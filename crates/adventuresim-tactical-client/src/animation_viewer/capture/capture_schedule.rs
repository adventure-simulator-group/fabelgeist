//! Capture-phase registration and its ordered readiness/terminal-consumption policy.

use super::capture_time::{
    CaptureTimeControl, arm_ragdoll_capture, capture_time_ready, finish_ragdoll_step,
    schedule_ragdoll_step,
};
use super::overlay::draw_skeleton_overlay;
use super::{
    CaptureSequence, capture_frame, collect_locomotion_presentation_events, draw_flat_ground_grid,
    drive_sequence, freeze_capture_look, reject_capture,
};
use crate::animation_viewer::{
    harness, inspection, setup::setup_viewer, view::position_capture_camera,
};
use crate::camera::TacticalCameraSet;
use bevy::prelude::*;

pub(in crate::animation_viewer) struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaptureTimeControl>()
            .add_systems(Startup, setup_viewer)
            .add_systems(
                PreUpdate,
                (
                    finish_ragdoll_step,
                    drive_sequence.run_if(admission_is_open),
                    schedule_ragdoll_step,
                    freeze_capture_look,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                position_capture_camera
                    .after(TacticalCameraSet::Offset)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                (draw_flat_ground_grid, draw_skeleton_overlay)
                    .chain()
                    .after(TransformSystems::Propagate),
            )
            .add_systems(
                Last,
                (
                    inspection::inspect_materials,
                    harness::update_readiness,
                    collect_locomotion_presentation_events,
                    reject_capture,
                    arm_ragdoll_capture
                        .run_if(harness::ready)
                        .run_if(admission_is_open),
                    capture_frame
                        .run_if(harness::ready)
                        .run_if(capture_time_ready)
                        .run_if(admission_is_open),
                )
                    .chain(),
            );
    }
}

fn admission_is_open(sequence: Res<CaptureSequence>) -> bool {
    sequence.admission.is_open()
}
