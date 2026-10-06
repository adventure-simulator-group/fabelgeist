//! Open deck boards only where the complete masonry fits between retained members.
use super::{assembly::Assembly, placement::Placement};
use crate::*;
use bevy::math::Vec3;

pub(super) const MASONRY_BEARING_METRES: f32 = 0.04;
pub(super) const CLOSURE_LAP_METRES: f32 = 0.05;
pub(super) const CLOSURE_THICKNESS_METRES: f32 = 0.02;
const CUT_TOLERANCE_METRES: f32 = 0.001;

pub(super) fn openings(
    plan: &BuildingPlan,
    placement: Placement,
) -> Result<Vec<HeatingFloorPenetration>, crate::GenerationError> {
    plan.storeys
        .iter()
        .filter(|s| s.level > 0)
        .map(|storey| {
            let elevation = f32::from(storey.level) * plan.storey_height_metres;
            let body = if StoreyIndex::from_serialized(storey.level) <= placement.site.storey_level
            {
                placement.site.body()?
            } else {
                placement
                    .site
                    .shaft(crate::spatial_geometry::Elevation::from_metres(elevation)?)?
            };
            let mut min = body.min().metres();
            let mut max = body.max().metres();
            min.y = elevation - 0.16;
            max.y = elevation;
            let core = SpatialBounds::from_metres(min, max)?;
            let margin = Vec3::new(
                super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
                0.0,
                super::placement::TIMBER_CLEARANCE_METRES + MASONRY_BEARING_METRES,
            );
            Ok(HeatingFloorPenetration {
                storey_level: StoreyIndex::from_serialized(storey.level),
                core,
                cut: SpatialBounds::from_metres(
                    core.min().metres() - margin,
                    core.max().metres() + margin,
                )?,
                closures: vec![],
            })
        })
        .collect()
}

/// Horizontal strips retain their full thickness; no hidden subtractive voids.
pub(super) fn pieces(
    source: SpatialBounds<Architectural>,
    cut: SpatialBounds<Architectural>,
) -> Result<Vec<SpatialBounds<Architectural>>, crate::GenerationError> {
    let min = source.min().metres().max(cut.min().metres());
    let max = source.max().metres().min(cut.max().metres());
    if (max - min).min_element() <= CUT_TOLERANCE_METRES {
        return Ok(vec![source]);
    }
    let mut result = vec![];
    let mut middle_min = source.min().metres();
    let mut middle_max = source.max().metres();
    for axis in [2, 0] {
        if min[axis] > middle_min[axis] + CUT_TOLERANCE_METRES {
            let mut piece_max = middle_max;
            piece_max[axis] = min[axis];
            result.push(SpatialBounds::from_metres(middle_min, piece_max)?);
        }
        if max[axis] < middle_max[axis] - CUT_TOLERANCE_METRES {
            let mut piece_min = middle_min;
            piece_min[axis] = max[axis];
            result.push(SpatialBounds::from_metres(piece_min, middle_max)?);
        }
        middle_min[axis] = min[axis];
        middle_max[axis] = max[axis];
    }
    Ok(result)
}

