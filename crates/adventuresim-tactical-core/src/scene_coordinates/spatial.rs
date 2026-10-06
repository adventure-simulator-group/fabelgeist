//! Tactical core owns the handoff from architectural to scene and gate datums.
use super::ArchitecturalPlanProjection;
use crate::{city_layout::grounding::SupportElevation, scene_input::BuildingOrientation};
use adventuresim_building_generator::spatial_geometry::GeometryResult as Result;
use adventuresim_building_generator::spatial_geometry::{
    Architectural, Displacement, GeometryError, GeometryFrame, PlanDirection, Position, Radians,
    RigidRotation,
};
use adventuresim_building_generator::{DoorError, DoorErrorCause, DoorSpec};
use bevy::{
    math::{Quat, Vec2, Vec3},
    prelude::{Reflect, Transform},
};

/// Scene X/Y/Z metres, including absolute scene elevation.
///
/// Frames and arithmetic roles are incompatible even when their wire layouts match.
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{Architectural, Position};
/// use adventuresim_tactical_core::scene_coordinates::Scene;
/// let architectural: Position<Architectural> = Position::ORIGIN;
/// let scene: Position<Scene> = architectural;
/// ```
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{Displacement, Position};
/// use adventuresim_tactical_core::scene_coordinates::{GateRelative, Scene};
/// let gate: Position<GateRelative> = Position::ORIGIN;
/// let scene: Position<Scene> = gate;
/// ```
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{Displacement, Position};
/// use adventuresim_tactical_core::scene_coordinates::Scene;
/// let position: Position<Scene> = Position::ORIGIN;
/// let displacement: Displacement<Scene> = position;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum Scene {}
impl GeometryFrame for Scene {}
/// Scene X/Z metres and Y metres relative to the selected gate support datum.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum GateRelative {}
impl GeometryFrame for GateRelative {}

/// Scene X/Z metres, with Y above the local support surface at each point.
/// Enclosure walls follow this surface; gate-post heads use GateRelative.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum GroundRelative {}
impl GeometryFrame for GroundRelative {}

/// Architectural Y=0 maps to the selected scene floor elevation.
#[derive(Clone, Copy, Debug)]
pub struct ArchitecturalFloorDatum {
    pub plan: ArchitecturalPlanProjection,
    pub floor: SupportElevation,
}
impl ArchitecturalFloorDatum {
    pub fn collision_centre(self, origin: Position<Architectural>) -> Result<CollisionCentreDatum> {
        CollisionCentreDatum::new(
            origin,
            Position::from_metres(Vec3::new(
                self.plan.centre.metres().x,
                self.floor.metres() + origin.metres().y,
                self.plan.centre.metres().y,
            ))?,
            self.plan.orientation,
        )
    }
}

/// The collision centre is subtracted before applying the existing native
/// transform. This retains the established f32 operation order and floor datum.
#[derive(Clone, Copy, Debug)]
pub struct CollisionCentreDatum {
    origin: Position<Architectural>,
    centre: Position<Scene>,
    pub(super) orientation: BuildingOrientation,
}
impl CollisionCentreDatum {
    pub fn new(
        origin: Position<Architectural>,
        centre: Position<Scene>,
        orientation: BuildingOrientation,
    ) -> Result<Self> {
        if !orientation.is_valid() {
            return Err(GeometryError::InvalidProjection);
        }
        Ok(Self {
            origin,
            centre,
            orientation,
        })
    }
    pub fn native_transform(self) -> Transform {
        Transform::from_translation(self.centre.metres())
            .with_rotation(Quat::from_rotation_y(self.orientation.yaw_radians()))
    }
    pub fn point(self, point: Position<Architectural>) -> Result<Position<Scene>> {
        Position::from_metres(
            self.native_transform()
                .transform_point(point.metres() - self.origin.metres()),
        )
    }
    pub fn displacement(
        self,
        displacement: Displacement<Architectural>,
    ) -> Result<Displacement<Scene>> {
        Displacement::from_metres(self.native_transform().rotation * displacement.metres())
    }
    pub fn architectural_point(self, point: Position<Scene>) -> Result<Position<Architectural>> {
        let transform = self.native_transform();
        Position::from_metres(
            transform.rotation.inverse() * (point.metres() - transform.translation)
                + self.origin.metres(),
        )
    }
    pub fn door(
        self,
        leaf: DoorSpec<Architectural>,
    ) -> std::result::Result<SceneDoorPose, DoorError> {
        let rotation = self.native_transform().rotation;
        let direction = |value: PlanDirection<Architectural>| {
            let value = rotation * Vec3::new(value.vector().x, 0.0, value.vector().y);
            PlanDirection::from_normalized(Vec2::new(value.x, value.z))
        };
        let construct = || {
            Ok(SceneDoorPose {
                // Keep the original quaternion product at render/physics ports.
                native_rotation: RigidRotation::from_quaternion(
                    rotation * Quat::from_rotation_y(leaf.closed_yaw_radians.radians()),
                )?,
                leaf: DoorSpec {
                    opening: leaf.opening,
                    source: leaf.source,
                    closed_centre: self.point(leaf.closed_centre)?,
                    hinge_centre: self.point(leaf.hinge_centre)?,
                    size_metres: leaf.size_metres,
                    closed_yaw_radians: Radians::new(
                        self.orientation.yaw_radians() + leaf.closed_yaw_radians.radians(),
                    )?,
                    tangent: direction(leaf.tangent)?,
                    outward: direction(leaf.outward)?,
                    open_angle_radians: leaf.open_angle_radians,
                },
            })
        };
        construct().map_err(|cause| DoorError {
            opening: leaf.opening,
            source_id: Some(leaf.source),
            cause: DoorErrorCause::Geometry(cause),
        })
    }
}

