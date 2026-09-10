//! A market and three service frontages exercise canonical outdoor placement.
use super::*;
use adventuresim_building_generator::{ServiceBuildingSize, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::FurnitureReview,
        playable_spacing_metres: 20.0,
        ..super::fixture("furniture-review", "city", 47_122, flat, open_yard, clear())
    }
}

fn open_yard(_: f32, _: f32) -> EnvironmentalSample {
    sample(TacticalSurface::Open, 0, 0, 0, 0)
}

pub(super) fn buildings() -> Vec<TacticalBuildingPlacement> {
    [
        (
            BuildingUse::Inn,
            Vec2::new(55.0, 0.0),
            std::f32::consts::FRAC_PI_2,
        ),
        (
            BuildingUse::Warehouse,
            Vec2::new(-55.0, -25.0),
            -std::f32::consts::FRAC_PI_2,
        ),
        (
            BuildingUse::Stable,
            Vec2::new(0.0, 60.0),
            std::f32::consts::PI,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(index, (usage, centre_metres, yaw))| TacticalBuildingPlacement {
            id: index as u64 + 1,
            program: BuildingProgram::validated_settlement(
                settlement_archetype(usage),
                usage,
                42,
                Some(ServiceBuildingSize::Medium),
            )
            .expect("curated furniture review service must validate"),
            centre_metres,
            orientation: BuildingOrientation::from_radians(yaw).unwrap(),
        },
    )
    .collect()
}

pub(super) fn streets() -> Vec<CityStreetPatch> {
    let mut streets = vec![CityStreetPatch::Market {
        corners_metres: corners(Vec2::splat(-25.0), Vec2::splat(25.0)),
        surface: CityStreetSurface::Fieldstone,
    }];
    for (start_metres, end_metres, surface) in [
        (
            Vec2::new(-90.0, -30.0),
            Vec2::new(90.0, -30.0),
            CityStreetSurface::Fieldstone,
        ),
        (
            Vec2::new(-90.0, 30.0),
            Vec2::new(90.0, 30.0),
            CityStreetSurface::Gravel,
        ),
        (
            Vec2::new(-30.0, -90.0),
            Vec2::new(-30.0, 90.0),
            CityStreetSurface::CompactedEarth,
        ),
        (
            Vec2::new(30.0, -90.0),
            Vec2::new(30.0, 90.0),
            CityStreetSurface::Fieldstone,
        ),
    ] {
        streets.push(CityStreetPatch::Corridor {
            start_metres,
            end_metres,
            half_width_metres: 3.5,
            surface,
        });
    }
    streets
}

pub(super) fn yards() -> Vec<CityYardPatch> {
    [
        (Vec2::new(34.0, -20.0), Vec2::new(72.0, 20.0)),
        (Vec2::new(-73.0, -46.0), Vec2::new(-36.0, -5.0)),
        (Vec2::new(-20.0, 39.0), Vec2::new(20.0, 80.0)),
    ]
    .into_iter()
    .map(|(min, max)| CityYardPatch {
        corners_metres: corners(min, max),
        surface: CityYardSurface::PackedEarth,
    })
    .collect()
}

fn corners(min: Vec2, max: Vec2) -> [Vec2; 4] {
    [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)]
}
