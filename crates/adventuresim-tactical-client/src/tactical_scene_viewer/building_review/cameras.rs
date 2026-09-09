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
    Sign,
    Mounting,
    Window,
    Exterior,
    Interior,
    /// A measured building-local point for working bays and human-height street views.
    LocalPoint(Vec3),
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
        let target = match self.target {
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
        };
        assert!(
            self.offset.is_finite() && self.offset.length() > 0.1,
            "invalid camera displacement"
        );
        let transform = super::super::buildings::building_transform(building);
        let world_target = transform.transform_point(target - bounds.centre());
        BuildingReviewCamera {
            position: world_target + transform.rotation * self.offset,
            target: world_target,
            plaster_raking_light: None,
        }
    }
}
