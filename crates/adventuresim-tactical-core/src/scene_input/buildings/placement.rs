//! The architectural ground floor, rather than the collider bottom, seats a building.
use super::GeneratedBuilding;
use bevy::prelude::Transform;

impl GeneratedBuilding {
    /// Transform recentered render and collision geometry into the scene.
    /// Authored plan Y=0 is the floor datum. Floor slabs and footings may extend
    /// below it; those buried parts must not lift the walls or door thresholds.
    pub fn geometry_datum(
        &self,
    ) -> Result<
        crate::scene_coordinates::CollisionCentreDatum,
        adventuresim_building_generator::spatial_geometry::GeometryError,
    > {
        use crate::scene_coordinates::{ArchitecturalFloorDatum, ArchitecturalPlanProjection};
        let origin = self.collision.bounds.centre()?;
        let plan =
            ArchitecturalPlanProjection::from_placement(&self.placement, self.collision.bounds)?;
        let floor = self.placement.base_elevation_metres;
        ArchitecturalFloorDatum { plan, floor }.collision_centre(origin)
    }
    pub fn transform(
        &self,
    ) -> Result<Transform, adventuresim_building_generator::spatial_geometry::GeometryError> {
        Ok(self.geometry_datum()?.native_transform())
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
    fn distant_promotion_and_preparation_preserve_the_bound_floor_and_program() {
        use crate::scene_input::{DistantBuildingPlacement, GeneratedBuildingRecipes};
        use adventuresim_world_schema::ProsperityTier;
        for floor in [-4.75, 9.5] {
            let distant = DistantBuildingPlacement {
                id: (1238).into(),
                prosperity: ProsperityTier::Wealthy,
                archetype: BuildingArchetype::FachwerkMerchantHouse,
                usage: Some(BuildingUse::Dwelling),
                service_size: None,
                seed: 7_989_866_213_631_017_260.into(),
                centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                    -321.574_13,
                    -345.572_57,
                ))
                .unwrap(),
                base_elevation_metres:
                    crate::city_layout::grounding::SupportElevation::from_metres(floor).unwrap(),
                orientation: BuildingOrientation::from_radians(-0.197_395_56).unwrap(),
            };
            let placement = TacticalBuildingPlacement::from(distant);
            assert_eq!(placement.program, distant.occupied_program());
            assert_eq!(
                placement.centre_metres.metres(),
                distant.centre_metres.metres()
            );
            assert_eq!(placement.orientation, distant.orientation);
            assert_eq!(placement.id, distant.id);
            assert_eq!(placement.base_elevation_metres.metres(), floor);
            let generated = super::super::prepare_buildings(
                std::slice::from_ref(&placement),
                &mut GeneratedBuildingRecipes::default(),
            )
            .unwrap();
            assert_eq!(generated[0].placement, placement);
            let origin = generated[0].collision.bounds.centre().unwrap().metres();
            let world_floor = generated[0]
                .transform()
                .unwrap()
                .transform_point(-Vec3::Y * origin.y);
            assert!((world_floor.y - floor).abs() < 0.000_01);
            assert!(
                crate::city_layout::grounding::SupportElevation::from_metres(f32::NAN).is_none()
            );
        }
    }

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
            let program = BuildingProgram::validated_settlement(
                archetype,
                usage,
                fabelgeist_determinism::Seed::from_u64(seed),
                size,
            )
            .unwrap();
            let recipe = GeneratedBuildingRecipe::generate(program.clone()).unwrap();
            let building = GeneratedBuilding {
                placement: TacticalBuildingPlacement {
                    base_elevation_metres:
                        crate::city_layout::grounding::SupportElevation::from_metres(base).unwrap(),
                    id: (id).into(),
                    program,
                    centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                        -28.5, 20.37,
                    ))
                    .unwrap(),
                    orientation: BuildingOrientation::from_radians(0.73).unwrap(),
                },
                plan: recipe.plan,
                collision: recipe.collision,
            };
            assert!(
                building.collision.bounds.min().metres().y < 0.0,
                "fixture needs a buried slab"
            );
            let origin = building.collision.bounds.centre().unwrap().metres();
            let transform = building.transform().unwrap();
            let floor = transform.transform_point(Vec3::new(origin.x, 0.0, origin.z) - origin);
            assert!((floor.y - base).abs() < 0.000_01);
            assert!(
                floor
                    .xz()
                    .distance(building.placement.centre_metres.metres())
                    < 0.000_01
            );
            let bottom =
                transform.transform_point(building.collision.bounds.min().metres() - origin);
            assert!(bottom.y < floor.y, "legitimate slab remains buried");
            let bearings = building
                .plan
                .resolved_geometry
                .structural_nodes
                .iter()
                .filter(|node| node.grounded && node.position.metres().y.abs() < 0.000_01)
                .collect::<Vec<_>>();
            assert!(!bearings.is_empty());
            for bearing in bearings {
                assert!(
                    (transform
                        .transform_point(bearing.position.metres() - origin)
                        .y
                        - base)
                        .abs()
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
