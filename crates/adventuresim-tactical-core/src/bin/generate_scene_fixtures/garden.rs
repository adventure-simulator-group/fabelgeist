//! An actual accepted household garden, normalized for repeatable close review.
use super::*;
use adventuresim_tactical_core::city_layout::CitySceneLayout;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::GardenReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "garden-review",
            "city",
            47_127,
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
        .expect("garden review city compiles");
    let mut garden = city.gardens.remove(0);
    let mut front = city
        .buildings
        .into_iter()
        .find(|b| b.id == garden.front_building_id)
        .unwrap();
    let origin = front.centre_metres;
    let orientation = front.orientation;
    let local = |p| orientation.world_to_local(p - origin);
    front.centre_metres = Vec2::ZERO;
    front.orientation = BuildingOrientation::IDENTITY;
    for bounds in std::iter::once(&mut garden.plot)
        .chain(std::iter::once(&mut garden.cultivated_bounds))
        .chain(garden.beds.iter_mut())
    {
        bounds.centre_metres = local(bounds.centre_metres);
        bounds.orientation = BuildingOrientation::IDENTITY;
    }
    for access in &mut garden.access {
        access.start_metres = local(access.start_metres);
        access.end_metres = local(access.end_metres);
    }
    for plant in &mut garden.plants {
        plant.centre_metres = local(plant.centre_metres);
        plant.orientation = BuildingOrientation::IDENTITY;
    }
    let mut yards = vec![CityYardPatch {
        corners_metres: garden.plot.corners(),
        surface: CityYardSurface::PackedEarth,
    }];
    yards.extend(garden.beds.iter().map(|b| CityYardPatch {
        corners_metres: b.corners(),
        surface: CityYardSurface::KitchenGarden,
    }));
    let street_depth = garden.access[0].start_metres.y;
    let offset = Vec2::new(0.0, 70.0);
    let mut distant_garden = garden.clone();
    distant_garden.owner = adventuresim_tactical_core::city_layout::CityPropertyId(10_032);
    distant_garden.front_building_id = distant_garden.owner.0;
    for bounds in std::iter::once(&mut distant_garden.plot)
        .chain(std::iter::once(&mut distant_garden.cultivated_bounds))
        .chain(distant_garden.beds.iter_mut())
    {
        bounds.centre_metres += offset;
    }
    for route in &mut distant_garden.access {
        route.start_metres += offset;
        route.end_metres += offset;
    }
    for plant in &mut distant_garden.plants {
        plant.id = adventuresim_tactical_core::city_layout::GardenPlantId(10_032);
        plant.centre_metres += offset;
    }
    let mut distant_yards = yards.clone();
    for yard in &mut distant_yards {
        for point in &mut yard.corners_metres {
            *point += offset;
        }
    }
    yards.extend(distant_yards);
    let distant = DistantBuildingPlacement {
        id: distant_garden.front_building_id,
        archetype: front.program.archetype,
        usage: front.program.usage,
        service_size: front.program.service_size,
        seed: front.program.seed,
        centre_metres: offset,
        orientation: front.orientation,
        base_elevation_metres: 0.0,
    };
    CitySceneLayout {
        playable: vec![front],
        distant: vec![distant],
        gardens: vec![garden, distant_garden],
        yards,
        streets: [0.0, offset.y]
            .map(|depth| CityStreetPatch::Corridor {
                start_metres: Vec2::new(-45.0, street_depth + depth),
                end_metres: Vec2::new(45.0, street_depth + depth),
                half_width_metres: 3.5,
                surface: CityStreetSurface::CompactedEarth,
            })
            .to_vec(),
        ..Default::default()
    }
}
