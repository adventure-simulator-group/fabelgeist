//! Validate the principal church's complete envelope and street approach.
use super::*;
use adventuresim_building_generator::interior::StandingClearance;
use recipes::Recipe;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChurchSitingIssue {
    GeometryOutsidePlot,
    MissingWestPortal,
    StreetDisconnected,
    ApproachBlocked,
}

pub(super) fn validate(
    lot: CityBuildingLot,
    placement: &TacticalBuildingPlacement,
    recipe: &Recipe,
    streets: &[CityStreetPatch],
) -> Result<(), CityCompileError> {
    let error = |issue| CityCompileError::Church {
        building: lot.id,
        issue,
    };
    let plot = CityPlotBounds::from(lot);
    if !recipe.fits(placement, plot) {
        return Err(error(ChurchSitingIssue::GeometryOutsidePlot));
    }
    let door = recipe
        .door_point(placement, adventuresim_building_generator::Direction::West)?
        .ok_or(error(ChurchSitingIssue::MissingWestPortal))?
        .metres();
    let local_door = lot.orientation.world_to_local(door - lot.centre_metres);
    let apron = services::SERVICE_EDGE_CLEARANCE_METRES - ORDINARY_STREET_HALF_WIDTH_METRES;
    let street = lot.centre_metres
        + lot.orientation.local_to_world(Vec2::new(
            local_door.x,
            -lot.footprint_metres.y * 0.5 - apron,
        ));
    if !plot.contains(door) || !streets.iter().any(|patch| patch.contains(street)) {
        return Err(error(ChurchSitingIssue::StreetDisconnected));
    }
    let origin = recipe.collision.bounds.centre()?.metres();
    let physical = |p| {
        let point = placement
            .orientation
            .world_to_local(p - placement.centre_metres)
            + Vec2::new(origin.x, origin.z);
        adventuresim_building_generator::spatial_geometry::Position::<
            adventuresim_building_generator::Architectural,
        >::from_metres(bevy::math::Vec3::new(point.x, 0.0, point.y))
    };
    // Door leaves are dynamic and excluded from static building collision.
    // This continuous body sweep includes the portal throat and its outer path.
    if !StandingClearance::new(
        &recipe.collision.cuboids,
        adventuresim_building_generator::spatial_geometry::Elevation::from_metres(0.0)?,
    )?
    .is_clear(physical(door)?, physical(street)?)
    {
        return Err(error(ChurchSitingIssue::ApproachBlocked));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::{CollisionCuboid, ResolvedItemId, ServiceBuildingSize};
    use bevy::math::Vec3;

    #[test]
    fn principal_frontage_rotates_with_its_plot_and_rejects_blocked_access() {
        let city = CitySite::central_german_market_town().generate(
            42,
            6500,
            &super::super::super::tests::economy(),
        );
        let mut lot = *city
            .lots
            .iter()
            .find(|lot| {
                lot.building_use() == Some(BuildingUse::ParishChurch)
                    && lot.service_size() == Some(ServiceBuildingSize::Large)
            })
            .unwrap();
        let recipe = CityRecipePalette::default().front(42, lot).unwrap();
        lot.centre_metres = Vec2::ZERO;
        for yaw in [0.0, core::f32::consts::FRAC_PI_2, 0.37] {
            lot.orientation = BuildingOrientation::from_radians(yaw).unwrap();
            let street_y = -lot.footprint_metres.y * 0.5 - services::SERVICE_EDGE_CLEARANCE_METRES;
            let street = CityStreetPatch::Corridor {
                start_metres: lot.orientation.local_to_world(Vec2::new(-20.0, street_y)),
                end_metres: lot.orientation.local_to_world(Vec2::new(20.0, street_y)),
                half_width_metres: ORDINARY_STREET_HALF_WIDTH_METRES,
                surface: CityStreetSurface::Fieldstone,
            };
            let placement = recipe
                .place(lot.id, lot.centre_metres, lot.orientation)
                .unwrap();
            validate(lot, &placement, &recipe, &[street]).unwrap();
            assert_eq!(
                validate(lot, &placement, &recipe, &[]),
                Err(CityCompileError::Church {
                    building: lot.id,
                    issue: ChurchSitingIssue::StreetDisconnected,
                })
            );
            let smaller = CityBuildingLot {
                footprint_metres: Vec2::splat(10.0),
                ..lot
            };
            assert_eq!(
                validate(smaller, &placement, &recipe, &[street]),
                Err(CityCompileError::Church {
                    building: lot.id,
                    issue: ChurchSitingIssue::GeometryOutsidePlot,
                })
            );
            let mut blocked = Recipe {
                program: recipe.program.clone(),
                collision: recipe.collision.clone(),
                render_min: recipe.render_min,
                render_max: recipe.render_max,
                doors: recipe.doors.clone(),
                ground_entrances: recipe.ground_entrances.clone(),
            };
            let door = blocked
                .doors
                .iter()
                .find(|d| d.outward.vector() == -Vec2::X)
                .unwrap();
            blocked.collision.cuboids.push(CollisionCuboid::<adventuresim_building_generator::spatial_geometry::Architectural>::from_metres(ResolvedItemId(999_000), Vec3::new(door.closed_centre.metres().x - 1.0, 1.0, door.closed_centre.metres().z), Vec3::new(0.2, 2.0, 3.0), 0.0, 0.0, 0.0).unwrap());
            assert_eq!(
                validate(lot, &placement, &blocked, &[street]),
                Err(CityCompileError::Church {
                    building: lot.id,
                    issue: ChurchSitingIssue::ApproachBlocked,
                })
            );
        }
    }
}
