use super::*;
use adventuresim_building_generator::spatial_geometry::{Architectural, Displacement, Position};
use adventuresim_tactical_core::scene_coordinates::PlotRelative;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReviewView {
    pub(super) slug: String,
    building: u64,
    target: ReviewTarget,
    /// Camera displacement from the target, in building-local metres.
    offset: Displacement<Architectural>,
    #[serde(default)]
    pub(super) openings: super::openings::OpeningPose,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewTarget {
    MainGable(u8),
    Heating(super::heating::HeatingTarget),
    Opening(super::openings::OpeningTarget),
    Sign,
    Mounting,
    Window,
    Exterior,
    Interior,
    /// A measured building-local point for working bays and human-height street views.
    LocalPoint(Position<Architectural>),
    /// Horizontal offset from the building placement, height above its level pad.
    PlotPoint(Position<PlotRelative>),
}

impl ReviewView {
    pub(super) fn camera(
        &self,
        buildings: &[GeneratedBuilding],
        signs: &BTreeMap<u64, ShopSign>,
    ) -> Result<BuildingReviewCamera> {
        let building = buildings
            .iter()
            .find(|b| b.placement.id == self.building)
            .ok_or_else(|| format!("review building {} is absent", self.building))?;
        let gable = self.gable(building)?;
        let target = self.target(building, signs, gable)?;
        let authored_offset = self.offset.metres();
        if authored_offset.length() <= 0.1 {
            return Err("review camera needs a nonzero viewing displacement".into());
        }
        let datum = building.geometry_datum()?;
        let world_target = datum.point(target)?;
        let offset = if let ReviewTarget::Opening(target) = self.target {
            let frame = target.frame(&building.plan)?;
            let tangent = frame.tangent.vector();
            let outward = frame.outward.vector();
            let horizontal = tangent * authored_offset.x + outward * authored_offset.z;
            Vec3::new(horizontal.x, authored_offset.y, horizontal.y)
        } else if let ReviewTarget::Heating(target) = self.target {
            target.offset(&building.plan, self.offset)?.metres()
        } else {
            gable.map_or(authored_offset, |wall| {
                let horizontal =
                    wall.frame.tangent * authored_offset.x + wall.frame.outward * authored_offset.z;
                Vec3::new(horizontal.x, authored_offset.y, horizontal.y)
            })
        };
        Ok(BuildingReviewCamera {
            position: world_target
                .translated(datum.displacement(Displacement::from_metres(offset)?)?)?
                .metres(),
            target: world_target.metres(),
            plaster_raking_light: None,
        })
    }
    fn gable<'a>(
        &self,
        building: &'a GeneratedBuilding,
    ) -> Result<Option<&'a adventuresim_building_generator::WallAssembly>> {
        if let ReviewTarget::MainGable(index) = self.target {
            Ok(Some(
                building
                    .plan
                    .wall_assemblies
                    .iter()
                    .filter(|wall| {
                        matches!(
                            wall.source,
                            adventuresim_building_generator::WallSourceId::RoofGable { .. }
                        )
                    })
                    .nth(usize::from(index))
                    .ok_or("review main gable aperture is absent")?,
            ))
        } else {
            Ok(None)
        }
    }
    fn target(
        &self,
        building: &GeneratedBuilding,
        signs: &BTreeMap<u64, ShopSign>,
        gable: Option<&adventuresim_building_generator::WallAssembly>,
    ) -> Result<Position<Architectural>> {
        let bounds = building.collision.bounds;
        let point = match self.target {
            ReviewTarget::Opening(target) => target.frame(&building.plan)?.centre.metres(),
            ReviewTarget::Heating(target) => target.anchor(&building.plan)?.metres(),
            ReviewTarget::MainGable(_) => {
                let wall = gable.ok_or("review gable wall is absent")?;
                let opening = building
                    .plan
                    .opening_assemblies
                    .iter()
                    .find(|opening| opening.host_wall == wall.id)
                    .ok_or("review gable opening is absent")?;
                Vec3::new(
                    wall.frame.origin.x,
                    opening.sill_elevation_metres + opening.profile.clear_height_metres() * 0.5,
                    wall.frame.origin.y,
                )
            }
            ReviewTarget::PlotPoint(point) => {
                let point = point.metres();
                Vec3::new(
                    bounds.centre()?.metres().x + point.x,
                    point.y,
                    bounds.centre()?.metres().z + point.z,
                )
            }
            ReviewTarget::LocalPoint(point) => point.metres(),
            ReviewTarget::Sign => {
                let sign = signs.get(&self.building).ok_or("review sign is absent")?;
                SignSite::for_plan(&building.plan)
                    .ok_or("review sign site is absent")?
                    .board(sign.mount)
                    .centre
            }
            ReviewTarget::Mounting => {
                SignSite::for_plan(&building.plan)
                    .ok_or("review mounting site is absent")?
                    .mounting
                    .contact
            }
            ReviewTarget::Window => {
                compile_operable_windows(&building.plan)
                    .into_iter()
                    .min_by(|a, b| a.closed_centre.z.total_cmp(&b.closed_centre.z))
                    .ok_or("review operable window is absent")?
                    .closed_centre
            }
            ReviewTarget::Exterior => bounds.centre()?.metres(),
            ReviewTarget::Interior => {
                let passage = building
                    .plan
                    .workplace
                    .as_ref()
                    .and_then(|work| work.passages.first())
                    .ok_or("review workplace passage is absent")?;
                Vec3::new(
                    (passage.bounds.min().metres().x + passage.bounds.max().metres().x) * 0.5,
                    passage.bounds.min().metres().y + 1.65,
                    passage.bounds.min().metres().z + 3.0,
                )
            }
        };
        Ok(Position::from_metres(point)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::{BuildingArchetype, BuildingProgram};
    use adventuresim_tactical_core::scene_input::{
        BuildingOrientation, GeneratedBuildingRecipe, TacticalBuildingPlacement,
    };
    use adventuresim_world_schema::settlement_buildings::BuildingUse;

    #[test]
    fn plot_camera_height_is_above_architectural_ground_not_buried_slab() {
        let program = BuildingProgram::validated_settlement(
            BuildingArchetype::TownHouse,
            BuildingUse::Dwelling,
            6_514_374_187_028_306_242,
            None,
        )
        .unwrap();
        let recipe = GeneratedBuildingRecipe::generate(program.clone()).unwrap();
        assert!(recipe.collision.bounds.min().metres().y < 0.0);
        let building = GeneratedBuilding {
            placement: TacticalBuildingPlacement {
                base_elevation_metres: -2.0,
                id: 304,
                program,
                centre_metres: Vec2::new(9.5, -8.5),
                orientation: BuildingOrientation::from_radians(0.73).unwrap(),
            },
            plan: recipe.plan,
            collision: recipe.collision,
        };
        let view = ReviewView {
            slug: "floor-contact".into(),
            building: 304,
            target: ReviewTarget::PlotPoint(
                Position::from_metres(Vec3::new(0.0, 0.05, -7.5)).unwrap(),
            ),
            offset: Displacement::from_metres(Vec3::new(0.0, 0.6, -2.0)).unwrap(),
            openings: default(),
        };
        let camera = view.camera(&[building], &BTreeMap::new()).unwrap();
        assert!((camera.target.y - (-1.95)).abs() < 0.000_01);
        assert!((camera.position.y - (-1.35)).abs() < 0.000_01);
    }
}
