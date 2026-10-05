//! Dynamic door leaves compiled from accepted opening assemblies.

use std::collections::BTreeMap;

use bevy::math::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{
    BuildingPlan, ClosureKind, ClosureState, OpeningAssemblyId, OpeningUse, ResolvedItemId,
    ResolvedSolid,
};

use crate::spatial_geometry::{
    Architectural, GeometryError, GeometryFrame, LeafDimensions, PlanDirection, Position, Radians,
};

const EXTERIOR_DOOR_OPEN_ANGLE_RADIANS: f32 = 100.0 * core::f32::consts::PI / 180.0;

/// A single operable exterior leaf in building-local coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(bound = "")]
pub struct DoorSpec<F: GeometryFrame> {
    pub opening: OpeningAssemblyId,
    pub source: ResolvedItemId,
    pub closed_centre: Position<F>,
    pub hinge_centre: Position<F>,
    /// Collider and render dimensions in leaf-local X/Y/Z axes.
    pub size_metres: LeafDimensions,
    pub closed_yaw_radians: Radians,
    pub tangent: PlanDirection<F>,
    pub outward: PlanDirection<F>,
    /// Signed relative yaw. Its sign is selected so the leaf always enters the room.
    pub open_angle_radians: Radians,
}

impl<'de, F: GeometryFrame> Deserialize<'de> for DoorSpec<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct SerializedLeaf {
            opening: OpeningAssemblyId,
            source: ResolvedItemId,
            closed_centre: Vec3,
            hinge_centre: Vec3,
            size_metres: Vec3,
            closed_yaw_radians: f32,
            tangent: Vec2,
            outward: Vec2,
            open_angle_radians: f32,
        }
        let value = SerializedLeaf::deserialize(deserializer)?;
        let construct = || {
            Ok(Self {
                opening: value.opening,
                source: value.source,
                closed_centre: Position::from_metres(value.closed_centre)?,
                hinge_centre: Position::from_metres(value.hinge_centre)?,
                size_metres: LeafDimensions::from_metres(value.size_metres)?,
                closed_yaw_radians: Radians::new(value.closed_yaw_radians)?,
                tangent: PlanDirection::from_normalized(value.tangent)?,
                outward: PlanDirection::from_normalized(value.outward)?,
                open_angle_radians: Radians::new(value.open_angle_radians)?,
            })
        };
        construct().map_err(|cause| {
            serde::de::Error::custom(DoorError {
                opening: value.opening,
                source_id: Some(value.source),
                cause: DoorErrorCause::Geometry(cause),
            })
        })
    }
}

impl<F: GeometryFrame> DoorSpec<F> {
    /// Furthest horizontal leaf corner from the hinge. Rotation preserves this
    /// radius, including a hinge offset from the edge or centre of the leaf.
    pub fn horizontal_sweep_radius_metres(self) -> Result<f32, DoorError> {
        let rotation = Quat::from_rotation_y(self.closed_yaw_radians.radians());
        let arm = self.closed_centre.metres() - self.hinge_centre.metres();
        let radius = [-1.0, 1.0]
            .into_iter()
            .flat_map(|x| [-1.0, 1.0].map(|z| (x, z)))
            .map(|(x, z)| {
                let point = arm
                    + rotation
                        * Vec3::new(
                            x * self.size_metres.metres().x * 0.5,
                            0.0,
                            z * self.size_metres.metres().z * 0.5,
                        );
                let radius = Vec2::new(point.x, point.z).length();
                if !point.is_finite() || !radius.is_finite() {
                    return Err(DoorError {
                        opening: self.opening,
                        source_id: Some(self.source),
                        cause: DoorErrorCause::Geometry(GeometryError::NonFinite {
                            role: crate::spatial_geometry::GeometryRole::SweepRadius,
                            axis: crate::spatial_geometry::CoordinateAxis::X,
                        }),
                    });
                }
                Ok(radius)
            })
            .try_fold(0.0_f32, |radius, corner| Ok(radius.max(corner?)))?;
        if !radius.is_finite() {
            return Err(DoorError {
                opening: self.opening,
                source_id: Some(self.source),
                cause: DoorErrorCause::Geometry(GeometryError::NonFinite {
                    role: crate::spatial_geometry::GeometryRole::SweepRadius,
                    axis: crate::spatial_geometry::CoordinateAxis::X,
                }),
            });
        }
        Ok(radius)
    }
}

/// Compiles independently simulated leaves for operable exterior doors.
pub fn compile_operable_doors(
    plan: &BuildingPlan,
) -> Result<Vec<DoorSpec<Architectural>>, DoorError> {
    let solids = plan
        .resolved_geometry
        .solids
        .iter()
        .map(|solid| (solid.id, solid))
        .collect::<BTreeMap<_, _>>();

    operable_openings(plan)
        .map(|opening| {
            let solid = opening
                .closure_solids
                .iter()
                .find_map(|id| solids.get(id).copied())
                .ok_or(DoorError {
                    opening: opening.id,
                    source_id: None,
                    cause: DoorErrorCause::MissingClosure,
                })?;
            door_from_solid(
                opening.id,
                opening.frame.tangent,
                opening.frame.outward,
                solid,
            )
        })
        .collect()
}

