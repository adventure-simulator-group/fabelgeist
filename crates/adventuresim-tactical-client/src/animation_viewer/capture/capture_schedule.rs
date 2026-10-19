//! Capture-phase registration and its ordered readiness/terminal-consumption policy.

use super::{
    capture_frame, collect_locomotion_presentation_events, draw_flat_ground_grid,
    draw_skeleton_overlay, drive_sequence, freeze_capture_look, reject_capture,
};
use crate::animation_viewer::{
    harness, inspection, setup::setup_viewer, view::position_capture_camera,
};
use crate::camera::TacticalCameraSet;
use bevy::prelude::*;

pub(in crate::animation_viewer) struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_viewer)
            .add_systems(PreUpdate, (drive_sequence, freeze_capture_look).chain())
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
                    capture_frame.run_if(harness::ready),
                )
                    .chain(),
            );
    }
}
