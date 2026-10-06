//! Static tactical collision compiled from accepted semantic wall assemblies.
//!
//! Collision deliberately does not consume render LOD triangles. Wall host
//! solids preserve authoritative opening subtraction, while doors and other
//! operable closures remain available for separate dynamic collision entities.

use std::collections::{BTreeMap, BTreeSet};

use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::spatial_geometry::{
    Architectural, CuboidDimensions, GeometryError, GeometryFrame, Position, Radians, SpatialBounds,
};
use crate::{BuildingPlan, ResolvedItemId, ResolvedSolid, compile_window_bars};

mod cuboid;
pub use cuboid::CuboidCorners;
mod footprint;
mod gable;
mod ground_contact;
pub use footprint::GroundFloorFootprint;
pub use ground_contact::GroundContact;
mod admission;
mod intersection;

#[cfg(test)]
#[path = "collision/arch_tests.rs"]
mod arch_tests;

/// An oriented represented cuboid in one declared frame, with nonnegative extents.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(bound = "")]
pub struct CollisionCuboid<F: GeometryFrame> {
    pub source: ResolvedItemId,
    pub centre: Position<F>,
    pub size: CuboidDimensions,
    pub yaw_radians: Radians,
    pub crossfall_radians: Radians,
    pub longfall_radians: Radians,
}

