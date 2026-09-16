//! Independent checks of downward support and finished occupied-floor openings.
use super::*;
use crate::heating::floors::{
    CLOSURE_LAP_METRES, CLOSURE_THICKNESS_METRES, MASONRY_BEARING_METRES,
};

pub(super) fn audit(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    support(plan, h, issues);
    shoulders(plan, h, issues);
    if h.floors.len() != plan.storeys.iter().filter(|s| s.level > 0).count() {
        fail(
            issues,
            "incomplete_heating_floor_penetrations",
            "each occupied floor needs a finished masonry penetration",
        );
    }
    for storey in plan.storeys.iter().filter(|s| s.level > 0) {
        let records = h
            .floors
            .iter()
            .filter(|f| f.storey_level == storey.level)
            .collect::<Vec<_>>();
        if records.len() != 1 {
            fail(
                issues,
                "incomplete_heating_floor_penetrations",
                "floor penetration ownership must be unique",
            );
            continue;
        }
        if !super::super::floor_bearings::valid(plan, storey.level) {
            fail(
                issues,
                "detached_heating_floor_bearing",
                "floor contacts must belong to the joists they actually intersect",
            );
        }
        opening(plan, h, records[0], issues);
    }
}

fn part<'a>(
    plan: &'a BuildingPlan,
    h: &DomesticHeatingPlan,
    kind: HeatingPartKind,
) -> Vec<&'a ResolvedSolid> {
    h.parts
        .iter()
        .filter(|p| p.kind == kind)
        .filter_map(|p| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == p.solid)
        })
        .collect()
}

fn support(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    let footings = part(plan, h, HeatingPartKind::Footing);
    let piers = part(plan, h, HeatingPartKind::SupportPier);
    if footings.len() != 1 || piers.len() != usize::from(h.kitchen.storey_level > 0) {
        fail(
            issues,
            "unsupported_upper_heating",
            "the appliance needs one plinth and a continuous pier when elevated",
        );
        return;
    }
    let footing = footings[0].cuboid_bounds();
    if (footing.min.y - h.floor_height_metres).abs() > GEOMETRY_TOLERANCE_METRES {
        fail(
            issues,
            "unsupported_upper_heating",
            "the appliance plinth must stand at the occupied floor elevation",
        );
    }
    if let Some(pier) = piers.first() {
        let bounds = pier.cuboid_bounds();
        let expected = rect(footing);
        let bearing = rect(bounds);
        let valid = bounds.min.y.abs() < GEOMETRY_TOLERANCE_METRES
            && (bounds.max.y - footing.min.y).abs() < GEOMETRY_TOLERANCE_METRES
            && expected.difference(&bearing).unsigned_area() < 0.00001
            && pier.supported_by.contains(&h.ground_support)
            && footings[0].supported_by.iter().any(|id| {
                plan.resolved_geometry.structural_nodes.iter().any(|node| {
                    node.id == *id
                        && !node.grounded
                        && node.supported_by.contains(&h.ground_support)
                })
            });
        if !valid {
            fail(
                issues,
                "unsupported_upper_heating",
                "the entire plinth must bear downward on continuous ground-founded masonry",
            );
        }
        let margin = Vec3::new(
            super::super::placement::TIMBER_CLEARANCE_METRES,
            0.0,
            super::super::placement::TIMBER_CLEARANCE_METRES,
        );
        if plan.resolved_geometry.solids.iter().any(|s| {
            !h.parts.iter().any(|p| p.solid == s.id)
                && crate::solid_overlap::overlaps_bounds(
                    s,
                    (bounds.min - margin, bounds.max + margin),
                    GEOMETRY_TOLERANCE_METRES,
                )
        }) {
            fail(
                issues,
                "blocked_heating_support",
                "the ground-to-kitchen pier or its clearance contains retained construction",
            );
        }
    }
}

