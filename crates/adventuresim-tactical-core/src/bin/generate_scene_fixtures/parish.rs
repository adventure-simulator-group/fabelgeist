//! Review recipes are generated through the same capacity-selected settlement API as cities.
use super::*;
use adventuresim_building_generator::ServiceBuildingSize;
use adventuresim_world_schema::settlement_buildings::BuildingUse;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::ParishReview,
        playable_spacing_metres: 30.0,
        ..super::fixture(
            "parish-review",
            "grassland",
            fabelgeist_determinism::Seed::from_u64(47_121),
            flat,
            open_yard,
            clear(),
        )
    }
}

pub(super) fn buildings() -> Result<
    Vec<TacticalBuildingPlacement>,
    adventuresim_building_generator::spatial_geometry::GeometryError,
> {
    [BuildingUse::Chapel, BuildingUse::ParishChurch]
        .into_iter()
        .enumerate()
        .flat_map(|(row, usage)| {
            [
                ServiceBuildingSize::Small,
                ServiceBuildingSize::Medium,
                ServiceBuildingSize::Large,
            ]
            .into_iter()
            .enumerate()
            .map(move |(column, size)| {
                Ok(TacticalBuildingPlacement {
                    base_elevation_metres:
                        adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
                    id: adventuresim_tactical_core::scene_input::SceneBuildingId(
                        (row * 3 + column + 1) as u64,
                    ),
                    program: BuildingProgram::settlement(
                        BuildingArchetype::ParishChurch,
                        Some(usage),
                        fabelgeist_determinism::Seed::from_u64(42),
                    )
                    .with_service_size(size),
                    centre_metres:
                        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                            Vec2::new((column as f32 - 1.0) * 60.0, 45.0 - row as f32 * 90.0),
                        )?,
                    orientation: BuildingOrientation::from_radians(-std::f32::consts::PI).ok_or(adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection)?,
                })
            })
        })
        .collect()
}

pub(super) fn yards()
-> adventuresim_building_generator::spatial_geometry::GeometryResult<Vec<CityYardPatch>> {
    Ok(vec![CityYardPatch {
        corners_metres: scene_corners([
            Vec2::new(-120.0, -120.0),
            Vec2::new(120.0, -120.0),
            Vec2::new(120.0, 120.0),
            Vec2::new(-120.0, 120.0),
        ])?,
        surface: CityYardSurface::PackedEarth,
    }])
}

fn open_yard(_: f32, _: f32) -> EnvironmentalSample {
    sample(TacticalSurface::Open, 0, 0, 0, 0)
}
