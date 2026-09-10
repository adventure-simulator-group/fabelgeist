//! Authored review inputs and read-only capture checks. Rendering belongs to presentation.
use super::capture_state::BuildingReviewCamera;
use adventuresim_building_generator::{
    compile_operable_doors, compile_operable_windows,
    signs::{ShopSign, SignSite},
};
use adventuresim_tactical_core::prelude::GeneratedBuilding;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

mod cameras;
mod openings;
mod readiness;
pub(super) use openings::spawn_openings;
pub(super) use readiness::{BuildingReviewPlugin, ready};
pub(super) const SHOP_PROFILE: &str = "shop-sign-review";
pub(super) const WORKPLACE_PROFILE: &str = "workplace-review";
pub(super) const PARISH_PROFILE: &str = "parish-review";

pub(super) fn is_profile(profile: &str) -> bool {
    matches!(profile, SHOP_PROFILE | WORKPLACE_PROFILE | PARISH_PROFILE)
}

#[derive(Resource, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewFixture {
    signs: BTreeMap<u64, ShopSign>,
    views: Vec<cameras::ReviewView>,
}

#[derive(Resource)]
pub(super) struct ReviewRequirements {
    buildings: usize,
    doors: usize,
    windows: usize,
    signs: BTreeMap<u64, ExpectedSign>,
    output: std::path::PathBuf,
}

struct ExpectedSign {
    sign: ShopSign,
    site: SignSite,
}

impl ReviewRequirements {
    fn for_buildings(buildings: &[GeneratedBuilding], output: &Path) -> Self {
        for building in buildings {
            assert!(
                adventuresim_building_generator::audit_plan(&building.plan).is_empty(),
                "review building must pass the structural audit"
            );
        }
        Self {
            buildings: buildings.len(),
            doors: buildings
                .iter()
                .map(|b| compile_operable_doors(&b.plan).len())
                .sum(),
            windows: buildings
                .iter()
                .map(|b| compile_operable_windows(&b.plan).len())
                .sum(),
            signs: BTreeMap::new(),
            output: output.to_owned(),
        }
    }
}

/// Furnished rooms require the identical production building and opening checks as facade reviews.
pub(super) fn setup_geometry_requirements(
    commands: &mut Commands,
    buildings: &[GeneratedBuilding],
    output: &Path,
) {
    commands.insert_resource(ReviewRequirements::for_buildings(buildings, output));
}

pub(super) fn setup(
    commands: &mut Commands,
    buildings: &[GeneratedBuilding],
    input: &Path,
    output: &Path,
    profile: &str,
) -> Option<Vec<BuildingReviewCamera>> {
    if !is_profile(profile) {
        return None;
    }
    let path = input.with_extension("review.json");
    let bytes =
        std::fs::read(&path).expect("building review requires its checked-in .review.json fixture");
    let fixture: ReviewFixture =
        serde_json::from_slice(&bytes).expect("valid building review inputs");
    let specs = super::selected_capture_views(profile, &[]).expect("known review profile");
    assert_eq!(
        fixture.views.len(),
        specs.len() - 1,
        "fixture must cover every review view"
    );
    for (view, spec) in fixture.views.iter().zip(&specs[1..]) {
        assert_eq!(view.slug, spec.slug);
    }
    let mut requirements = ReviewRequirements::for_buildings(buildings, output);
    for (&id, sign) in &fixture.signs {
        let building = buildings
            .iter()
            .find(|b| b.placement.id == id)
            .expect("sign building exists");
        assert!(
            building
                .placement
                .program
                .usage
                .and_then(adventuresim_building_generator::signs::shop_trade)
                .is_some(),
            "sign requires a public shopfront"
        );
        let site = SignSite::for_plan(&building.plan).expect("review sign has structural support");
        assert!(
            site.supports(&building.plan, sign.mount),
            "review sign must fit its specified mount"
        );
        requirements.signs.insert(
            id,
            ExpectedSign {
                sign: sign.clone(),
                site,
            },
        );
    }
    let cameras = fixture
        .views
        .iter()
        .map(|view| view.camera(buildings, &fixture.signs))
        .collect();
    std::fs::write(output.join("input.review.json"), bytes).expect("copy review provenance");
    commands.insert_resource(requirements);
    commands.insert_resource(fixture);
    Some(cameras)
}

pub(super) fn insert_authored_sign(world: &mut World, entity: Entity, id: u64) {
    let sign = world
        .get_resource::<ReviewFixture>()
        .and_then(|fixture| fixture.signs.get(&id))
        .cloned();
    if let Some(sign) = sign {
        world.entity_mut(entity).insert(sign);
    }
}