/// Construction failures retain the fixed member's physical identity and cause.
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("collision member {source_id:?}: {cause}")]
pub struct CollisionError {
    pub source_id: ResolvedItemId,
    #[source]
    pub cause: GeometryError,
}
impl<F: GeometryFrame> CollisionCuboid<F> {
    /// Native geometry adapter. Every field is checked before the solid exists.
    pub fn from_metres(
        source: ResolvedItemId,
        centre: Vec3,
        size: Vec3,
        yaw_radians: f32,
        crossfall_radians: f32,
        longfall_radians: f32,
    ) -> Result<Self, CollisionError> {
        let construct = || {
            Ok(Self {
                source,
                centre: Position::from_metres(centre)?,
                size: CuboidDimensions::from_metres(size)?,
                yaw_radians: Radians::new(yaw_radians)?,
                crossfall_radians: Radians::new(crossfall_radians)?,
                longfall_radians: Radians::new(longfall_radians)?,
            })
        };
        construct().map_err(|cause| CollisionError {
            source_id: source,
            cause,
        })
    }
    /// Separately rounded centre/half-extent envelope used by placement. Contact
    /// exclusion must instead use computed corners, which can reach Y=0 even
    /// when this expression rounds its minimum above the datum.
    pub fn bounds(self) -> Result<SpatialBounds<F>, CollisionError> {
        let half = self.size.metres() * 0.5;
        let orientation = bevy::math::Quat::from_rotation_y(self.yaw_radians.radians())
            * bevy::math::Quat::from_rotation_x(self.crossfall_radians.radians())
            * bevy::math::Quat::from_rotation_z(self.longfall_radians.radians());
        let rotated_half = (orientation * Vec3::X).abs() * half.x
            + (orientation * Vec3::Y).abs() * half.y
            + (orientation * Vec3::Z).abs() * half.z;
        SpatialBounds::from_metres(
            self.centre.metres() - rotated_half,
            self.centre.metres() + rotated_half,
        )
        .map_err(|cause| CollisionError {
            source_id: self.source,
            cause,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildingCollision {
    pub bounds: SpatialBounds<Architectural>,
    pub cuboids: Vec<CollisionCuboid<Architectural>>,
}

impl BuildingCollision {
    /// Envelope of exact fixed-solid cross-sections at architectural Y=0.
    /// Tilted members contribute only their floor contact, not projections of
    /// higher geometry. Buried slabs touching the datum remain included.
    /// This bounds a footprint; it does not claim every enclosed point is a
    /// structural bearing. Both vertical bounds are the architectural datum.
    pub fn ground_floor_contact_bounds(
        &self,
    ) -> Result<Option<SpatialBounds<Architectural>>, crate::plan_geometry::PlanGeometryError> {
        let mut bounds: Option<SpatialBounds<Architectural>> = None;
        for solid in &self.cuboids {
            for point in solid.ground_contact()?.points() {
                let point = point.metres();
                let point = Position::from_metres(Vec3::new(point.x, 0.0, point.y))?;
                bounds = Some(match bounds {
                    None => SpatialBounds::at(point),
                    Some(bounds) => bounds.including(point),
                });
            }
        }
        Ok(bounds)
    }
}

/// Compiles static collision from authoritative wall hosts and walkable timber
/// surfaces and fixed main-gable glazing. Operable doors remain separate
/// gameplay entities. This does not add collision for every roof enclosure.
pub fn compile_building_collision(
    plan: &BuildingPlan,
) -> Result<BuildingCollision, CollisionError> {
    let solids = plan
        .resolved_geometry
        .solids
        .iter()
        .map(|solid| (solid.id, solid))
        .collect::<BTreeMap<_, _>>();
    let mut selected = plan
        .wall_assemblies
        .iter()
        .filter(|wall| wall.replaced_by_owner.is_none())
        .flat_map(|wall| wall.host_solids.iter().copied())
        .collect::<BTreeSet<_>>();
    selected.extend(gable::solids(plan));
    if let Some(heating) = &plan.domestic_heating {
        selected.extend(heating.parts.iter().map(|p| p.solid));
    }
    if let Some(frame) = &plan.timber_frame {
        selected.extend(
            frame
                .floors
                .iter()
                .flat_map(|floor| floor.floor_solids.iter().copied()),
        );
        selected.extend(frame.circulation.stair_solids.iter().copied());
        selected.extend(frame.circulation.landing_solids.iter().copied());
    }
    if let Some(workplace) = &plan.workplace {
        selected.extend(
            workplace
                .parts
                .iter()
                .filter(|part| {
                    !matches!(
                        part.feature,
                        crate::WorkplaceFeature::Boarding | crate::WorkplaceFeature::ProcessLiquid
                    )
                })
                .map(|part| part.solid),
        );
    }
    selected.extend(
        plan.resolved_geometry
            .solids
            .iter()
            .filter(|solid| {
                matches!(
                    solid.role,
                    crate::SolidRole::InteriorFloor
                        | crate::SolidRole::ChurchBell
                        | crate::SolidRole::ChurchBellFrame
                        | crate::SolidRole::ChurchBellFitting
                        | crate::SolidRole::ChurchBellAxle
                        | crate::SolidRole::ChurchBellHeadstock
                        | crate::SolidRole::ChurchBellBearing
                        | crate::SolidRole::ChurchBellCrown
                        | crate::SolidRole::ChurchFloor
                        | crate::SolidRole::GalleryFloor
                        | crate::SolidRole::StairTread
                        | crate::SolidRole::StairNewel
                ) || crate::spiral_stairs::owns_landing(solid)
            })
            .map(|solid| solid.id),
    );
    let mut cuboids = selected
        .into_iter()
        .filter_map(|id| solids.get(&id).copied())
        .map(|solid| collision_parts(plan, solid))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    for bar in compile_window_bars(plan)? {
        cuboids.push(CollisionCuboid::from_metres(
            bar.source,
            bar.centre.metres(),
            bar.size_metres.metres(),
            bar.yaw_radians.radians(),
            0.0,
            0.0,
        )?);
    }
    let bounds = collision_bounds(plan, &cuboids)?;
    Ok(BuildingCollision { bounds, cuboids })
}

pub(crate) fn collision_parts(
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
) -> Result<Vec<CollisionCuboid<Architectural>>, CollisionError> {
    if matches!(solid.shape, crate::ResolvedSolidShape::CylinderAlongX) {
        return crate::axle::collision(solid);
    }
    if matches!(solid.shape, crate::ResolvedSolidShape::BellShell) {
        return crate::bell::collision(solid);
    }
    let wall = plan
        .wall_assemblies
        .iter()
        .find(|wall| wall.host_solids.contains(&solid.id));
    let Some(arch) = crate::arch_geometry::ArchGeometry::from_solid(solid, wall) else {
        return Ok(vec![solid.cuboid_envelope()]);
    };
    // Each convex arch section has a conservative box. This keeps the clear
    // crown traversable while retaining masonry above it, with bounded steps
    // along the curved intrados rather than one aperture-blocking host box.
    arch.strips()
        .into_iter()
        .map(|strip| {
            let outward = -strip.depth.normalize();
            let tangent = Vec3::Y.cross(outward);
            let local = |point: Vec3| Vec3::new(point.dot(tangent), point.y, point.dot(outward));
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for point in strip
                .front
                .into_iter()
                .flat_map(|point| [point, point + strip.depth])
            {
                min = min.min(local(point));
                max = max.max(local(point));
            }
            let centre = (min + max) * 0.5;
            CollisionCuboid::from_metres(
                solid.id,
                tangent * centre.x + Vec3::Y * centre.y + outward * centre.z,
                max - min,
                (-tangent.z).atan2(tangent.x),
                0.0,
                0.0,
            )
        })
        .collect()
}

fn collision_bounds(
    plan: &BuildingPlan,
    cuboids: &[CollisionCuboid<Architectural>],
) -> Result<SpatialBounds<Architectural>, CollisionError> {
    let dimensions = plan.dimensions_metres();
    let fallback =
        SpatialBounds::from_metres(Vec3::ZERO, Vec3::new(dimensions.x, 0.0, dimensions.y))
            .map_err(|cause| CollisionError {
                source_id: ResolvedItemId::default(),
                cause,
            })?;
    cuboids.iter().try_fold(
        fallback,
        |bounds, cuboid| Ok(bounds.union(cuboid.bounds()?)),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn ground_contact_excludes_projections_but_keeps_buried_slabs() {
        use super::*;
        let part = |centre, size| {
            CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
                ResolvedItemId(1),
                centre,
                size,
                0.0,
                0.0,
                0.0,
            )
            .unwrap()
        };
        let collision =
            BuildingCollision {
                bounds: crate::spatial_geometry::SpatialBounds::<
                    crate::spatial_geometry::Architectural,
                >::from_metres(Vec3::splat(-10.0), Vec3::splat(10.0))
                .unwrap(),
                cuboids: vec![
                    part(Vec3::new(0.0, -0.08, 0.0), Vec3::new(6.0, 0.16, 8.0)),
                    part(Vec3::new(0.0, 4.0, 0.0), Vec3::new(10.0, 1.0, 12.0)),
                    part(Vec3::new(0.0, -2.0, 0.0), Vec3::splat(1.0)),
                ],
            };
        let contact = collision.ground_floor_contact_bounds().unwrap().unwrap();
        assert_eq!(contact.min().metres(), Vec3::new(-3.0, 0.0, -4.0));
        assert_eq!(contact.max().metres(), Vec3::new(3.0, 0.0, 4.0));
        assert!(collision.bounds.max().metres().x > contact.max().metres().x);
    }

    #[test]
    fn pitched_and_rolled_collision_bounds_contain_all_transformed_corners() {
        use super::*;
        let rotation = bevy::math::Quat::from_euler(bevy::math::EulerRot::YXZ, 0.3, 0.6, 0.25);
        let cuboid = CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
            ResolvedItemId(1),
            Vec3::new(2.0, 1.0, 3.0),
            Vec3::new(0.4, 0.16, 3.0),
            0.3,
            0.6,
            0.25,
        )
        .unwrap();
        let bounds = cuboid.bounds().unwrap();
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let corner = cuboid.centre.metres()
                        + rotation * (cuboid.size.metres() * Vec3::new(x, y, z) * 0.5);
                    minimum = minimum.min(corner);
                    maximum = maximum.max(corner);
                }
            }
        }
        assert!((bounds.min().metres() - minimum).abs().max_element() < 0.001);
        assert!((bounds.max().metres() - maximum).abs().max_element() < 0.001);
        assert!(bounds.max().metres().y - bounds.min().metres().y > cuboid.size.metres().y * 5.0);
    }
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, OpeningUse, generate};

    #[test]
    fn collision_comes_from_wall_hosts_and_preserves_door_voids() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        let collision = compile_building_collision(&plan).unwrap();
        assert!(!collision.cuboids.is_empty());
        let door = plan
            .opening_assemblies
            .iter()
            .find(|opening| opening.use_kind == OpeningUse::Door)
            .expect("town house has a door");
        let door_point = Vec3::new(
            door.frame.origin.x,
            door.sill_elevation_metres + door.profile.clear_height_metres() * 0.45,
            door.frame.origin.y,
        );
        assert!(collision.cuboids.iter().all(|cuboid| {
            let offset = door_point - cuboid.centre.metres();
            let local = bevy::math::Quat::from_rotation_y(-cuboid.yaw_radians.radians()) * offset;
            let half = cuboid.size.metres() * 0.5 - Vec3::splat(0.01);
            local.x.abs() > half.x || local.y.abs() > half.y || local.z.abs() > half.z
        }));
        assert!(
            collision
                .bounds
                .plan_half_extents()
                .unwrap()
                .metres()
                .min_element()
                > 1.0
        );
    }

    #[test]
    fn timber_floors_and_stair_treads_receive_collision() {
        let plan = generate(&BuildingProgram::fixture(
            BuildingArchetype::FachwerkMerchantHouse,
            42,
        ))
        .unwrap();
        let frame = plan.timber_frame.as_ref().expect("merchant house frame");
        let collision = compile_building_collision(&plan).unwrap();
        let sources = collision
            .cuboids
            .iter()
            .map(|cuboid| cuboid.source)
            .collect::<BTreeSet<_>>();

        assert!(
            frame
                .circulation
                .stair_solids
                .iter()
                .all(|id| sources.contains(id))
        );
        assert!(
            frame
                .circulation
                .landing_solids
                .iter()
                .all(|id| sources.contains(id))
        );
        assert!(
            frame
                .floors
                .iter()
                .all(|floor| { floor.floor_solids.iter().all(|id| sources.contains(id)) })
        );
    }

    #[test]
    fn barred_windows_add_permanent_collision_bars() {
        let (plan, bars) = (0..64)
            .find_map(|seed| {
                let plan = generate(&BuildingProgram::fixture(
                    BuildingArchetype::FachwerkMerchantHouse,
                    seed,
                ))
                .ok()?;
                let bars = crate::compile_window_bars(&plan).unwrap();
                (!bars.is_empty()).then_some((plan, bars))
            })
            .expect("seed range contains at least one barred merchant-house window");
        let collision = compile_building_collision(&plan).unwrap();
        let sources = collision
            .cuboids
            .iter()
            .map(|cuboid| cuboid.source)
            .collect::<BTreeSet<_>>();

        assert!(bars.iter().all(|bar| sources.contains(&bar.source)));
    }
}
