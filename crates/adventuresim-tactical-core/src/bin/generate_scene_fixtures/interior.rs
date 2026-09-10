//! Flat review sites retain ordinary settlement recipes and production terrain.
use super::*;
use adventuresim_building_generator::{ServiceBuildingSize, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

pub(super) fn catalog_fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::InteriorFurnitureCatalog,
        playable_spacing_metres: 30.0,
        ..super::fixture(
            "interior-furniture-catalog",
            "grassland",
            47_123,
            flat,
            open_yard,
            clear(),
        )
    }
}

pub(super) fn rooms_fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::InteriorFurnitureRooms,
        playable_spacing_metres: 30.0,
        ..super::fixture(
            "interior-furniture-rooms",
            "grassland",
            47_124,
            flat,
            open_yard,
            clear(),
        )
    }
}

fn open_yard(_: f32, _: f32) -> EnvironmentalSample {
    sample(TacticalSurface::Open, 0, 0, 0, 0)
}

/// Stable IDs identify each building in both cutaway captures and layout proofs.
pub(super) fn buildings() -> Vec<TacticalBuildingPlacement> {
    [
        BuildingUse::Dwelling,
        BuildingUse::Inn,
        BuildingUse::GeneralShop,
        BuildingUse::Smithy,
        BuildingUse::ParishChurch,
        BuildingUse::Hospital,
        BuildingUse::Guardhouse,
        BuildingUse::Warehouse,
        BuildingUse::Castle,
        BuildingUse::Cathedral,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, usage)| TacticalBuildingPlacement {
        id: index as u64 + 1,
        program: BuildingProgram::validated_settlement(
            settlement_archetype(usage),
            usage,
            42,
            Some(ServiceBuildingSize::Medium),
        )
        .expect("curated interior review settlement recipe must validate"),
        centre_metres: match usage {
            BuildingUse::Castle => Vec2::new(0.0, 90.0),
            BuildingUse::Cathedral => Vec2::new(0.0, -90.0),
            _ => Vec2::new(
                (index % 4) as f32 * 50.0 - 75.0,
                (index / 4) as f32 * 80.0 - 40.0,
            ),
        },
        orientation: BuildingOrientation::IDENTITY,
    })
    .collect()
}

/// The catalog's five-by-seven pairs and ten building samples fit this flat yard.
pub(super) fn yards() -> Vec<CityYardPatch> {
    vec![CityYardPatch {
        corners_metres: [
            Vec2::new(-120.0, -120.0),
            Vec2::new(120.0, -120.0),
            Vec2::new(120.0, 120.0),
            Vec2::new(-120.0, 120.0),
        ],
        surface: CityYardSurface::PackedEarth,
    }]
}
