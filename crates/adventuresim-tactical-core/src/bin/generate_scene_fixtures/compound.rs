//! A complete generated merchant property, isolated for street and court review.
use super::*;
use adventuresim_tactical_core::city_layout::CitySceneLayout;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::CompoundReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "compound-review",
            "city",
            47_124,
            flat,
            |_, _| sample(TacticalSurface::Open, 0, 0, 0, 0),
            clear(),
        )
    }
}

pub(super) fn layout() -> CitySceneLayout {
    let mut city = CitySite::central_german_market_town()
        .generate(
            42,
            900,
            &adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder(),
        )
        .compile(42)
        .expect("review city compiles");
    let mut compound = city.compounds.remove(0);
    let front = city
        .buildings
        .iter()
        .find(|b| b.id == compound.front_building_id)
        .unwrap();
    let origin = front.centre_metres;
    let orientation = front.orientation;
    let local = |point| orientation.world_to_local(point - origin);
    let mut playable = city
        .buildings
        .into_iter()
        .filter(|b| [compound.front_building_id, compound.rear_building_id].contains(&b.id))
        .collect::<Vec<_>>();
    for building in &mut playable {
        building.centre_metres = local(building.centre_metres);
        building.orientation = BuildingOrientation::IDENTITY;
    }
    for bounds in [&mut compound.plot, &mut compound.court] {
        bounds.centre_metres = local(bounds.centre_metres);
        bounds.orientation = BuildingOrientation::IDENTITY;
    }
    for access in &mut compound.access {
        access.start_metres = local(access.start_metres);
        access.end_metres = local(access.end_metres);
    }
    for wall in &mut compound.boundary.walls {
        wall.start_metres = local(wall.start_metres);
        wall.end_metres = local(wall.end_metres);
    }
    compound.boundary.gate.centre_metres = local(compound.boundary.gate.centre_metres);
    compound.boundary.gate.orientation = BuildingOrientation::IDENTITY;
    let yards = vec![CityYardPatch {
        corners_metres: compound.plot.corners(),
        surface: CityYardSurface::PackedEarth,
    }];
    CitySceneLayout {
        playable,
        yards,
        parishes: Vec::new(),
        compounds: vec![compound],
        streets: vec![CityStreetPatch::Corridor {
            start_metres: Vec2::new(-45.0, -11.0),
            end_metres: Vec2::new(45.0, -11.0),
            half_width_metres: 3.5,
            surface: CityStreetSurface::CompactedEarth,
        }],
        ..Default::default()
    }
}
