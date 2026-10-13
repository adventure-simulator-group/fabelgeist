//! A market and three service frontages exercise canonical outdoor placement.
use super::*;
use adventuresim_building_generator::{ServiceBuildingSize, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::FurnitureReview,
        playable_spacing_metres: 20.0,
        ..super::fixture(
            "furniture-review",
            "city",
            fabelgeist_determinism::Seed::from_u64(47_122),
            flat,
            open_yard,
            clear(),
        )
    }
}

pub(super) fn buildings() -> Result<Vec<TacticalBuildingPlacement>, Box<dyn std::error::Error>> {
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
    .map(|(index, (usage, centre_metres, yaw))| {
        Ok(TacticalBuildingPlacement {
            base_elevation_metres:
                adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
            id: adventuresim_tactical_core::scene_input::SceneBuildingId(index as u64 + 1),
            program: BuildingProgram::validated_settlement(
                settlement_archetype(usage),
                usage,
                fabelgeist_determinism::Seed::from_u64(42),
                Some(ServiceBuildingSize::Medium),
            )?,
            centre_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                centre_metres,
            )?,
            orientation: BuildingOrientation::from_radians(yaw).ok_or(
                adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection,
            )?,
        })
    })
    .collect()
}

pub(super) fn streets()
-> adventuresim_building_generator::spatial_geometry::GeometryResult<Vec<CityStreetPatch>> {
    let mut streets = vec![CityStreetPatch::Market {
        corners_metres: scene_corners(corners(Vec2::splat(-25.0), Vec2::splat(25.0)))?,
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
            start_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(start_metres)?,
            end_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(end_metres)?,
            half_width_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(3.5)
                    ?,
            surface,
        });
    }
    Ok(streets)
}

pub(super) fn yards()
-> adventuresim_building_generator::spatial_geometry::GeometryResult<Vec<CityYardPatch>> {
    [
        (Vec2::new(34.0, -20.0), Vec2::new(72.0, 20.0)),
        (Vec2::new(-73.0, -46.0), Vec2::new(-36.0, -5.0)),
        (Vec2::new(-20.0, 39.0), Vec2::new(20.0, 80.0)),
    ]
    .into_iter()
    .map(|(min, max)| {
        Ok(CityYardPatch {
            corners_metres: scene_corners(corners(min, max))?,
            surface: CityYardSurface::PackedEarth,
        })
    })
    .collect()
}

fn open_yard(_: f32, _: f32) -> EnvironmentalSample {
    sample(TacticalSurface::Open, 0, 0, 0, 0)
}

fn corners(min: Vec2, max: Vec2) -> [Vec2; 4] {
    [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)]
}
