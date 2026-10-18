//! Fixed enclosure and operable gate geometry share one descriptor.
use super::*;
use crate::scene_coordinates::{GateRelative, GroundRelative};
use adventuresim_building_generator::spatial_geometry::{
    CuboidDimensions, GeometryError, Position,
};
use adventuresim_building_generator::{DoorSpec, OpeningAssemblyId, ResolvedItemId};
use bevy::math::Vec3;

const BOUNDARY_GATE_OPENING_DOMAIN: u64 = 0xc300_0000_0000_0000;
const GATE_LEAF_THICKNESS_METRES: f32 = 0.055;
const GATE_GROUND_GAP_METRES: f32 = 0.05;
const GATE_POST_WIDTH_METRES: f32 = 0.3;
const GATE_POST_HEAD_METRES: f32 = 0.15;
const WALL_CAP_HEIGHT_METRES: f32 = 0.1;
const WALL_CAP_OVERHANG_METRES: f32 = 0.025;
const GATE_OPEN_ANGLE_RADIANS: f32 = core::f32::consts::FRAC_PI_2;
const GATE_HINGE_CLEARANCE_METRES: f32 = 0.025;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CityBoundaryMaterial {
    Masonry,
    Timber,
    Iron,
}

/// Identity of one fixed enclosure member, retained through support projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundarySupportElement {
    Owner,
    GateLanding,
    Wall(usize),
    WallCap(usize),
    GatePost(PropertySide),
}

