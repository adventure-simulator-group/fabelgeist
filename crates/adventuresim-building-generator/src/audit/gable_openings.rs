//! Apertures contribute coverage only with intact geometry and reciprocal owners.
use bevy::math::Vec3;

use crate::{
    BuildingPlan, ClosureState, OpeningAssembly, ResolvedSolid, WallAssembly, WallSourceId,
};

const POSITION_TOLERANCE_METRES: f32 = 0.001;

pub(super) fn valid(plan: &BuildingPlan, wall: &WallAssembly, opening: &OpeningAssembly) -> bool {
    let WallSourceId::RoofGable { roof, enclosure } = wall.source else {
        return false;
    };
    let linked = plan
        .roof_assemblies
        .iter()
        .find(|r| r.id == roof)
        .is_some_and(|roof| {
            roof.parent.is_none()
                && roof.enclosure_faces.iter().any(|face| {
                    face.id == enclosure
                        && face.inset_walls == [wall.id]
                        && (Vec3::new(wall.frame.origin.x, 0.0, wall.frame.origin.y)
                            .dot(face.normal())
                            + wall.thickness_metres * 0.5
                            - face.polygon[0].dot(face.normal()))
                        .abs()
                            < POSITION_TOLERANCE_METRES
                })
        });
    let half = opening.frame.tangent.abs() * opening.profile.exterior_width_metres() * 0.5
        + opening.frame.outward.abs() * wall.thickness_metres * 0.5;
    let origin = opening.frame.origin;
    let min = Vec3::new(
        origin.x - half.x,
        opening.sill_elevation_metres,
        origin.y - half.y,
    );
    let max = Vec3::new(
        origin.x + half.x,
        opening.sill_elevation_metres + opening.profile.clear_height_metres(),
        origin.y + half.y,
    );
    let void_matches = plan
        .resolved_geometry
        .voids
        .iter()
        .find(|v| v.id == opening.void_id)
        .is_some_and(|v| {
            v.owner == opening.owner
                && v.subtracts_from == wall.owner
                && v.bounds.min.distance(min) < POSITION_TOLERANCE_METRES
                && v.bounds.max.distance(max) < POSITION_TOLERANCE_METRES
        });
    let intact = opening
        .jamb_solids
        .into_iter()
        .chain([opening.spandrel_solid])
        .chain(opening.sill_solid)
        .all(|id| {
            plan.resolved_geometry
                .solids
                .iter()
                .any(|s| s.id == id && s.owner == wall.owner && material_depth_matches(wall, s))
        });
    let frame_clear = plan.timber_frame.as_ref().is_some_and(|frame| {
        frame.members.iter().all(|member| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == member.solid)
                .is_some_and(|s| {
                    !crate::solid_overlap::overlaps_bounds(s, (min, max), POSITION_TOLERANCE_METRES)
                })
        })
    });
    linked
        && opening.host_wall == wall.id
        && opening.host_source == wall.source
        && wall.opening_ids == [opening.id]
        && opening.frame.inside_room.is_none()
        && opening.closure.state == ClosureState::Closed
        && void_matches
        && intact
        && frame_clear
        && shared_head(plan, opening)
        && fixed_glass(plan, wall, opening)
}

fn fixed_glass(plan: &BuildingPlan, wall: &WallAssembly, opening: &OpeningAssembly) -> bool {
    if opening.closure_solids.len() != 1
        || opening.closure.layers != [crate::ClosureKind::LeadedGlazing]
    {
        return false;
    }
    let Some(solid) = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|s| s.id == opening.closure_solids[0])
    else {
        return false;
    };
    let tangent = Vec3::new(opening.frame.tangent.x, 0.0, opening.frame.tangent.y);
    let outward = Vec3::new(opening.frame.outward.x, 0.0, opening.frame.outward.y);
    let origin = Vec3::new(
        opening.frame.origin.x,
        opening.sill_elevation_metres,
        opening.frame.origin.y,
    );
    let bounds = solid.cuboid_bounds();
    let size = bounds.max - bounds.min;
    solid.owner == opening.owner
        && solid.role == crate::SolidRole::LeadedGlazing
        && solid.yaw_radians == 0.0
        && solid.crossfall_radians == 0.0
        && solid.longfall_radians == 0.0
        && (size.dot(outward.abs()) - crate::FIXED_GABLE_GLAZING_DEPTH_METRES).abs()
            < POSITION_TOLERANCE_METRES
        && opening.closure.swing_clearance_metres == 0.0
        && matches!(solid.shape, crate::ResolvedSolidShape::Cuboid)
        && (size.dot(tangent.abs()) - opening.profile.exterior_width_metres()).abs()
            < POSITION_TOLERANCE_METRES
        && (size.y - opening.profile.clear_height_metres()).abs() < POSITION_TOLERANCE_METRES
        && (solid.centre - origin).dot(tangent).abs() < POSITION_TOLERANCE_METRES
        && (solid.centre.y - origin.y - size.y * 0.5).abs() < POSITION_TOLERANCE_METRES
        && (solid.centre - origin).dot(outward).abs() + size.dot(outward.abs()) * 0.5
            <= wall.thickness_metres * 0.5
}

pub(super) fn material<'a>(plan: &'a BuildingPlan, wall: &WallAssembly) -> Vec<&'a ResolvedSolid> {
    let mut ids = wall.host_solids.clone();
    if let Some(frame) = &plan.timber_frame {
        for member in frame
            .bays
            .iter()
            .filter(|b| b.wall == Some(wall.id))
            .flat_map(|b| &b.member_ids)
        {
            if let Some(member) = frame.members.iter().find(|m| m.id == *member) {
                ids.push(member.solid);
            }
        }
    }
    plan.resolved_geometry
        .solids
        .iter()
        .filter(|s| ids.contains(&s.id))
        .collect()
}

pub(super) fn shared_head(plan: &BuildingPlan, opening: &OpeningAssembly) -> bool {
    let crate::OpeningHeadKind::TimberFrameMember { member } = opening.head_kind else {
        return false;
    };
    matches!(opening.host_source, WallSourceId::RoofGable { .. })
        && plan.timber_frame.as_ref().is_some_and(|frame| {
            frame.members.iter().any(|m| {
                m.id == member
                    && m.solid == opening.head_solid
                    && m.structural
                    && matches!(
                        m.role,
                        crate::TimberMemberRole::Rail | crate::TimberMemberRole::Collar
                    )
                    && plan
                        .resolved_geometry
                        .solids
                        .iter()
                        .any(|s| s.id == m.solid && s.owner == m.owner)
            })
        })
}

pub(super) fn material_depth_matches(wall: &WallAssembly, solid: &ResolvedSolid) -> bool {
    let n = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
    let bounds = solid.cuboid_bounds();
    let centre = (bounds.min + bounds.max) * 0.5;
    let radius = (bounds.max - bounds.min).dot(n.abs()) * 0.5;
    let plane = wall.frame.origin.dot(wall.frame.outward);
    (centre.dot(n) - plane).abs() < radius + wall.thickness_metres * 0.5 - POSITION_TOLERANCE_METRES
}
