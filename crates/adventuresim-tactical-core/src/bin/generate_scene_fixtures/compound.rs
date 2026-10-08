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
            fabelgeist_determinism::Seed::from_u64(47_124),
            flat,
            |_, _| sample(TacticalSurface::Open, 0, 0, 0, 0),
            clear(),
        )
    }
}

pub(super) fn layout() -> Result<CitySceneLayout, Box<dyn std::error::Error>> {
    let city = CitySite::central_german_market_town()?
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(900),
            &adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder(),
        )?
        .compile((42).into())?;
    let mut layout = CitySceneLayout::default();
    for (id, side, offset) in [
        (1, PropertySide::Left, Vec2::new(-18.0, 0.0)),
        (2, PropertySide::Right, Vec2::new(18.0, 0.0)),
    ] {
        let property = isolate(&city, id, side, offset)?;
        layout.playable.extend(property.playable);
        layout.compounds.extend(property.compounds);
        layout.yards.extend(property.yards);
    }
    layout.streets.push(CityStreetPatch::Corridor {
        start_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            Vec2::new(-60.0, -11.0),
        )?,
        end_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            Vec2::new(60.0, -11.0),
        )?,
        half_width_metres:
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(3.5)?,
        surface: CityStreetSurface::CompactedEarth,
    });
    Ok(layout)
}

fn isolate(
    city: &CompiledCityLayout,
    id: u64,
    passage: PropertySide,
    offset: Vec2,
) -> Result<CitySceneLayout, Box<dyn std::error::Error>> {
    let mut compound = city
        .compounds
        .iter()
        .find(|property| property.boundary.gate.hinge == passage.opposite())
        .ok_or_else(|| {
            std::io::Error::other("review city lacks the requested passage orientation")
        })?
        .clone();
    let front = city
        .buildings
        .iter()
        .find(|b| b.id == compound.front_building_id)
        .ok_or_else(|| std::io::Error::other("review property lacks its front member"))?;
    let origin = front.centre_metres.metres();
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
            adventuresim_tactical_core::scene_input::SceneBuildingId(id)
        } else {
            adventuresim_tactical_core::scene_input::SceneBuildingId(MAX_CITY_LOTS as u64 + id)
        };
        building.centre_metres =
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                building.centre_metres.metres(),
            ))?;
        building.orientation = BuildingOrientation::IDENTITY;
    }
    for bounds in [&mut compound.plot, &mut compound.court] {
        bounds.relocate(
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                bounds.centre_metres(),
            ))?,
        )?;
        bounds.rotate(BuildingOrientation::IDENTITY)?;
    }
    for access in &mut compound.access {
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
    for wall in &mut compound.boundary.walls {
        wall.start_metres =
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
                wall.start_metres.metres(),
            ))?;
        wall.end_metres = adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            local(wall.end_metres.metres()),
        )?;
    }
    compound.boundary.gate.centre_metres =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(local(
            compound.boundary.gate.centre_metres.metres(),
        ))?;
    compound.boundary.gate.orientation = BuildingOrientation::IDENTITY;
    compound.id = CityPropertyId(id);
    compound.front_building_id = adventuresim_tactical_core::scene_input::SceneBuildingId(id);
    compound.rear_building_id =
        adventuresim_tactical_core::scene_input::SceneBuildingId(MAX_CITY_LOTS as u64 + id);
    let yards = vec![CityYardPatch::from_bounds(
        compound.plot,
        CityYardSurface::PackedEarth,
    )?];
    Ok(CitySceneLayout {
        playable,
        yards,
        parishes: Vec::new(),
        compounds: vec![compound],
        ..Default::default()
    })
}