/// The datum and support policy accompany each pose rather than being inferred
/// from an unframed vector or an independently enumerated member list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CityBoundaryPose {
    Wall {
        index: usize,
        centre: Position<GroundRelative>,
    },
    WallCap {
        index: usize,
        centre: Position<GroundRelative>,
    },
    GatePost {
        side: PropertySide,
        centre: Position<GateRelative>,
    },
}
impl CityBoundaryPose {
    pub fn element(self) -> BoundarySupportElement {
        match self {
            Self::Wall { index, .. } => BoundarySupportElement::Wall(index),
            Self::WallCap { index, .. } => BoundarySupportElement::WallCap(index),
            Self::GatePost { side, .. } => BoundarySupportElement::GatePost(side),
        }
    }
    /// Native arithmetic adapter: X/Z are already scene coordinates in all
    /// cases. The caller must match the pose before interpreting its Y datum.
    pub fn plan_metres(self) -> Vec2 {
        match self {
            Self::Wall { centre, .. } | Self::WallCap { centre, .. } => {
                Vec2::new(centre.metres().x, centre.metres().z)
            }
            Self::GatePost { centre, .. } => Vec2::new(centre.metres().x, centre.metres().z),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityBoundaryMember {
    pub pose: CityBoundaryPose,
    pub size_metres: CuboidDimensions,
    pub orientation: BuildingOrientation,
    pub material: CityBoundaryMaterial,
}
impl CityBoundaryMember {
    /// Before grading, the packing check explicitly binds every relative height
    /// to the same zero datum. Terrain projection later uses the declared pose.
    pub fn packing_cuboid(
        self,
        source: ResolvedItemId,
    ) -> Result<
        adventuresim_building_generator::CollisionCuboid<GateRelative>,
        adventuresim_building_generator::CollisionError,
    > {
        let centre = match self.pose {
            CityBoundaryPose::Wall { centre, .. } | CityBoundaryPose::WallCap { centre, .. } => {
                Position::from_metres(centre.metres()).map_err(|cause| {
                    adventuresim_building_generator::CollisionError {
                        source_id: source,
                        cause,
                    }
                })?
            }
            CityBoundaryPose::GatePost { centre, .. } => centre,
        };
        Ok(adventuresim_building_generator::CollisionCuboid {
            source,
            centre,
            size: self.size_metres,
            yaw_radians: adventuresim_building_generator::spatial_geometry::Radians::new(
                self.orientation.yaw_radians(),
            )
            .map_err(|cause| adventuresim_building_generator::CollisionError {
                source_id: source,
                cause,
            })?,
            crossfall_radians: adventuresim_building_generator::spatial_geometry::Radians::ZERO,
            longfall_radians: adventuresim_building_generator::spatial_geometry::Radians::ZERO,
        })
    }
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, thiserror::Error)]
#[error("enclosure {element:?}: {cause}")]
pub struct BoundaryGeometryError {
    pub element: BoundarySupportElement,
    #[source]
    pub cause: GeometryError,
}
impl CityBoundary {
    pub fn fixed_members(&self) -> Result<Vec<CityBoundaryMember>, BoundaryGeometryError> {
        let mut members = Vec::new();
        for (index, wall) in self.walls.iter().enumerate() {
            let centre = (wall.start_metres + wall.end_metres) * 0.5;
            let length = wall.start_metres.distance(wall.end_metres);
            let orientation =
                BuildingOrientation::from_frontage_tangent(wall.end_metres - wall.start_metres)
                    .ok_or(BoundaryGeometryError {
                        element: BoundarySupportElement::Wall(index),
                        cause: GeometryError::InvalidProjection,
                    })?;
            for (cap, height, thickness, elevation) in [
                (
                    false,
                    wall.height_metres,
                    wall.thickness_metres,
                    wall.height_metres * 0.5,
                ),
                (
                    true,
                    WALL_CAP_HEIGHT_METRES,
                    wall.thickness_metres + WALL_CAP_OVERHANG_METRES * 2.0,
                    wall.height_metres + WALL_CAP_HEIGHT_METRES * 0.5,
                ),
            ] {
                let element = if cap {
                    BoundarySupportElement::WallCap(index)
                } else {
                    BoundarySupportElement::Wall(index)
                };
                let construct = || {
                    let centre = Position::from_metres(Vec3::new(centre.x, elevation, centre.y))?;
                    Ok(CityBoundaryMember {
                        pose: if cap {
                            CityBoundaryPose::WallCap { index, centre }
                        } else {
                            CityBoundaryPose::Wall { index, centre }
                        },
                        size_metres: CuboidDimensions::from_metres(Vec3::new(
                            length, height, thickness,
                        ))?,
                        orientation,
                        material: CityBoundaryMaterial::Masonry,
                    })
                };
                members
                    .push(construct().map_err(|cause| BoundaryGeometryError { element, cause })?);
            }
        }
        for side in [PropertySide::Left, PropertySide::Right] {
            members.push(self.gate.post(side)?);
        }
        Ok(members)
    }
}
impl CityGate {
    /// Scene X/Z and gate-relative Y; support planning binds the post's head to
    /// the accepted gate datum, with its base embedded in the local surface.
    pub fn post(self, side: PropertySide) -> Result<CityBoundaryMember, BoundaryGeometryError> {
        let centre = self.centre_metres
            + self.orientation.local_to_world(
                Vec2::X * side.sign() * (self.width_metres + GATE_POST_WIDTH_METRES) * 0.5,
            );
        let height = self.height_metres + GATE_POST_HEAD_METRES;
        let construct = || {
            Ok(CityBoundaryMember {
                pose: CityBoundaryPose::GatePost {
                    side,
                    centre: Position::from_metres(Vec3::new(centre.x, height * 0.5, centre.y))?,
                },
                size_metres: CuboidDimensions::from_metres(Vec3::new(
                    GATE_POST_WIDTH_METRES,
                    height,
                    GATE_POST_WIDTH_METRES,
                ))?,
                orientation: self.orientation,
                material: CityBoundaryMaterial::Masonry,
            })
        };
        construct().map_err(|cause| BoundaryGeometryError {
            element: BoundarySupportElement::GatePost(side),
            cause,
        })
    }

    pub fn owns_opening(opening: OpeningAssemblyId) -> bool {
        opening.0 & BOUNDARY_GATE_OPENING_DOMAIN == BOUNDARY_GATE_OPENING_DOMAIN
    }

    pub fn door(
        self,
        property: CityPropertyId,
    ) -> Result<
        DoorSpec<crate::scene_coordinates::GateRelative>,
        adventuresim_building_generator::DoorError,
    > {
        use adventuresim_building_generator::spatial_geometry::{
            LeafDimensions, PlanDirection, Position, Radians,
        };
        let opening = OpeningAssemblyId(BOUNDARY_GATE_OPENING_DOMAIN | property.0);
        let tangent = self.orientation.local_to_world(Vec2::X);
        let outward = self.orientation.local_to_world(-Vec2::Y);
        let leaf_centre = self.centre_metres
            - outward
                * ((GATE_POST_WIDTH_METRES + GATE_LEAF_THICKNESS_METRES) * 0.5
                    + GATE_HINGE_CLEARANCE_METRES);
        let closed_centre = Vec3::new(
            leaf_centre.x,
            self.height_metres * 0.5 + GATE_GROUND_GAP_METRES,
            leaf_centre.y,
        );
        let construct = || {
            Ok(DoorSpec {
                opening,
                source: ResolvedItemId(opening.0),
                closed_centre: Position::from_metres(closed_centre)?,
                hinge_centre: Position::from_metres(
                    closed_centre
                        + Vec3::new(tangent.x, 0.0, tangent.y)
                            * self.width_metres
                            * 0.5
                            * self.hinge.sign(),
                )?,
                size_metres: LeafDimensions::from_metres(Vec3::new(
                    self.width_metres,
                    self.height_metres,
                    GATE_LEAF_THICKNESS_METRES,
                ))?,
                closed_yaw_radians: Radians::new(self.orientation.yaw_radians())?,
                tangent: PlanDirection::from_normalized(tangent)?,
                outward: PlanDirection::from_normalized(outward)?,
                open_angle_radians: Radians::new(self.hinge.sign() * GATE_OPEN_ANGLE_RADIANS)?,
            })
        };
        construct().map_err(|cause| adventuresim_building_generator::DoorError {
            opening,
            source_id: Some(ResolvedItemId(opening.0)),
            cause: adventuresim_building_generator::DoorErrorCause::Geometry(cause),
        })
    }
}
