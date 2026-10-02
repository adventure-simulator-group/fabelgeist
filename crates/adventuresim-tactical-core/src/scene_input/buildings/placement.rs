//! The architectural ground floor, rather than the collider bottom, seats a building.
use super::GeneratedBuilding;
use bevy::prelude::{Quat, Transform};

impl GeneratedBuilding {
    /// Transform recentered render and collision geometry into the scene.
    /// Authored plan Y=0 is the floor datum. Floor slabs and footings may extend
    /// below it; those buried parts must not lift the walls or door thresholds.
    pub fn transform(&self) -> Transform {
        Transform::from_xyz(
            self.placement.centre_metres.x,
            self.pad_elevation_metres + self.collision.bounds.centre().y,
            self.placement.centre_metres.y,
        )
        .with_rotation(Quat::from_rotation_y(
            self.placement.orientation.yaw_radians(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_input::{
        BuildingOrientation, GeneratedBuildingRecipe, TacticalBuildingPlacement,
    };
    use adventuresim_building_generator::{
        BuildingArchetype, BuildingProgram, OpeningUse, ServiceBuildingSize,
    };
    use adventuresim_world_schema::settlement_buildings::BuildingUse;
    use bevy::math::{Vec2, Vec3, Vec3Swizzles};

    #[test]
    fn buried_floor_slabs_do_not_raise_bearings_or_thresholds() {
        // Exact occupied recipes of Goslar's reported buildings 304 and 14.
        for (id, archetype, usage, seed, size, base) in [
            (
                304,
                BuildingArchetype::TownHouse,
                BuildingUse::Dwelling,
                6_514_374_187_028_306_242,
                None,
                -2.0,
            ),
            (
                14,
                BuildingArchetype::ParishChurch,
                BuildingUse::ParishChurch,
                42,
                Some(ServiceBuildingSize::Large),
                3.0,
            ),
        ] {
            let program =
                BuildingProgram::validated_settlement(archetype, usage, seed, size).unwrap();
            let recipe = GeneratedBuildingRecipe::generate(program.clone()).unwrap();
            let building = GeneratedBuilding {
                placement: TacticalBuildingPlacement {
                    id,
                    program,
                    centre_metres: Vec2::new(-28.5, 20.37),
                    orientation: BuildingOrientation::from_radians(0.73).unwrap(),
                },
                plan: recipe.plan,
                collision: recipe.collision,
                pad_elevation_metres: base,
            };
            assert!(
                building.collision.bounds.min.y < 0.0,
                "fixture needs a buried slab"
            );
            let origin = building.collision.bounds.centre();
            let transform = building.transform();
            let floor = transform.transform_point(Vec3::new(origin.x, 0.0, origin.z) - origin);
            assert!((floor.y - base).abs() < 0.000_01);
            assert!(floor.xz().distance(building.placement.centre_metres) < 0.000_01);
            let bottom = transform.transform_point(building.collision.bounds.min - origin);
            assert!(bottom.y < floor.y, "legitimate slab remains buried");
            let bearings = building
                .plan
                .resolved_geometry
                .structural_nodes
                .iter()
                .filter(|node| node.grounded && node.position.y.abs() < 0.000_01)
                .collect::<Vec<_>>();
            assert!(!bearings.is_empty());
            for bearing in bearings {
                assert!(
                    (transform.transform_point(bearing.position - origin).y - base).abs()
                        < 0.000_01
                );
            }
            let mut thresholds = 0;
            for door in &building.plan.opening_assemblies {
                if matches!(door.use_kind, OpeningUse::Door | OpeningUse::Gate)
                    && door.frame.outside_room.is_none()
                    && door.sill_elevation_metres.abs() < 0.000_01
                {
                    let sill = transform.transform_point(
                        Vec3::new(
                            door.frame.origin.x,
                            door.sill_elevation_metres,
                            door.frame.origin.y,
                        ) - origin,
                    );
                    assert!((sill.y - floor.y).abs() < 0.000_01);
                    thresholds += 1;
                }
            }
            assert!(thresholds > 0);
        }
    }
}
