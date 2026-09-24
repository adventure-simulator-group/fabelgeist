//! Open deck boards only where the complete masonry fits between retained members.
use super::{assembly::Assembly, placement::Placement};
use crate::*;
use bevy::math::Vec3;

pub(super) const MASONRY_BEARING_METRES: f32 = 0.04;
pub(super) const CLOSURE_LAP_METRES: f32 = 0.05;
pub(super) const CLOSURE_THICKNESS_METRES: f32 = 0.02;
const CUT_TOLERANCE_METRES: f32 = 0.001;

pub(super) fn openings(plan: &BuildingPlan, placement: Placement) -> Vec<HeatingFloorPenetration> {
    plan.storeys
        .iter()
        .filter(|s| s.level > 0)
        .map(|storey| {
            let elevation = f32::from(storey.level) * plan.storey_height_metres;
            let mut core = if storey.level <= placement.storey_level {
                placement.body()
            } else {
                placement.shaft(elevation)
            };
            core.min.y = elevation - 0.16;
            core.max.y = elevation;
            let margin = Vec3::new(
                super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
                0.0,
                super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
            );
            HeatingFloorPenetration {
                storey_level: storey.level,
                core,
                cut: ResolvedBounds {
                    min: core.min - margin,
                    max: core.max + margin,
                },
                closures: vec![],
            }
        })
        .collect()
}

/// Horizontal strips retain their full thickness; no hidden subtractive voids.
pub(super) fn pieces(source: ResolvedBounds, cut: ResolvedBounds) -> Vec<ResolvedBounds> {
    let min = source.min.max(cut.min);
    let max = source.max.min(cut.max);
    if (max - min).min_element() <= CUT_TOLERANCE_METRES {
        return vec![source];
    }
    let mut result = vec![];
    let mut middle = source;
    for axis in [2, 0] {
        if min[axis] > middle.min[axis] + CUT_TOLERANCE_METRES {
            let mut piece = middle;
            piece.max[axis] = min[axis];
            result.push(piece);
        }
        if max[axis] < middle.max[axis] - CUT_TOLERANCE_METRES {
            let mut piece = middle;
            piece.min[axis] = max[axis];
            result.push(piece);
        }
        middle.min[axis] = min[axis];
        middle.max[axis] = max[axis];
    }
    result
}

pub(super) fn supported(plan: &BuildingPlan, placement: Placement) -> bool {
    let Some(frame) = &plan.timber_frame else {
        return placement.storey_level == 0;
    };
    openings(plan, placement).iter().all(|opening| {
        let Some(floor) = frame
            .floors
            .iter()
            .find(|f| f.level == opening.storey_level)
        else {
            return false;
        };
        let remaining = floor
            .floor_solids
            .iter()
            .filter_map(|id| plan.resolved_geometry.solids.iter().find(|s| s.id == *id))
            .flat_map(|s| pieces(s.cuboid_bounds(), opening.cut))
            .collect::<Vec<_>>();
        let contacts = floor
            .floor_joist_interfaces
            .iter()
            .filter_map(|id| {
                plan.resolved_geometry
                    .support_interfaces
                    .iter()
                    .find(|i| i.id == *id)
            })
            .collect::<Vec<_>>();
        let touches = |b: ResolvedBounds, i: &&SupportInterface| {
            (b.max.min(i.bounds.max) - b.min.max(i.bounds.min)).min_element() > CUT_TOLERANCE_METRES
        };
        remaining.iter().all(|b| {
            !super::floor_bearings::contacts(&plan.resolved_geometry, &frame.members, floor, *b)
                .is_empty()
        }) && contacts
            .iter()
            .all(|i| remaining.iter().any(|b| touches(*b, i)))
    })
}

pub(super) fn cut(plan: &mut BuildingPlan, placement: Placement) -> Vec<HeatingFloorPenetration> {
    let openings = openings(plan, placement);
    let Some(frame) = &mut plan.timber_frame else {
        return openings;
    };
    let mut slot = 0_u64;
    for opening in &openings {
        let floor = frame
            .floors
            .iter_mut()
            .find(|f| f.level == opening.storey_level)
            .unwrap();
        let mut retained = vec![];
        for id in floor.floor_solids.clone() {
            let index = plan
                .resolved_geometry
                .solids
                .iter()
                .position(|s| s.id == id)
                .unwrap();
            let source = plan.resolved_geometry.solids.remove(index);
            for (index, bounds) in pieces(source.cuboid_bounds(), opening.cut)
                .into_iter()
                .enumerate()
            {
                let mut piece = source.clone();
                if index > 0 {
                    slot += 1;
                    piece.id = ResolvedItemId(
                        (1_u64 << 60) | (u64::from(source.owner.0) << 32) | 0x0950_0000 | slot,
                    );
                }
                piece.centre = (bounds.min + bounds.max) * 0.5;
                piece.size = bounds.max - bounds.min;
                super::floor_bearings::attach(
                    &mut plan.resolved_geometry,
                    &frame.members,
                    floor,
                    &mut piece,
                    &mut slot,
                );
                retained.push(piece.id);
                plan.resolved_geometry.solids.push(piece);
            }
        }
        floor.floor_solids = retained;
        floor.floor_solid = floor.floor_solids[0];
    }
    openings
}

/// Mineral cover slabs bridge the clearance band and bear on the retained deck.
pub(super) fn close(a: &mut Assembly<'_>, mut openings: Vec<HeatingFloorPenetration>) {
    for opening in &mut openings {
        let margin = Vec3::new(CLOSURE_LAP_METRES, 0.0, CLOSURE_LAP_METRES);
        let mut outer = ResolvedBounds {
            min: opening.cut.min - margin,
            max: opening.cut.max + margin,
        };
        outer.min.y = opening.core.max.y;
        outer.max.y = outer.min.y + CLOSURE_THICKNESS_METRES;
        let inner = ResolvedBounds {
            min: Vec3::new(opening.core.min.x, outer.min.y, opening.core.min.z),
            max: Vec3::new(opening.core.max.x, outer.max.y, opening.core.max.z),
        };
        for bounds in pieces(outer, inner) {
            opening.closures.push(a.absolute_part(
                HeatingPartKind::FloorClosure,
                BuildingLodMaterial::Earthenware,
                bounds,
            ));
        }
    }
    a.plan.floors = openings;
}