fn opening(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    opening: &HeatingFloorPenetration,
    issues: &mut Vec<AuditIssue>,
) {
    let solids = if opening.storey_level <= h.kitchen.storey_level {
        part(plan, h, HeatingPartKind::Footing)
    } else {
        part(plan, h, HeatingPartKind::Flue)
    };
    let Some(first) = solids.first() else {
        return;
    };
    let mut core = solids.iter().fold(first.cuboid_bounds(), |a, s| {
        let b = s.cuboid_bounds();
        ResolvedBounds {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    });
    core.max.y = f32::from(opening.storey_level) * plan.storey_height_metres;
    core.min.y = core.max.y - 0.16;
    let margin = Vec3::new(
        super::super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
        0.0,
        super::super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
    );
    let cut = ResolvedBounds {
        min: core.min - margin,
        max: core.max + margin,
    };
    if opening.core.min.distance(core.min) > GEOMETRY_TOLERANCE_METRES
        || opening.core.max.distance(core.max) > GEOMETRY_TOLERANCE_METRES
        || opening.cut.min.distance(cut.min) > GEOMETRY_TOLERANCE_METRES
        || opening.cut.max.distance(cut.max) > GEOMETRY_TOLERANCE_METRES
        || plan.resolved_geometry.solids.iter().any(|s| {
            !h.parts.iter().any(|p| p.solid == s.id)
                && crate::solid_overlap::overlaps_bounds(
                    s,
                    (cut.min, cut.max),
                    GEOMETRY_TOLERANCE_METRES,
                )
        })
    {
        fail(
            issues,
            "blocked_heating_floor_penetration",
            "retained floors and timber must clear the actual masonry section",
        );
    }
    closures(plan, h, opening, core, cut, issues);
}

fn closures(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    opening: &HeatingFloorPenetration,
    core: ResolvedBounds,
    cut: ResolvedBounds,
    issues: &mut Vec<AuditIssue>,
) {
    let lap = Vec3::new(CLOSURE_LAP_METRES, 0.0, CLOSURE_LAP_METRES);
    let outer = ResolvedBounds {
        min: cut.min - lap,
        max: cut.max + lap,
    };
    let expected = rect(outer).difference(&rect(core));
    let mut actual = geo::MultiPolygon::new(vec![]);
    let mut supported = true;
    for id in &opening.closures {
        let solid = plan.resolved_geometry.solids.iter().find(|s| s.id == *id);
        let Some(solid) = solid else {
            supported = false;
            continue;
        };
        let bounds = solid.cuboid_bounds();
        if !h.parts.iter().any(|p| {
            p.solid == *id
                && p.kind == HeatingPartKind::FloorClosure
                && p.material == BuildingLodMaterial::Earthenware
        }) || (bounds.min.y - core.max.y).abs() > GEOMETRY_TOLERANCE_METRES
            || (bounds.max.y - bounds.min.y - CLOSURE_THICKNESS_METRES).abs()
                > GEOMETRY_TOLERANCE_METRES
        {
            supported = false;
        }
        actual = actual.union(&rect(bounds));
        supported &= cover_bearings(plan, h, core, cut, outer, bounds);
    }
    if !supported
        || expected.difference(&actual).unsigned_area() > 0.00001
        || actual.difference(&expected).unsigned_area() > 0.00001
    {
        fail(
            issues,
            "open_heating_floor_clearance",
            "supported mineral slabs must close the complete annular floor clearance",
        );
    }
}

fn shoulders(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    for shoulder in part(plan, h, HeatingPartKind::FlueShoulder) {
        let bounds = shoulder.cuboid_bounds();
        let mut bearing = geo::MultiPolygon::new(vec![]);
        for support in h.parts.iter().filter_map(|p| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == p.solid)
        }) {
            let b = support.cuboid_bounds();
            if support.id != shoulder.id
                && (b.max.y - bounds.min.y).abs() < GEOMETRY_TOLERANCE_METRES
            {
                bearing = bearing.union(&rect(b));
            }
        }
        if rect(bounds).difference(&bearing).unsigned_area() > 0.00001 {
            fail(
                issues,
                "unsupported_flue_shoulder",
                "the widened lower flue must bear downward on the masonry hood",
            );
        }
    }
}

fn cover_bearings(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    core: ResolvedBounds,
    cut: ResolvedBounds,
    outer: ResolvedBounds,
    bounds: ResolvedBounds,
) -> bool {
    let bearing_at = bounds.min.y;
    let mut deck = geo::MultiPolygon::new(vec![]);
    let mut masonry = geo::MultiPolygon::new(vec![]);
    for support in &plan.resolved_geometry.solids {
        let b = support.cuboid_bounds();
        if (b.max.y - bearing_at).abs() > GEOMETRY_TOLERANCE_METRES {
            continue;
        }
        if support.role == SolidRole::FrameFloor {
            deck = deck.union(&rect(b));
        }
        if h.parts.iter().any(|p| {
            p.solid == support.id
                && matches!(
                    p.kind,
                    HeatingPartKind::SupportPier | HeatingPartKind::FlueShoulder
                )
        }) {
            masonry = masonry.union(&rect(b));
        }
    }
    let ledge = Vec3::new(MASONRY_BEARING_METRES, 0.0, MASONRY_BEARING_METRES);
    let inner_edge = ResolvedBounds {
        min: core.min - ledge,
        max: core.max + ledge,
    };
    let required_inner = rect(inner_edge)
        .difference(&rect(core))
        .intersection(&rect(bounds));
    let required_outer = rect(outer)
        .difference(&rect(cut))
        .intersection(&rect(bounds));
    required_inner.unsigned_area() > 0.001
        && required_outer.unsigned_area() > 0.001
        && required_inner.difference(&masonry).unsigned_area() < 0.00001
        && required_outer.difference(&deck).unsigned_area() < 0.00001
}
