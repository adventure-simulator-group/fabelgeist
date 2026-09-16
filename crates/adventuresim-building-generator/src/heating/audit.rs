//! Independent checks of ownership, smoke continuity, masonry and weather cuts.
use crate::*;
use bevy::math::{Vec2, Vec3};
use geo::{Area, BooleanOps};
use std::collections::BTreeSet;
mod enclosures;
mod roof;
mod weathering;
const GEOMETRY_TOLERANCE_METRES: f32 = 0.002;

pub(crate) fn audit(plan: &BuildingPlan) -> Vec<AuditIssue> {
    let mut issues = Vec::new();
    let Some(h) = &plan.domestic_heating else {
        if plan
            .resolved_geometry
            .solids
            .iter()
            .any(|s| s.role == SolidRole::DomesticHeating)
        {
            fail(
                &mut issues,
                "unowned_domestic_heating",
                "heating geometry has no programme",
            );
        }
        return issues;
    };
    ownership(plan, h, &mut issues);
    clearance(plan, h, &mut issues);
    bearings(plan, h, &mut issues);
    passages(plan, h, &mut issues);
    enclosures::audit(plan, h, &mut issues);
    roof::audit(plan, h, &mut issues);
    issues
}
fn fail(issues: &mut Vec<AuditIssue>, code: &'static str, message: &str) {
    issues.push(AuditIssue {
        code,
        message: message.to_owned(),
    });
}
fn ownership(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    let room = |r: HeatingRoom| {
        plan.storeys
            .iter()
            .find(|s| s.level == r.storey_level)
            .and_then(|s| s.rooms.iter().find(|room| room.id == r.room_id))
    };
    let wall = plan.wall_assemblies.iter().find(|w| w.id == h.fire_wall);
    let valid = plan.storeys.len() == 1
        && h.kitchen.storey_level == 0
        && h.heated_room.storey_level == 0
        && room(h.kitchen).is_some_and(|r| r.kind == RoomKind::Kitchen)
        && room(h.heated_room)
            .is_some_and(|r| matches!(r.kind, RoomKind::CommonRoom | RoomKind::GreatHall))
        && wall.is_some_and(|w| {
            w.owner == h.owner
                && w.opening_ids.is_empty()
                && [w.frame.inside_room, w.frame.outside_room].contains(&Some(h.kitchen.room_id))
                && [w.frame.inside_room, w.frame.outside_room]
                    .contains(&Some(h.heated_room.room_id))
        })
        && plan.resolved_geometry.structural_nodes.iter().any(|n| {
            n.id == h.ground_support && n.grounded && n.position.y.abs() < GEOMETRY_TOLERANCE_METRES
        });
    let ids = h.parts.iter().map(|p| p.solid).collect::<BTreeSet<_>>();
    let actual = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|s| s.role == SolidRole::DomesticHeating)
        .map(|s| s.id)
        .collect::<BTreeSet<_>>();
    if !valid
        || ids.len() != h.parts.len()
        || ids != actual
        || h.parts.iter().any(|p| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == p.solid)
                .is_none_or(|s| s.owner != h.owner)
        })
    {
        fail(
            issues,
            "invalid_domestic_heating_ownership",
            "heating parts, ground bearing or kitchen/Stube references disagree",
        );
    }
}
fn clearance(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    let ids = h.parts.iter().map(|p| p.solid).collect::<BTreeSet<_>>();
    if plan.resolved_geometry.solids.iter().any(|s| {
        crate::solid_overlap::overlaps_bounds(
            s,
            (h.operating_space.min, h.operating_space.max),
            GEOMETRY_TOLERANCE_METRES,
        )
    }) {
        fail(
            issues,
            "blocked_hearth_operation",
            "the kitchen-side operating space is obstructed",
        );
    }
    for part in &h.parts {
        if let Some(solid) = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == part.solid)
            && (!matches!(solid.shape, ResolvedSolidShape::Cuboid)
                || (part.kind != HeatingPartKind::RoofFlashing
                    && [
                        solid.yaw_radians,
                        solid.crossfall_radians,
                        solid.longfall_radians,
                    ]
                    .into_iter()
                    .any(|angle| angle.abs() > 0.0001)))
        {
            fail(
                issues,
                "invalid_domestic_appliance_shape",
                "appliance masonry must retain its axis-aligned cuboid section; only roof sheets are inclined",
            );
        }
    }
    for part in h.parts.iter().filter(|p| p.kind == HeatingPartKind::Flue) {
        let Some(flue) = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == part.solid)
        else {
            continue;
        };
        let b = flue.cuboid_bounds();
        let margin = Vec3::new(
            super::placement::TIMBER_CLEARANCE_METRES,
            0.0,
            super::placement::TIMBER_CLEARANCE_METRES,
        );
        if plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|s| !ids.contains(&s.id))
            .any(|s| {
                crate::solid_overlap::overlaps_bounds(
                    s,
                    (b.min - margin, b.max + margin),
                    GEOMETRY_TOLERANCE_METRES,
                )
            })
        {
            fail(
                issues,
                "domestic_flue_timber_clearance",
                "the completed flue intersects retained construction or its clearance band",
            );
        }
    }
}
fn passages(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    use HeatingPassageKind::*;
    let expected = [
        HearthMouth,
        StoveFirebox,
        StoveSmokeReturn,
        HoodThroat,
        FlueBore,
        StoveChamber,
    ];
    let routes = expected.map(|kind| {
        let records = h
            .passages
            .iter()
            .filter(|p| p.kind == kind)
            .collect::<Vec<_>>();
        if records.len() != 1 {
            return None;
        }
        plan.resolved_geometry
            .voids
            .iter()
            .find(|v| v.id == records[0].void)
    });
    if h.passages.len() != expected.len() || routes.iter().any(Option::is_none) {
        fail(
            issues,
            "incomplete_domestic_smoke_route",
            "the hearth, stove return, hood and outdoor flue need unique passages",
        );
        return;
    }
    let routes = routes.map(Option::unwrap);
    for route in routes {
        if route.owner != h.owner
            || route.subtracts_from != h.owner
            || (route.bounds.max - route.bounds.min).min_element() < 0.1
            || plan.resolved_geometry.solids.iter().any(|s| {
                crate::solid_overlap::overlaps_bounds(
                    s,
                    (route.bounds.min, route.bounds.max),
                    GEOMETRY_TOLERANCE_METRES,
                )
            })
        {
            fail(
                issues,
                "blocked_domestic_smoke_route",
                "a smoke passage is invalid or contains actual solid material",
            );
        }
    }
    let mut reached = BTreeSet::from([4]);
    loop {
        let before = reached.len();
        for index in 0..routes.len() {
            if reached.iter().any(|other| {
                let a = routes[index].bounds;
                let b = routes[*other].bounds;
                (a.max.min(b.max) - a.min.max(b.min)).min_element() > GEOMETRY_TOLERANCE_METRES
            }) {
                reached.insert(index);
            }
        }
        if reached.len() == before {
            break;
        }
    }
    if reached.len() != routes.len() {
        fail(
            issues,
            "disconnected_domestic_smoke_route",
            "the stove and hearth do not connect to the outdoor bore",
        );
    }
    flue_shell(plan, h, routes[4].bounds, issues);
}
fn flue_shell(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    bore: ResolvedBounds,
    issues: &mut Vec<AuditIssue>,
) {
    let flues = h
        .parts
        .iter()
        .filter(|p| p.kind == HeatingPartKind::Flue)
        .filter_map(|p| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == p.solid)
        })
        .collect::<Vec<_>>();
    if flues.len() != 4 {
        fail(
            issues,
            "open_domestic_flue_shell",
            "the flue lacks a complete masonry shell",
        );
        return;
    }
    let outer = flues
        .iter()
        .map(|s| s.cuboid_bounds())
        .fold(flues[0].cuboid_bounds(), |a, b| ResolvedBounds {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        });
    let expected = rect(outer).difference(&rect(bore));
    for fraction in [0.01, 0.5, 0.99] {
        let y = outer.min.y + (outer.max.y - outer.min.y) * fraction;
        let mut actual = geo::MultiPolygon::new(vec![]);
        for solid in &flues {
            let b = solid.cuboid_bounds();
            if b.min.y <= y && b.max.y >= y {
                actual = actual.union(&rect(b));
            }
        }
        if expected.difference(&actual).unsigned_area() > 0.00001
            || actual.intersection(&rect(bore)).unsigned_area() > 0.00001
        {
            fail(
                issues,
                "open_domestic_flue_shell",
                "a section of the masonry flue is open or blocks its bore",
            );
            break;
        }
    }
}
fn rect(bounds: ResolvedBounds) -> geo::Polygon<f32> {
    geo::Rect::new(
        geo::coord! {x:bounds.min.x,y:bounds.min.z},
        geo::coord! {x:bounds.max.x,y:bounds.max.z},
    )
    .to_polygon()
}
fn polygon(points: &[Vec3]) -> geo::Polygon<f32> {
    geo::Polygon::new(
        geo::LineString::new(points.iter().map(|v| geo::coord! {x:v.x,y:v.z}).collect()),
        vec![],
    )
}