pub(super) fn supported(
    plan: &BuildingPlan,
    placement: Placement,
) -> Result<bool, crate::GenerationError> {
    let Some(frame) = &plan.timber_frame else {
        return Ok(placement.site.storey_level == StoreyIndex::GROUND);
    };
    for opening in openings(plan, placement)? {
        let Some(floor) = frame
            .floors
            .iter()
            .find(|f| StoreyIndex::from_serialized(f.level) == opening.storey_level)
        else {
            return Ok(false);
        };
        let mut remaining = Vec::new();
        for id in &floor.floor_solids {
            let Some(solid) = plan.resolved_geometry.solids.iter().find(|s| s.id == *id) else {
                continue;
            };
            remaining.extend(pieces(solid.cuboid_bounds()?, opening.cut)?);
        }
        for &bounds in &remaining {
            if super::floor_bearings::contacts(
                &plan.resolved_geometry,
                &frame.members,
                floor,
                bounds,
            )?
            .is_empty()
            {
                return Ok(false);
            }
        }
        for id in &floor.floor_joist_interfaces {
            let Some(interface) = plan
                .resolved_geometry
                .support_interfaces
                .iter()
                .find(|i| i.id == *id)
            else {
                continue;
            };
            if !remaining.iter().any(|b| {
                (b.max().metres().min(interface.bounds.max().metres())
                    - b.min().metres().max(interface.bounds.min().metres()))
                .min_element()
                    > CUT_TOLERANCE_METRES
            }) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

pub(super) fn cut(
    plan: &mut BuildingPlan,
    placement: Placement,
) -> Result<Vec<HeatingFloorPenetration>, crate::GenerationError> {
    let openings = openings(plan, placement)?;
    let Some(frame) = &mut plan.timber_frame else {
        return Ok(openings);
    };
    let mut slot = 0_u64;
    for opening in &openings {
        let floor = frame
            .floors
            .iter_mut()
            .find(|f| StoreyIndex::from_serialized(f.level) == opening.storey_level)
            .ok_or(HeatingConstructionError::MissingFloor {
                storey: opening.storey_level,
            })?;
        let mut retained = vec![];
        for id in floor.floor_solids.clone() {
            let index = plan
                .resolved_geometry
                .solids
                .iter()
                .position(|s| s.id == id)
                .ok_or(HeatingConstructionError::MissingSolid { solid: id })?;
            let source = plan.resolved_geometry.solids.remove(index);
            for (index, bounds) in pieces(source.cuboid_bounds()?, opening.cut)?
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
                piece.centre =
                    crate::spatial_geometry::Position::<crate::Architectural>::from_metres(
                        (bounds.min().metres() + bounds.max().metres()) * 0.5,
                    )?;
                piece.size = crate::spatial_geometry::CuboidDimensions::from_metres(
                    bounds.max().metres() - bounds.min().metres(),
                )?;
                super::floor_bearings::attach(
                    &mut plan.resolved_geometry,
                    &frame.members,
                    floor,
                    &mut piece,
                    &mut slot,
                )?;
                retained.push(piece.id);
                plan.resolved_geometry.solids.push(piece);
            }
        }
        floor.floor_solids = retained;
        floor.floor_solid =
            floor
                .floor_solids
                .first()
                .copied()
                .ok_or(HeatingConstructionError::MissingDeck {
                    storey: opening.storey_level,
                })?;
    }
    Ok(openings)
}

/// Mineral cover slabs bridge the clearance band and bear on the retained deck.
pub(super) fn close(
    a: &mut Assembly<'_>,
    mut openings: Vec<HeatingFloorPenetration>,
) -> Result<(), crate::GenerationError> {
    for opening in &mut openings {
        let margin = Vec3::new(CLOSURE_LAP_METRES, 0.0, CLOSURE_LAP_METRES);
        let mut min = opening.cut.min().metres() - margin;
        let mut max = opening.cut.max().metres() + margin;
        min.y = opening.core.max().metres().y;
        max.y = min.y + CLOSURE_THICKNESS_METRES;
        let outer = SpatialBounds::<Architectural>::from_metres(min, max)?;
        let inner = SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                opening.core.min().metres().x,
                outer.min().metres().y,
                opening.core.min().metres().z,
            ),
            Vec3::new(
                opening.core.max().metres().x,
                outer.max().metres().y,
                opening.core.max().metres().z,
            ),
        )?;
        for bounds in pieces(outer, inner)? {
            opening.closures.push(a.absolute_part(
                HeatingPartKind::FloorClosure,
                BuildingLodMaterial::Earthenware,
                bounds,
            )?);
        }
    }
    a.plan.floors = openings;

    Ok(())
}
