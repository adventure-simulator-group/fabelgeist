//! Scene-root fixture motion and explicit capture ground-observation authority.

use super::*;
use adventuresim_building_generator::spatial_geometry::{
    CoordinateAxis, GeometryError, GeometryRole, Position,
};
use fabelgeist_geometry::{GeometryResult, SignedLength};
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene_coordinates::{Scene, ScenePlanPoint},
};

#[derive(Clone, Copy)]
pub(super) enum CaptureGround {
    Terrain,
    AuthoredPlane(SupportElevation),
}

impl CaptureGround {
    pub(super) fn collect_bones(
        self,
        subject: Entity,
        bones: &Query<(&HumanoidBone, &GlobalTransform)>,
        terrain: &SceneTerrain,
    ) -> GeometryResult<CapturedBones> {
        bones
            .iter()
            .filter(|(bone, _)| bone.owner == subject && report_bone_label(bone.role).is_some())
            .map(|(bone, transform)| {
                let rotation = transform.rotation();
                let elevation = match self {
                    CaptureGround::AuthoredPlane(height) => Some(height),
                    CaptureGround::Terrain => {
                        ScenePlanPoint::from_metres(transform.translation().xz())
                            .and_then(|point| terrain.height_at(point))
                    }
                };
                Ok((
                    bone.role,
                    BoneSample {
                        position: transform.translation().to_array(),
                        rotation_xyzw: [rotation.x, rotation.y, rotation.z, rotation.w],
                        terrain_clearance_metres: elevation
                            .map(|height| {
                                SignedLength::from_metres(transform.translation().y - height.metres())
                            })
                            .transpose()?,
                    },
                ))
            })
            .collect()
    }
}

impl CaptureSequence {
    /// Admit the initial scene root while retaining its current scale.
    pub(super) fn initial_root(
        &mut self,
        terrain: &SceneTerrain,
        mut transform: Transform,
    ) -> Option<Transform> {
        let ground = self.admit_terrain_height(terrain, ScenePlanPoint::ORIGIN)?;
        let position = self.admit_scene_position(Vec3::new(
            0.0,
            ground.metres() + CAPTURE_ROOT_GROUND_OFFSET_METRES,
            0.0,
        ))?;
        transform.translation = position.metres();
        transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        Some(transform)
    }

    /// Native scene-root translation advances by the existing controller step;
    /// admit its footprint before queries and its complete position before Bevy.
    pub(super) fn fixture_root_position(
        &mut self,
        terrain: &SceneTerrain,
        current: Vec3,
        frame: &PlannedFrame,
        motion: &subject_motion::CaptureMotion,
        terrain_ik: TerrainIkMode,
    ) -> Option<Position<Scene>> {
        let horizontal = current.xz() + motion.world_velocity.xz() * motion.elapsed.seconds();
        let point = self.admit_terrain_point(horizontal)?;
        let vertical = if is_quickstep_scenario(&frame.scenario) {
            let ground = self.admit_terrain_height(terrain, point)?.metres()
                + CAPTURE_ROOT_GROUND_OFFSET_METRES;
            ground
                + quickstep_fixture_vertical_state(frame.scenario_frame)
                    .0
                    .max(0.0)
        } else if terrain_ik.is_enabled() {
            self.admit_terrain_height(terrain, point)?.metres() + CAPTURE_ROOT_GROUND_OFFSET_METRES
        } else {
            current.y
        };
        self.admit_scene_position(Vec3::new(horizontal.x, vertical, horizontal.y))
    }

    /// The full ragdoll scenario lowers the production-like controller root
    /// while its capsule island resolves contact against the same terrain.
    pub(super) fn full_ragdoll_root(
        &mut self,
        terrain: &SceneTerrain,
        mut transform: Transform,
        frame: &PlannedFrame,
    ) -> Option<Transform> {
        let fall = ((frame.scenario_frame + 4) as f32 / 8.0).clamp(0.0, 1.0);
        transform.rotation = Quat::from_rotation_y(std::f32::consts::PI)
            * Quat::from_rotation_x(1.25 * smoothstep01(fall));
        let ground = self.admit_terrain_height(terrain, ScenePlanPoint::ORIGIN)?;
        transform.translation.y =
            ground.metres() + CAPTURE_ROOT_GROUND_OFFSET_METRES - 0.62 * smoothstep01(fall);
        transform.translation = self.admit_scene_position(transform.translation)?.metres();
        Some(transform)
    }

    /// An authored flat plane is measured from the visible scene-root datum;
    /// failure to admit that observation cannot select terrain instead.
    pub(super) fn capture_ground(
        &mut self,
        terrain_ik: TerrainIkMode,
        subject: Vec3,
    ) -> Option<CaptureGround> {
        if terrain_ik.is_enabled() {
            return Some(CaptureGround::Terrain);
        }
        let height = subject.y - CAPTURE_ROOT_GROUND_OFFSET_METRES;
        if let Some(height) = SupportElevation::from_metres(height) {
            return Some(CaptureGround::AuthoredPlane(height));
        }
        self.admission = CaptureAdmission::Rejected(RejectedCaptureAdmission::Geometry(
            GeometryError::NonFinite {
                role: GeometryRole::Elevation,
                axis: CoordinateAxis::Y,
            },
        ));
        None
    }
}
