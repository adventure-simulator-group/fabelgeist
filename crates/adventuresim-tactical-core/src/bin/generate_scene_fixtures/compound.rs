//! Two complete properties demonstrating both independent access orientations.
use super::*;
use adventuresim_tactical_core::city_layout::{
    CityPropertyId, CitySceneLayout, CompiledCityLayout, MAX_CITY_LOTS, PropertySide,
};

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
    let city = CitySite::central_german_market_town()
        .generate(
            42,
            900,
            &adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder(),
        )
        .compile(42)
        .expect("review city compiles");
    let mut layout = CitySceneLayout::default();
    for (id, side, offset) in [
        (1, PropertySide::Left, Vec2::new(-18.0, 0.0)),
        (2, PropertySide::Right, Vec2::new(18.0, 0.0)),
    ] {
        let property = isolate(&city, id, side, offset);
        layout.playable.extend(property.playable);
        layout.compounds.extend(property.compounds);
        layout.yards.extend(property.yards);
    }
    layout.streets.push(CityStreetPatch::Corridor {
        start_metres: Vec2::new(-60.0, -11.0),
        end_metres: Vec2::new(60.0, -11.0),
        half_width_metres: 3.5,
        surface: CityStreetSurface::CompactedEarth,
    });
    layout
}

fn isolate(
    city: &CompiledCityLayout,
    id: u64,
    passage: PropertySide,
    offset: Vec2,
) -> CitySceneLayout {
    let mut compound = city
        .compounds
        .iter()
        .find(|property| property.boundary.gate.hinge == passage.opposite())
        .expect("review city includes both passage orientations")
        .clone();
    let front = city
        .buildings
        .iter()
        .find(|b| b.id == compound.front_building_id)
        .unwrap();
    let origin = front.centre_metres;
    let orientation = front.orientation;
    let local = |point| orientation.world_to_local(point - origin) + offset;
    let mut playable = city
        .buildings
        .iter()
        .filter(|b| [compound.front_building_id, compound.rear_building_id].contains(&b.id))
        .cloned()
        .collect::<Vec<_>>();
    for building in &mut playable {
        building.id = if building.id == compound.front_building_id {
            id
        } else {
            MAX_CITY_LOTS as u64 + id
        };
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
    compound.id = CityPropertyId(id);
    compound.front_building_id = id;
    compound.rear_building_id = MAX_CITY_LOTS as u64 + id;
    let yards = vec![CityYardPatch {
        corners_metres: compound.plot.corners(),
        surface: CityYardSurface::PackedEarth,
    }];
    CitySceneLayout {
        playable,
        yards,
        parishes: Vec::new(),
        compounds: vec![compound],
        ..Default::default()
    }
}
