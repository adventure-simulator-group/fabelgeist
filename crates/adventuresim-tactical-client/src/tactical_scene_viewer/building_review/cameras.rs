use super::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReviewView {
    pub(super) slug: String,
    building: u64,
    target: ReviewTarget,
    /// Camera displacement from the target, in building-local metres.
    offset: Vec3,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewTarget {
    MainGable(u8),
    Heating(super::heating::HeatingTarget),
    Sign,
    Mounting,
    Window,
    Exterior,
    Interior,
    /// A measured building-local point for working bays and human-height street views.
    LocalPoint(Vec3),
    /// Horizontal offset from the building placement, height above its level pad.
    PlotPoint(Vec3),
}

impl ReviewView {
    pub(super) fn camera(
        &self,
        buildings: &[GeneratedBuilding],
        signs: &BTreeMap<u64, ShopSign>,
    ) -> BuildingReviewCamera {
        let building = buildings
            .iter()
            .find(|b| b.placement.id == self.building)
            .expect("review camera building exists");
        let bounds = building.collision.bounds;
        let gable = self.gable(building);
        let target = self.target(building, signs, gable);
        assert!(
            self.offset.is_finite() && self.offset.length() > 0.1,
            "invalid camera displacement"
        );
        let transform = super::super::buildings::building_transform(building);
        let world_target = transform.transform_point(target - bounds.centre());
        let offset = if let ReviewTarget::Heating(target) = self.target {
            target.offset(&building.plan, self.offset)
        } else {
            gable.map_or(self.offset, |wall| {
                let horizontal =
                    wall.frame.tangent * self.offset.x + wall.frame.outward * self.offset.z;
                Vec3::new(horizontal.x, self.offset.y, horizontal.y)
            })
        };
        BuildingReviewCamera {
            position: world_target + transform.rotation * offset,
            target: world_target,
            plaster_raking_light: None,
        }
    }
    fn gable<'a>(
        &self,
        building: &'a GeneratedBuilding,
    ) -> Option<&'a adventuresim_building_generator::WallAssembly> {
        if let ReviewTarget::MainGable(index) = self.target {
            Some(
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
                    .expect("review requires a main gable aperture"),
            )
        } else {
            None
        }
    }
    fn target(
        &self,
        building: &GeneratedBuilding,
        signs: &BTreeMap<u64, ShopSign>,
        gable: Option<&adventuresim_building_generator::WallAssembly>,
    ) -> Vec3 {
        let bounds = building.collision.bounds;
        match self.target {
            ReviewTarget::Heating(target) => target.anchor(&building.plan),
            ReviewTarget::MainGable(_) => {
                let wall = gable.unwrap();
                let opening = building
                    .plan
                    .opening_assemblies
                    .iter()
                    .find(|opening| opening.host_wall == wall.id)
                    .expect("gable opening");
                Vec3::new(
                    wall.frame.origin.x,
                    opening.sill_elevation_metres + opening.profile.clear_height_metres() * 0.5,
                    wall.frame.origin.y,
                )
            }
            ReviewTarget::PlotPoint(point) => {
                assert!(point.is_finite(), "invalid plot review target");
                Vec3::new(
                    bounds.centre().x + point.x,
                    bounds.min.y + point.y,
                    bounds.centre().z + point.z,
                )
            }
            ReviewTarget::LocalPoint(point) => {
                assert!(point.is_finite(), "invalid local review target");
                point
            }
            ReviewTarget::Sign => {
                let sign = signs
                    .get(&self.building)
                    .expect("sign view requires an authored sign");
                SignSite::for_plan(&building.plan)
                    .expect("sign site")
                    .board(sign.mount)
                    .centre
            }
            ReviewTarget::Mounting => {
                SignSite::for_plan(&building.plan)
                    .expect("mounting site")
                    .mounting
                    .contact
            }
            ReviewTarget::Window => {
                compile_operable_windows(&building.plan)
                    .into_iter()
                    .min_by(|a, b| a.closed_centre.z.total_cmp(&b.closed_centre.z))
                    .expect("window view requires an operable window")
                    .closed_centre
            }
            ReviewTarget::Exterior => bounds.centre(),
            ReviewTarget::Interior => {
                let passage = building
                    .plan
                    .workplace
                    .as_ref()
                    .and_then(|work| work.passages.first())
                    .expect("workplace interior review requires a reserved passage");
                Vec3::new(
                    (passage.min.x + passage.max.x) * 0.5,
                    passage.min.y + 1.65,
                    passage.min.z + 3.0,
                )
            }
        }
    }
}
