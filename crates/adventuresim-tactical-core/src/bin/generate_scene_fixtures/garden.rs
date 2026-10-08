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
            fabelgeist_determinism::Seed::from_u64(47_127),
            flat,
            |_, _| sample(TacticalSurface::Open, 0, 0, 0, 0),
            clear(),
        )
    }
}

pub(super) fn layout() -> Result<CitySceneLayout, Box<dyn std::error::Error>> {
    let mut economy = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
    economy.services = vec![
        adventuresim_world_schema::SettlementService::Inn,
        adventuresim_world_schema::SettlementService::Temple,
    ];
    let city = CitySite::central_german_market_town()?
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(900),
            &economy,
        )?
        .compile((42).into())?;
    let mut garden = city
        .gardens
        .into_iter()
        .next()
        .ok_or_else(|| std::io::Error::other("review city has no accepted garden"))?;
    let mut front = city
        .buildings
        .into_iter()
        .find(|b| b.id == garden.front_building_id)
        .ok_or_else(|| std::io::Error::other("review garden lacks its front member"))?;
    normalize_garden(&mut garden, &mut front)?;
    let mut yards = vec![CityYardPatch::from_bounds(
        garden.plot,
        CityYardSurface::PackedEarth,
    )?];
    for bed in &garden.beds {
        yards.push(CityYardPatch::from_bounds(
            *bed,
            CityYardSurface::KitchenGarden,
        )?);
    }
    // The isolated catalogue has a straight frontage street. Seat its near
    // edge at the retained property edge rather than using an access endpoint
    // as a street centre, which left an unowned gap before the threshold.
    let street_half_width = 3.5;
    let street_depth =
        garden.plot.centre_metres().y - garden.plot.dimensions_metres().y * 0.5 - street_half_width;
    let offset = Vec2::new(0.0, 70.0);
    let mut distant_garden = garden.clone();
    distant_garden.owner = adventuresim_tactical_core::city_layout::CityPropertyId(10_032);
    distant_garden.front_building_id =
        adventuresim_tactical_core::scene_input::SceneBuildingId(distant_garden.owner.0);
    translate_catalogue_garden(&mut distant_garden, offset)?;
    let mut distant_yards = yards.clone();
    for yard in &mut distant_yards {
        for point in &mut yard.corners_metres {
            *point = point.translated(
                adventuresim_tactical_core::scene_coordinates::PlanDisplacement::try_from(offset)?,
            )?;
        }
    }
    yards.extend(distant_yards);
    let distant = DistantBuildingPlacement {
        prosperity: adventuresim_world_schema::ProsperityTier::Comfortable,
        id: distant_garden.front_building_id,
        archetype: front.program.archetype,
        usage: front.program.usage,
        service_size: front.program.service_size,
        seed: front.program.seed,
        centre_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            offset,
        )?,
        orientation: front.orientation,
        base_elevation_metres:
            adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
    };
    Ok(CitySceneLayout {
        playable: vec![front],
        distant: vec![distant],
        gardens: vec![garden, distant_garden],
        yards,
        streets: catalogue_streets(street_depth, street_half_width, offset)?,
        ..Default::default()
    })
}

/// Rigid normalization preserves the accepted garden's complete member poses.
fn normalize_garden(
    garden: &mut adventuresim_tactical_core::city_layout::CityGarden,
    front: &mut TacticalBuildingPlacement,
) -> adventuresim_building_generator::spatial_geometry::GeometryResult<()> {
    let origin = front.centre_metres.metres();
    let orientation = front.orientation;
    let local = |p| orientation.world_to_local(p - origin);
    front.centre_metres = adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::ORIGIN;
    front.orientation = BuildingOrientation::IDENTITY;
    for bounds in std::iter::once(&mut garden.plot)
        .chain(std::iter::once(&mut garden.cultivated_bounds))
        .chain(garden.beds.iter_mut())
    {
        bounds.relocate(
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                bounds.centre_metres(),
            ))?,
        )?;
        bounds.rotate(BuildingOrientation::IDENTITY)?;
    }
    for access in &mut garden.access {
        access.update_endpoints(
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                access.start_metres(),
            ))?,
            access.end(),
        )?;
        access.update_endpoints(
            access.start(),
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                access.end_metres(),
            ))?,
        )?;
    }
    for plant in &mut garden.plants {
        plant.centre_metres =
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                plant.centre_metres.metres(),
            ))?;
        plant.orientation = BuildingOrientation::IDENTITY;
    }
    Ok(())
}

fn translate_catalogue_garden(
    garden: &mut adventuresim_tactical_core::city_layout::CityGarden,
    offset: Vec2,
) -> Result<(), adventuresim_building_generator::spatial_geometry::GeometryError> {
    for bounds in std::iter::once(&mut garden.plot)
        .chain(std::iter::once(&mut garden.cultivated_bounds))
        .chain(garden.beds.iter_mut())
    {
        bounds.relocate(
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                bounds.centre_metres() + (offset),
            )?,
        )?;
    }
    for route in &mut garden.access {
        route.update_endpoints(
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                route.start_metres() + (offset),
            )?,
            route.end(),
        )?;
        route.update_endpoints(
            route.start(),
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                route.end_metres() + (offset),
            )?,
        )?;
    }
    for plant in &mut garden.plants {
        plant.id = adventuresim_tactical_core::city_layout::GardenPlantId(garden.owner.0);
        plant.centre_metres = plant.centre_metres.translated(
            adventuresim_tactical_core::scene_coordinates::PlanDisplacement::try_from(offset)?,
        )?;
    }
    Ok(())
}

fn catalogue_streets(
    street_depth: f32,
    street_half_width: f32,
    offset: Vec2,
) -> adventuresim_building_generator::spatial_geometry::GeometryResult<Vec<CityStreetPatch>> {
    [0.0, offset.y]
        .into_iter()
        .map(|depth| {
            Ok(CityStreetPatch::Corridor {
                start_metres:
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                        Vec2::new(-45.0, street_depth + depth),
                    )?,
                end_metres:
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                        Vec2::new(45.0, street_depth + depth),
                    )?,
                half_width_metres:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        street_half_width,
                    )?,
                surface: CityStreetSurface::CompactedEarth,
            })
        })
        .collect()
}
