//! Independent checks of downward support and finished occupied-floor openings.
use super::*;
use crate::GenerationResult as Result;
use crate::heating::floors::{
    CLOSURE_LAP_METRES, CLOSURE_THICKNESS_METRES, MASONRY_BEARING_METRES,
};

pub(super) fn audit(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    support(plan, h, issues)?;
    shoulders(plan, h, issues)?;
    if h.floors.len() != plan.storeys.iter().filter(|s| s.level > 0).count() {
        fail(
            issues,
            "incomplete_heating_floor_penetrations",
            "each occupied floor needs a finished masonry penetration",
        );
    }
    let _: () = for storey in plan.storeys.iter().filter(|s| s.level > 0) {
        let records = h
            .floors
            .iter()
            .filter(|f| f.storey_level == StoreyIndex::from_serialized(storey.level))
            .collect::<Vec<_>>();
        if records.len() != 1 {
            fail(
                issues,
                "incomplete_heating_floor_penetrations",
                "floor penetration ownership must be unique",
            );
            continue;
        }
        if !super::super::floor_bearings::valid(
            plan,
            crate::StoreyIndex::from_serialized(storey.level),
        ) {
            fail(
                issues,
                "detached_heating_floor_bearing",
                "floor contacts must belong to the joists they actually intersect",
            );
        }
        opening(plan, h, records[0], issues)?;
    };
    Ok(())
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

fn support(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let footings = part(plan, h, HeatingPartKind::Footing);
    let piers = part(plan, h, HeatingPartKind::SupportPier);
    if footings.len() != 1
        || piers.len() != usize::from(h.kitchen.storey_level > StoreyIndex::GROUND)
    {
        fail(
            issues,
            "unsupported_upper_heating",
            "the appliance needs one plinth and a continuous pier when elevated",
        );
        return Ok(());
    }
    let footing = footings[0].cuboid_bounds()?;
    if (footing.min().metres().y - h.floor_height_metres.metres()).abs() > GEOMETRY_TOLERANCE_METRES
    {
        fail(
            issues,
            "unsupported_upper_heating",
            "the appliance plinth must stand at the occupied floor elevation",
        );
    }
    let _: () = if let Some(pier) = piers.first() {
        let bounds = pier.cuboid_bounds()?;
        let expected = rect(footing);
        let bearing = rect(bounds);
        let valid = bounds.min().metres().y.abs() < GEOMETRY_TOLERANCE_METRES
            && (bounds.max().metres().y - footing.min().metres().y).abs()
                < GEOMETRY_TOLERANCE_METRES
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
                    (
                        bounds.min().metres() - margin,
                        bounds.max().metres() + margin,
                    ),
                    GEOMETRY_TOLERANCE_METRES,
                )
        }) {
            fail(
                issues,
                "blocked_heating_support",
                "the ground-to-kitchen pier or its clearance contains retained construction",
            );
        }
    };
    Ok(())
}

