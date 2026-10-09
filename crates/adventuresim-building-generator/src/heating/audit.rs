//! Independent checks of ownership, smoke continuity, masonry and weather cuts.
use crate::GenerationResult as Result;
use crate::*;
use bevy::math::{Vec2, Vec3};
use geo::{Area, BooleanOps};
use std::collections::BTreeSet;
mod enclosures;
mod floors;
mod roof;
pub(super) mod roof_route;
mod weathering;
const GEOMETRY_TOLERANCE_METRES: f32 = 0.002;
const AREA_TOLERANCE_SQUARE_METRES: f32 = 0.00001;

pub(crate) fn audit(plan: &BuildingPlan) -> Result<Vec<AuditIssue>> {
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
        return Ok(issues);
    };
    ownership(plan, h, &mut issues)?;
    clearance(plan, h, &mut issues)?;
    bearings(plan, h, &mut issues)?;
    floors::audit(plan, h, &mut issues)?;
    passages(plan, h, &mut issues)?;
    enclosures::audit(plan, h, &mut issues)?;
    roof::audit(plan, h, &mut issues)?;
    Ok(issues)
}
fn fail(issues: &mut Vec<AuditIssue>, code: &'static str, message: &str) {
    issues.push(AuditIssue {
        code,
        message: message.to_owned(),
    });
}
fn ownership(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let room = |r: HeatingRoom| {
        plan.storeys
            .iter()
            .find(|s| StoreyIndex::from_serialized(s.level) == r.storey_level)
            .and_then(|s| {
                s.rooms
                    .iter()
                    .find(|room| RoomIndex::from_serialized(room.id) == r.room_id)
            })
    };
    let wall = plan.wall_assemblies.iter().find(|w| w.id == h.fire_wall);
    let valid = h.kitchen.storey_level == h.heated_room.storey_level
        && (h.floor_height_metres.metres()
            - f32::from(h.kitchen.storey_level.serialized_ordinal()?) * plan.storey_height_metres)
            .abs()
            < GEOMETRY_TOLERANCE_METRES
        && room(h.kitchen).is_some_and(|r| r.kind == RoomKind::Kitchen)
        && room(h.heated_room)
            .is_some_and(|r| matches!(r.kind, RoomKind::CommonRoom | RoomKind::GreatHall))
        && wall.is_some_and(|w| {
            StoreyIndex::from_serialized(w.storey_level) == h.kitchen.storey_level
                && w.owner == h.owner
                && w.opening_ids.is_empty()
                && [w.frame.inside_room, w.frame.outside_room]
                    .contains(&Some(h.kitchen.room_id.serialized_ordinal()))
                && [w.frame.inside_room, w.frame.outside_room]
                    .contains(&Some(h.heated_room.room_id.serialized_ordinal()))
        })
        && plan.resolved_geometry.structural_nodes.iter().any(|n| {
            n.id == h.ground_support
                && n.grounded
                && n.position.metres().y.abs() < GEOMETRY_TOLERANCE_METRES
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

    Ok(())
}
fn clearance(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let ids = h.parts.iter().map(|p| p.solid).collect::<BTreeSet<_>>();
    if plan.resolved_geometry.solids.iter().any(|s| {
        crate::solid_overlap::overlaps_bounds(
            s,
            (
                h.operating_space.min().metres(),
                h.operating_space.max().metres(),
            ),
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
                    .any(|angle| angle.radians().abs() > 0.0001)))
        {
            fail(
                issues,
                "invalid_domestic_appliance_shape",
                "appliance masonry must retain its axis-aligned cuboid section; only roof sheets are inclined",
            );
        }
    }
    let _: () = for part in h.parts.iter().filter(|p| {
        matches!(
            p.kind,
            HeatingPartKind::Flue | HeatingPartKind::FlueShoulder
        )
    }) {
        let Some(flue) = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == part.solid)
        else {
            continue;
        };
        let b = flue.cuboid_bounds()?;
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
                    (b.min().metres() - margin, b.max().metres() + margin),
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
    };
    Ok(())
}
fn passages(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
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
        return Ok(());
    }
    let routes = routes.map(Option::unwrap);
    for route in routes {
        if route.owner != h.owner
            || route.subtracts_from != h.owner
            || (route.bounds.max().metres() - route.bounds.min().metres()).min_element() < 0.1
            || plan.resolved_geometry.solids.iter().any(|s| {
                crate::solid_overlap::overlaps_bounds(
                    s,
                    (route.bounds.min().metres(), route.bounds.max().metres()),
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
                (a.max().metres().min(b.max().metres()) - a.min().metres().max(b.min().metres()))
                    .min_element()
                    > GEOMETRY_TOLERANCE_METRES
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
    flue_shell(plan, h, routes[4].bounds, issues)?;

    Ok(())
}
fn flue_shell(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    bore: SpatialBounds<Architectural>,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
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
        return Ok(());
    }
    let mut outer = flues[0].cuboid_bounds()?;
    for solid in &flues {
        outer = outer.union(solid.cuboid_bounds()?);
    }
    let expected = rect(outer).difference(&rect(bore));
    let _: () = for fraction in [0.01, 0.5, 0.99] {
        let y =
            outer.min().metres().y + (outer.max().metres().y - outer.min().metres().y) * fraction;
        let mut actual = geo::MultiPolygon::new(vec![]);
        for solid in &flues {
            let b = solid.cuboid_bounds()?;
            if b.min().metres().y <= y && b.max().metres().y >= y {
                actual = actual.union(&rect(b));
            }
        }
        if expected.difference(&actual).unsigned_area() > AREA_TOLERANCE_SQUARE_METRES
            || actual.intersection(&rect(bore)).unsigned_area() > AREA_TOLERANCE_SQUARE_METRES
        {
            fail(
                issues,
                "open_domestic_flue_shell",
                "a section of the masonry flue is open or blocks its bore",
            );
            break;
        }
    };
    Ok(())
}
fn rect(bounds: SpatialBounds<Architectural>) -> geo::Polygon<f32> {
    geo::Rect::new(
        geo::coord! {x:bounds.min().metres().x,y:bounds.min().metres().z},
        geo::coord! {x:bounds.max().metres().x,y:bounds.max().metres().z},
    )
    .to_polygon()
}
fn polygon(points: &[Vec3]) -> geo::Polygon<f32> {
    geo::Polygon::new(
        geo::LineString::new(points.iter().map(|v| geo::coord! {x:v.x,y:v.z}).collect()),
        vec![],
    )
}

fn bearings(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let _: () = for part in &h.parts {
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
                if solid.cuboid_bounds()?.min().metres().y > GEOMETRY_TOLERANCE_METRES {
                    fail(
                        issues,
                        "detached_heating_bearing",
                        "a heating part above ground cannot declare a foundation",
                    );
                }
                continue;
            }
            if node.supported_by.is_empty()
                || crate::geometry_index::try_any(node.supported_by.iter(), |parent| {
                    Ok::<bool, crate::GenerationError>(!crate::geometry_index::try_any(
                        plan.resolved_geometry
                            .solids
                            .iter()
                            .filter(|s| s.id != solid.id && s.supported_by.contains(parent)),
                        |support| {
                            crate::geometry_index::try_any(
                                super::contact::measured(solid, support)?,
                                |contact| {
                                    Ok::<bool, crate::GenerationError>(
                                        plan.resolved_geometry.support_interfaces.iter().any(|i| {
                                            i.node == *node_id
                                                && i.owner == h.owner
                                                && i.bounds
                                                    .min()
                                                    .metres()
                                                    .distance(contact.min().metres())
                                                    < GEOMETRY_TOLERANCE_METRES
                                                && i.bounds
                                                    .max()
                                                    .metres()
                                                    .distance(contact.max().metres())
                                                    < GEOMETRY_TOLERANCE_METRES
                                        }),
                                    )
                                },
                            )
                        },
                    )?)
                })?
            {
                fail(
                    issues,
                    "detached_heating_bearing",
                    "heating support must contact actual finished masonry or sheet geometry",
                );
            }
        }
    };
    Ok(())
}