fn bearings(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    for part in &h.parts {
        let Some(solid) = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == part.solid)
        else {
            continue;
        };
        for node_id in &solid.supported_by {
            let Some(node) = plan
                .resolved_geometry
                .structural_nodes
                .iter()
                .find(|n| n.id == *node_id)
            else {
                continue;
            };
            if node.grounded {
                if solid.cuboid_bounds().min.y > GEOMETRY_TOLERANCE_METRES {
                    fail(
                        issues,
                        "detached_heating_bearing",
                        "a heating part above ground cannot declare a foundation",
                    );
                }
                continue;
            }
            if node.supported_by.is_empty()
                || node.supported_by.iter().any(|parent| {
                    !plan
                        .resolved_geometry
                        .solids
                        .iter()
                        .filter(|s| s.id != solid.id && s.supported_by.contains(parent))
                        .any(|support| {
                            super::contact::measured(solid, support).is_some_and(|contact| {
                                plan.resolved_geometry.support_interfaces.iter().any(|i| {
                                    i.node == *node_id
                                        && i.owner == h.owner
                                        && i.bounds.min.distance(contact.min)
                                            < GEOMETRY_TOLERANCE_METRES
                                        && i.bounds.max.distance(contact.max)
                                            < GEOMETRY_TOLERANCE_METRES
                                })
                            })
                        })
                })
            {
                fail(
                    issues,
                    "detached_heating_bearing",
                    "heating support must contact actual finished masonry or sheet geometry",
                );
            }
        }
    }
}