/// One already-bound gate elevation. X/Z are already in scene coordinates.
#[derive(Clone, Copy, Debug)]
pub struct GateDatum(pub SupportElevation);
impl GateDatum {
    pub fn from_metres(metres: f32) -> Result<Self> {
        SupportElevation::from_metres(metres)
            .map(Self)
            .ok_or(GeometryError::NonFinite {
                role: adventuresim_building_generator::spatial_geometry::GeometryRole::Elevation,
                axis: adventuresim_building_generator::spatial_geometry::CoordinateAxis::Y,
            })
    }
    pub fn point(self, point: Position<GateRelative>) -> Result<Position<Scene>> {
        Position::from_metres(point.metres() + Vec3::Y * self.0.metres())
    }
    pub fn gate_point(self, point: Position<Scene>) -> Result<Position<GateRelative>> {
        Position::from_metres(point.metres() - Vec3::Y * self.0.metres())
    }
    pub fn door(
        self,
        leaf: DoorSpec<GateRelative>,
    ) -> std::result::Result<SceneDoorPose, DoorError> {
        let construct = || {
            Ok(SceneDoorPose {
                native_rotation: RigidRotation::from_quaternion(Quat::from_rotation_y(
                    leaf.closed_yaw_radians.radians(),
                ))?,
                leaf: DoorSpec {
                    opening: leaf.opening,
                    source: leaf.source,
                    closed_centre: self.point(leaf.closed_centre)?,
                    hinge_centre: self.point(leaf.hinge_centre)?,
                    size_metres: leaf.size_metres,
                    closed_yaw_radians: leaf.closed_yaw_radians,
                    tangent: PlanDirection::from_normalized(leaf.tangent.vector())?,
                    outward: PlanDirection::from_normalized(leaf.outward.vector())?,
                    open_angle_radians: leaf.open_angle_radians,
                },
            })
        };
        construct().map_err(|cause| DoorError {
            opening: leaf.opening,
            source_id: Some(leaf.source),
            cause: DoorErrorCause::Geometry(cause),
        })
    }
}

/// The scene leaf and its original native quaternion product are one conversion
/// result. Native rotation is an adapter output, not a second authoring input.
/// The paired values can only be constructed by the datum conversions.
///
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::Radians;
/// use adventuresim_tactical_core::scene_coordinates::SceneDoorPose;
/// fn change_yaw(mut pose: SceneDoorPose, yaw: Radians) {
///     pose.leaf.closed_yaw_radians = yaw;
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SceneDoorPose {
    leaf: DoorSpec<Scene>,
    native_rotation: RigidRotation,
}

/// Property-plan X/Z axes and Y relative to its ungraded packing datum.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum PlotRelative {}
impl GeometryFrame for PlotRelative {}

/// Architectural floor elevation expressed relative to the gate's datum. The
/// packing producer uses zero before terrain grounding assigns scene support.
#[derive(Clone, Copy, Debug)]
pub struct ArchitecturalGateDatum {
    pub plan: ArchitecturalPlanProjection,
    pub floor: adventuresim_building_generator::spatial_geometry::Elevation<GateRelative>,
}
impl ArchitecturalGateDatum {
    pub fn point(self, point: Position<Architectural>) -> Result<Position<GateRelative>> {
        if !self.plan.orientation.is_valid() {
            return Err(GeometryError::InvalidProjection);
        }
        let origin = self.plan.origin.metres();
        let centre = self.plan.centre.metres();
        Position::from_metres(
            Quat::from_rotation_y(self.plan.orientation.yaw_radians())
                * (point.metres() - Vec3::new(origin.x, 0.0, origin.y))
                + Vec3::new(centre.x, self.floor.metres(), centre.y),
        )
    }
}

impl SceneDoorPose {
    /// The converted scene leaf. Changing the returned copy leaves this pose intact.
    pub fn leaf(self) -> DoorSpec<Scene> {
        self.leaf
    }

    /// Original quaternion product retained at rendering and physics boundaries.
    pub fn native_rotation(self) -> Quat {
        self.native_rotation.quaternion()
    }
}