pub(crate) fn operable_openings(
    plan: &BuildingPlan,
) -> impl Iterator<Item = &crate::OpeningAssembly> {
    plan.opening_assemblies.iter().filter(|opening| {
        opening.use_kind == OpeningUse::Door
            && opening.closure.state == ClosureState::Operable
            && opening.closure.layers.contains(&ClosureKind::DoorLeaf)
            && opening.frame.inside_room.is_some()
            && opening.frame.outside_room.is_none()
    })
}

fn door_from_solid(
    opening: OpeningAssemblyId,
    tangent: Vec2,
    outward: Vec2,
    solid: &ResolvedSolid,
) -> Result<DoorSpec<Architectural>, DoorError> {
    let construct = || {
        let tangent_owner = PlanDirection::from_vector(tangent)?;
        let outward_owner = PlanDirection::from_vector(outward)?;
        let tangent = tangent_owner.vector();
        let outward = outward_owner.vector();
        let width =
            tangent.x.abs() * solid.size.metres().x + tangent.y.abs() * solid.size.metres().z;
        let thickness =
            outward.x.abs() * solid.size.metres().x + outward.y.abs() * solid.size.metres().z;
        let hinge_centre =
            solid.centre.metres() - Vec3::new(tangent.x, 0.0, tangent.y) * width * 0.5;
        let positive_swing = Quat::from_rotation_y(0.01) * Vec3::new(tangent.x, 0.0, tangent.y);
        let enters_room = Vec2::new(positive_swing.x, positive_swing.z).dot(-outward) > 0.0;
        Ok(DoorSpec {
            opening,
            source: solid.id,
            closed_centre: Position::from_metres(solid.centre.metres())?,
            hinge_centre: Position::from_metres(hinge_centre)?,
            size_metres: LeafDimensions::from_metres(Vec3::new(
                width,
                solid.size.metres().y,
                thickness,
            ))?,
            closed_yaw_radians: Radians::new(-tangent.y.atan2(tangent.x))?,
            tangent: tangent_owner,
            outward: outward_owner,
            open_angle_radians: Radians::new(if enters_room {
                EXTERIOR_DOOR_OPEN_ANGLE_RADIANS
            } else {
                -EXTERIOR_DOOR_OPEN_ANGLE_RADIANS
            })?,
        })
    };
    construct().map_err(|cause| DoorError {
        opening,
        source_id: Some(solid.id),
        cause: DoorErrorCause::Geometry(cause),
    })
}

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq, Serialize)]
#[error("door {opening:?} member {source_id:?}: {cause}")]
pub struct DoorError {
    pub opening: OpeningAssemblyId,
    pub source_id: Option<ResolvedItemId>,
    #[source]
    pub cause: DoorErrorCause,
}
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq, Serialize)]
pub enum DoorErrorCause {
    #[error("operable opening has no resolved closure")]
    MissingClosure,
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, generate};

    #[test]
    fn town_house_exterior_doors_compile_as_inward_swinging_cuboids() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        let doors = compile_operable_doors(&plan).unwrap();

        assert!(!doors.is_empty());
        for door in doors {
            assert!(door.size_metres.metres().cmpgt(Vec3::ZERO).all());
            let closed_arm = Vec3::new(door.tangent.vector().x, 0.0, door.tangent.vector().y);
            let open_arm = Quat::from_rotation_y(door.open_angle_radians.radians()) * closed_arm;
            assert!(Vec2::new(open_arm.x, open_arm.z).dot(-door.outward.vector()) > 0.0);
        }
    }

    #[test]
    fn sweep_radius_bounds_rotated_leaf_corners_with_offset_hinges() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        for mut door in compile_operable_doors(&plan).unwrap() {
            door.hinge_centre = door
                .hinge_centre
                .translated(
                    crate::spatial_geometry::Displacement::from_metres(Vec3::new(0.03, 0.0, -0.02))
                        .unwrap(),
                )
                .unwrap();
            let radius = door.horizontal_sweep_radius_metres().unwrap();
            let mut furthest = 0.0_f32;
            for angle in [0.0, 0.3, door.open_angle_radians.radians()] {
                let pivot = Quat::from_rotation_y(angle);
                let leaf = Quat::from_rotation_y(door.closed_yaw_radians.radians() + angle);
                let centre = door.hinge_centre.metres()
                    + pivot * (door.closed_centre.metres() - door.hinge_centre.metres());
                for x in [-1.0, 1.0] {
                    for z in [-1.0, 1.0] {
                        let point = centre
                            + leaf
                                * Vec3::new(
                                    x * door.size_metres.metres().x * 0.5,
                                    0.0,
                                    z * door.size_metres.metres().z * 0.5,
                                );
                        let distance = Vec2::new(
                            point.x - door.hinge_centre.metres().x,
                            point.z - door.hinge_centre.metres().z,
                        )
                        .length();
                        assert!(distance <= radius + 0.000_01);
                        furthest = furthest.max(distance);
                    }
                }
            }
            assert!((radius - furthest).abs() < 0.000_01);
        }
    }
}

#[cfg(test)]
mod admission_tests;