fn opening(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    opening: &HeatingFloorPenetration,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let solids = if opening.storey_level <= h.kitchen.storey_level {
        part(plan, h, HeatingPartKind::Footing)
    } else {
        part(plan, h, HeatingPartKind::Flue)
    };
    let Some(first) = solids.first() else {
        return Ok(());
    };
    let mut core = first.cuboid_bounds()?;
    for solid in &solids {
        core = core.union(solid.cuboid_bounds()?);
    }
    let mut min = core.min().metres();
    let mut max = core.max().metres();
    max.y = f32::from(opening.storey_level.serialized_ordinal()?) * plan.storey_height_metres;
    min.y = max.y - 0.16;
    let core = SpatialBounds::from_metres(min, max)?;
    let margin = Vec3::new(
        super::super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
        0.0,
        super::super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
    );
    let cut = SpatialBounds::<Architectural>::from_metres(
        core.min().metres() - margin,
        core.max().metres() + margin,
    )?;
    if opening.core.min().metres().distance(core.min().metres()) > GEOMETRY_TOLERANCE_METRES
        || opening.core.max().metres().distance(core.max().metres()) > GEOMETRY_TOLERANCE_METRES
        || opening.cut.min().metres().distance(cut.min().metres()) > GEOMETRY_TOLERANCE_METRES
        || opening.cut.max().metres().distance(cut.max().metres()) > GEOMETRY_TOLERANCE_METRES
        || plan.resolved_geometry.solids.iter().any(|s| {
            !h.parts.iter().any(|p| p.solid == s.id)
                && crate::solid_overlap::overlaps_bounds(
                    s,
                    (cut.min().metres(), cut.max().metres()),
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
    closures(plan, h, opening, core, cut, issues)?;

    Ok(())
}

fn closures(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    opening: &HeatingFloorPenetration,
    core: SpatialBounds<Architectural>,
    cut: SpatialBounds<Architectural>,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let lap = Vec3::new(CLOSURE_LAP_METRES, 0.0, CLOSURE_LAP_METRES);
    let outer = SpatialBounds::<Architectural>::from_metres(
        cut.min().metres() - lap,
        cut.max().metres() + lap,
    )?;
    let expected = rect(outer).difference(&rect(core));
    let mut actual = geo::MultiPolygon::new(vec![]);
    let mut supported = true;
    for id in &opening.closures {
        let solid = plan.resolved_geometry.solids.iter().find(|s| s.id == *id);
        let Some(solid) = solid else {
            supported = false;
            continue;
        };
        let bounds = solid.cuboid_bounds()?;
        if !h.parts.iter().any(|p| {
            p.solid == *id
                && p.kind == HeatingPartKind::FloorClosure
                && p.material == BuildingLodMaterial::Earthenware
        }) || (bounds.min().metres().y - core.max().metres().y).abs() > GEOMETRY_TOLERANCE_METRES
            || (bounds.max().metres().y - bounds.min().metres().y - CLOSURE_THICKNESS_METRES).abs()
                > GEOMETRY_TOLERANCE_METRES
        {
            supported = false;
        }
        actual = actual.union(&rect(bounds));
        supported &= cover_bearings(plan, h, core, cut, outer, bounds)?;
    }
    let _: () = if !supported
        || expected.difference(&actual).unsigned_area() > 0.00001
        || actual.difference(&expected).unsigned_area() > 0.00001
    {
        fail(
            issues,
            "open_heating_floor_clearance",
            "supported mineral slabs must close the complete annular floor clearance",
        );
    };
    Ok(())
}

fn shoulders(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let _: () = for shoulder in part(plan, h, HeatingPartKind::FlueShoulder) {
        let bounds = shoulder.cuboid_bounds()?;
        let mut bearing = geo::MultiPolygon::new(vec![]);
        for support in h.parts.iter().filter_map(|p| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == p.solid)
        }) {
            let b = support.cuboid_bounds()?;
            if support.id != shoulder.id
                && (b.max().metres().y - bounds.min().metres().y).abs() < GEOMETRY_TOLERANCE_METRES
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
    };
    Ok(())
}

fn cover_bearings(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    core: SpatialBounds<Architectural>,
    cut: SpatialBounds<Architectural>,
    outer: SpatialBounds<Architectural>,
    bounds: SpatialBounds<Architectural>,
) -> Result<bool> {
    let bearing_at = bounds.min().metres().y;
    let mut deck = geo::MultiPolygon::new(vec![]);
    let mut masonry = geo::MultiPolygon::new(vec![]);
    for support in &plan.resolved_geometry.solids {
        let b = support.cuboid_bounds()?;
        if (b.max().metres().y - bearing_at).abs() > GEOMETRY_TOLERANCE_METRES {
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
    let inner_edge = SpatialBounds::<Architectural>::from_metres(
        core.min().metres() - ledge,
        core.max().metres() + ledge,
    )?;
    let required_inner = rect(inner_edge)
        .difference(&rect(core))
        .intersection(&rect(bounds));
    let required_outer = rect(outer)
        .difference(&rect(cut))
        .intersection(&rect(bounds));
    Ok(required_inner.unsigned_area() > 0.001
        && required_outer.unsigned_area() > 0.001
        && required_inner.difference(&masonry).unsigned_area() < 0.00001
        && required_outer.difference(&deck).unsigned_area() < 0.00001)
}
