//! Keep the architectural processional route open through furnished naves.
use super::geometry::{PERSON_RADIUS, Rect, room_bounds};
use crate::interior::InteriorResult as Result;
use crate::{BuildingPlan, ChurchRouteKind, RoomKind};
use bevy::math::Vec2;

pub(super) fn nave_routes(
    plan: &BuildingPlan,
    height: crate::spatial_geometry::Elevation<crate::Architectural>,
) -> Result<Vec<Rect>> {
    let height = height.metres();
    let mut routes = Vec::new();
    if let Some(church) = &plan.small_church {
        let route = church.public_route;
        if (route.min().metres().y - height).abs() <= PERSON_RADIUS {
            routes.push(Rect::from_metres(
                Vec2::new(
                    route.min().metres().x + route.max().metres().x,
                    route.min().metres().z + route.max().metres().z,
                ) * 0.5,
                Vec2::new(
                    route.max().metres().x - route.min().metres().x,
                    route.max().metres().z - route.min().metres().z,
                ) * 0.5,
            )?);
        }
    }
    if let Some(church) = &plan.church {
        for route in &church.circulation {
            if route.kind != ChurchRouteKind::PublicProcessional {
                continue;
            }
            for pair in route.waypoints.windows(2) {
                if (pair[0].y - height).abs() <= PERSON_RADIUS
                    && (pair[1].y - height).abs() <= PERSON_RADIUS
                {
                    let a = Vec2::new(pair[0].x, pair[0].z);
                    let b = Vec2::new(pair[1].x, pair[1].z);
                    routes.push(Rect::from_metres(
                        (a + b) * 0.5,
                        (b - a).abs() * 0.5 + Vec2::splat(route.width_metres * 0.5),
                    )?);
                }
            }
        }
    }
    // Chancel fittings own their own approach. Do not extend a nave furniture
    // exclusion through the altar at the far end of the architectural route.
    let mut intersections = Vec::new();
    for storey in &plan.storeys {
        for room in storey
            .rooms
            .iter()
            .filter(|room| room.kind == RoomKind::Nave)
        {
            let bounds = room_bounds(room, crate::StoreyIndex::from_serialized(storey.level))?;
            let (min, max) = (bounds.min.metres(), bounds.max.metres());
            for route in &routes {
                let a = min.max(route.centre.metres() - route.half.metres());
                let b = max.min(route.centre.metres() + route.half.metres());
                if a.cmplt(b).all() {
                    intersections.push(Rect::from_metres((a + b) * 0.5, (b - a) * 0.5)?);
                }
            }
        }
    }
    Ok(intersections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interior::{furnish, validate_layout};
    use crate::{BuildingArchetype, BuildingProgram, generate};

    #[test]
    fn parish_furnishings_preserve_the_full_architectural_nave_aisle() {
        for seed in [42, 47, 101] {
            let program = BuildingProgram::fixture(BuildingArchetype::ParishChurch, seed);
            let plan = generate(&program).unwrap();
            let layout = furnish(&plan, &program).unwrap();
            let routes = nave_routes(
                &plan,
                crate::spatial_geometry::Elevation::from_metres(0.16).unwrap(),
            )
            .unwrap();
            assert!(!routes.is_empty());
            assert!(layout.placements.iter().all(|placement| {
                !routes
                    .iter()
                    .any(|route| route.overlaps(placement.footprint().unwrap()))
            }));
            validate_layout(&plan, &layout).unwrap();
            let pulpit = layout
                .placements
                .iter()
                .find(|p| p.key.kind() == crate::furniture::FurnitureKind::Pulpit)
                .expect("parish nave retains a preaching position");
            let nave = plan.storeys[0]
                .rooms
                .iter()
                .find(|r| r.kind == RoomKind::Nave)
                .unwrap();
            let bounds = room_bounds(nave, crate::StoreyIndex::GROUND).unwrap();
            let (min, max) = (bounds.min.metres(), bounds.max.metres());
            assert!(pulpit.centre_metres.metres().y >= (min.y + max.y) * 0.5);
            assert_eq!(pulpit.facing, crate::Direction::South);
        }
    }
}
