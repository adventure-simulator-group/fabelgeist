//! Keep the architectural processional route open through furnished naves.
use super::geometry::{PERSON_RADIUS, Rect, room_bounds};
use crate::{BuildingPlan, ChurchRouteKind, RoomKind};
use bevy::math::Vec2;

pub(super) fn nave_routes(plan: &BuildingPlan, height: f32) -> Vec<Rect> {
    let mut routes = Vec::new();
    if let Some(church) = &plan.small_church {
        let route = church.public_route;
        if (route.min.y - height).abs() <= PERSON_RADIUS {
            routes.push(Rect::new(
                Vec2::new(route.min.x + route.max.x, route.min.z + route.max.z) * 0.5,
                Vec2::new(route.max.x - route.min.x, route.max.z - route.min.z) * 0.5,
            ));
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
                    routes.push(Rect::new(
                        (a + b) * 0.5,
                        (b - a).abs() * 0.5 + Vec2::splat(route.width_metres * 0.5),
                    ));
                }
            }
        }
    }
    // Chancel fittings own their own approach. Do not extend a nave furniture
    // exclusion through the altar at the far end of the architectural route.
    plan.storeys
        .iter()
        .flat_map(|storey| &storey.rooms)
        .filter(|room| room.kind == RoomKind::Nave)
        .flat_map(|room| {
            let (min, max) = room_bounds(room);
            routes.iter().filter_map(move |route| {
                let a = min.max(route.centre - route.half);
                let b = max.min(route.centre + route.half);
                a.cmplt(b)
                    .all()
                    .then(|| Rect::new((a + b) * 0.5, (b - a) * 0.5))
            })
        })
        .collect()
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
            let routes = nave_routes(&plan, 0.16);
            assert!(!routes.is_empty());
            assert!(layout.placements.iter().all(|placement| {
                !routes
                    .iter()
                    .any(|route| route.overlaps(placement.footprint()))
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
            let (min, max) = room_bounds(nave);
            assert!(pulpit.centre_metres.y >= (min.y + max.y) * 0.5);
            assert_eq!(pulpit.facing, crate::Direction::South);
        }
    }
}
