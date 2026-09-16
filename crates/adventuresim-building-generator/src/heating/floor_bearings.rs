//! Re-measure deck bearings after board cuts; the joists and their joints stay intact.
use crate::*;
const INTERFACE_DEPTH_METRES: f32 = 0.004;
const MINIMUM_CONTACT_METRES: f32 = 0.002;

pub(super) fn contacts(
    geometry: &ResolvedGeometry,
    members: &[TimberFrameMember],
    floor: &TimberFloorAssembly,
    bounds: ResolvedBounds,
) -> Vec<(StructuralNodeId, ResolvedBounds)> {
    members
        .iter()
        .filter(|m| floor.joist_members.contains(&m.id))
        .filter_map(|member| {
            let solid = geometry.solids.iter().find(|s| s.id == member.solid)?;
            let joist = solid.cuboid_bounds();
            if (joist.max.y - bounds.min.y).abs() > MINIMUM_CONTACT_METRES {
                return None;
            }
            let mut min = joist.min.max(bounds.min);
            let mut max = joist.max.min(bounds.max);
            if max.x - min.x < MINIMUM_CONTACT_METRES || max.z - min.z < MINIMUM_CONTACT_METRES {
                return None;
            }
            min.y = bounds.min.y - INTERFACE_DEPTH_METRES;
            max.y = bounds.min.y + INTERFACE_DEPTH_METRES;
            Some((member.start_node, ResolvedBounds { min, max }))
        })
        .collect()
}

pub(super) fn attach(
    geometry: &mut ResolvedGeometry,
    members: &[TimberFrameMember],
    floor: &mut TimberFloorAssembly,
    piece: &mut ResolvedSolid,
    slot: &mut u64,
) {
    piece.supported_by.clear();
    for (node, bounds) in contacts(geometry, members, floor, piece.cuboid_bounds()) {
        *slot += 1;
        let id =
            ResolvedItemId((4_u64 << 60) | (u64::from(piece.owner.0) << 32) | 0x0950_0000 | *slot);
        geometry.support_interfaces.push(SupportInterface {
            id,
            owner: piece.owner,
            node,
            bounds,
        });
        floor.floor_joist_interfaces.push(id);
        floor.bearing_interfaces.push(id);
        piece.supported_by.push(node);
    }
    piece.supported_by.sort_unstable();
    piece.supported_by.dedup();
}

pub(super) fn valid(plan: &BuildingPlan, level: u16) -> bool {
    let Some(frame) = &plan.timber_frame else {
        return false;
    };
    let Some(floor) = frame.floors.iter().find(|f| f.level == level) else {
        return false;
    };
    floor.floor_joist_interfaces.iter().all(|id| {
        let Some(interface) = plan
            .resolved_geometry
            .support_interfaces
            .iter()
            .find(|i| i.id == *id)
        else {
            return false;
        };
        frame
            .members
            .iter()
            .filter(|m| floor.joist_members.contains(&m.id))
            .filter(|m| [m.start_node, m.end_node].contains(&interface.node))
            .any(|member| {
                plan.resolved_geometry.solids.iter().any(|solid| {
                    solid.id == member.solid
                        && crate::solid_overlap::overlaps_bounds(
                            solid,
                            (interface.bounds.min, interface.bounds.max),
                            MINIMUM_CONTACT_METRES,
                        )
                })
            })
    })
}
