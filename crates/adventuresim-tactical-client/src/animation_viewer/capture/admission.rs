//! Terminal capture admission rejection, separate from loading/readiness waits.

use super::*;
use adventuresim_building_generator::spatial_geometry::{GeometryError, Position};
use adventuresim_tactical_core::animation::{AnimationQuantity, AnimationValueError, UnitInterval};
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene::TerrainSupportError,
    scene_coordinates::{Scene, ScenePlanPoint},
};

/// One terminal authority: a rejected admission cannot return to driving.
pub(super) enum CaptureAdmission {
    Open,
    Rejected(RejectedCaptureAdmission),
    Terminated,
}

pub(super) enum RejectedCaptureAdmission {
    FrameInterval(AnimationValueError),
    Locomotion(AnimationValueError),
    Terrain(TerrainSupportError),
    Geometry(GeometryError),
}

impl CaptureSequence {
    pub(in crate::animation_viewer) fn admit_terrain_height(
        &mut self,
        terrain: &SceneTerrain,
        point: ScenePlanPoint,
    ) -> Option<SupportElevation> {
        match terrain.required_height_at(point) {
            Ok(height) => Some(height),
            Err(error) => {
                self.admission =
                    CaptureAdmission::Rejected(RejectedCaptureAdmission::Terrain(error));
                None
            }
        }
    }

    pub(super) fn admit_terrain_point(&mut self, metres: Vec2) -> Option<ScenePlanPoint> {
        match ScenePlanPoint::try_from(metres) {
            Ok(point) => Some(point),
            Err(error) => {
                self.admission =
                    CaptureAdmission::Rejected(RejectedCaptureAdmission::Geometry(error));
                None
            }
        }
    }

    pub(in crate::animation_viewer) fn admit_scene_position(
        &mut self,
        metres: Vec3,
    ) -> Option<Position<Scene>> {
        match Position::from_metres(metres) {
            Ok(position) => Some(position),
            Err(error) => {
                self.admission =
                    CaptureAdmission::Rejected(RejectedCaptureAdmission::Geometry(error));
                None
            }
        }
    }

    /// A failed support calculation is terminal admission, not another readiness wait.
    pub(super) fn admit_locomotion_support(
        &mut self,
        skeleton: &SkeletonState,
        speed: NonnegativeSpeed,
    ) -> Option<(
        adventuresim_tactical_core::animation::UnitInterval,
        adventuresim_tactical_core::animation::UnitInterval,
    )> {
        match locomotion_support_weights(skeleton, speed) {
            Ok(weights) => Some(weights),
            Err(error) => {
                self.admission =
                    CaptureAdmission::Rejected(RejectedCaptureAdmission::Locomotion(error));
                None
            }
        }
    }

    /// Active IK owns these observations; absent observations cannot become flight.
    pub(super) fn admit_ik_support(
        &mut self,
        diagnostics: LegIkDiagnostics,
        cadence: (UnitInterval, UnitInterval),
        skeleton: &SkeletonState,
        terrain_ik: TerrainIkMode,
    ) -> Option<(UnitInterval, UnitInterval)> {
        let ordinary_support_required =
            terrain_ik.is_enabled() && crate::animation::authored_locomotion_ik_owns(skeleton);
        if !ordinary_support_required
            && diagnostics.left_solve_target.is_none()
            && diagnostics.right_solve_target.is_none()
        {
            return Some(cadence);
        }
        match diagnostics
            .left_support_weight
            .zip(diagnostics.right_support_weight)
        {
            Some(weights) => Some(weights),
            None => {
                self.admission = CaptureAdmission::Rejected(RejectedCaptureAdmission::Locomotion(
                    AnimationValueError::UnavailableObservation {
                        quantity: AnimationQuantity::SupportObservation,
                    },
                ));
                None
            }
        }
    }

    /// Driving is permitted only before admission terminates and between captures.
    pub(super) fn can_drive_frame(&self) -> bool {
        self.admission.is_open()
            && !self.applied
            && !self.capture_in_flight
            && self.index < self.plan.len()
    }
}

impl CaptureAdmission {
    pub(super) fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

impl RejectedCaptureAdmission {
    pub(super) fn message(&self) -> String {
        match self {
            Self::FrameInterval(error) => format!("capture frame interval rejected: {error}\n"),
            Self::Locomotion(error) => {
                format!("capture locomotion observation rejected: {error}\n")
            }
            Self::Terrain(error) => format!("capture terrain support rejected: {error}\n"),
            Self::Geometry(error) => format!("capture scene geometry rejected: {error}\n"),
        }
    }
}

/// Admission rejection must terminate even while equipment is still loading.
pub(in crate::animation_viewer) fn reject_capture(
    mut sequence: ResMut<CaptureSequence>,
    mut exit: MessageWriter<AppExit>,
) {
    if let CaptureAdmission::Rejected(rejection) = &sequence.admission {
        fail_capture(&sequence, &rejection.message(), &mut exit);
        sequence.admission = CaptureAdmission::Terminated;
    }
}

pub(super) fn fail_capture(
    sequence: &CaptureSequence,
    message: &str,
    exit: &mut MessageWriter<AppExit>,
) {
    let path = sequence.output.join("failure.txt");
    if let Err(error) = fs::write(&path, message) {
        error!(%error, ?path, "Failed to write capture failure artifact");
    }
    error!(%message, ?path, "Animation capture failed");
    exit.write(AppExit::Error(std::num::NonZeroU8::MIN));
}
