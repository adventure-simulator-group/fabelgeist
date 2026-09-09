//! Retain both sides when an attached roof interrupts an internal ridge or hip.
use super::*;

const CUT_EDGE_CLEARANCE_METRES: f32 = 0.08;
const CUT_PARALLEL_EPSILON: f32 = 0.000_001;
const SPLIT_CAP_SLOT_BASE: u64 = 0xF0_0000;

pub(super) fn split_internal_edges(
    assembly: &mut RoofAssembly,
    cut: ResolvedBounds,
    geometry: &mut ResolvedGeometry,
) {
    let mut added = Vec::new();
    let initial_edges = assembly.edges.len();
    for edge in &mut assembly.edges {
        if !matches!(edge.kind, RoofEdgeKind::Ridge | RoofEdgeKind::Hip) {
            continue;
        }
        let Some((low, high)) = interior_cut(edge.start, edge.end, cut) else {
            continue;
        };
        let old_end = edge.end;
        let mut second = edge.clone();
        let delta = old_end - edge.start;
        let clearance = CUT_EDGE_CLEARANCE_METRES / delta.length();
        edge.end = edge.start + delta * (low - clearance).max(0.0);
        second.start += delta * (high + clearance).min(1.0);
        second.id = ResolvedItemId(
            (0xB_u64 << 60)
                | (assembly.id.0 << 32)
                | SPLIT_CAP_SLOT_BASE
                | (initial_edges + added.len()) as u64,
        );
        second.flashing = None;
        added.push(second);
    }
    assembly.edges.extend(added);
    split_caps(assembly.owner, cut, geometry);
}

fn split_caps(owner: GeometryOwnerId, cut: ResolvedBounds, geometry: &mut ResolvedGeometry) {
    let mut added = Vec::new();
    let mut interfaces = Vec::new();
    let initial_solids = geometry.solids.len();
    for solid in geometry
        .solids
        .iter_mut()
        .filter(|solid| solid.owner == owner && solid.role == SolidRole::RoofEdgeTreatment)
    {
        let axis = Vec3::new(
            solid.yaw_radians.cos() * solid.longfall_radians.cos(),
            solid.longfall_radians.sin(),
            solid.yaw_radians.sin() * solid.longfall_radians.cos(),
        );
        let start = solid.centre - axis * solid.size.x * 0.5;
        let end = solid.centre + axis * solid.size.x * 0.5;
        let Some((low, high)) = interior_cut(start, end, cut) else {
            continue;
        };
        let length = solid.size.x;
        let mut second = solid.clone();
        let first_length = low * length - CUT_EDGE_CLEARANCE_METRES;
        let second_length = (1.0 - high) * length - CUT_EDGE_CLEARANCE_METRES;
        if first_length <= 0.0 || second_length <= 0.0 {
            continue;
        }
        solid.centre = start + axis * first_length * 0.5;
        solid.size.x = first_length;
        second.centre = end - axis * second_length * 0.5;
        second.size.x = second_length;
        second.id = ResolvedItemId(
            (0x8_u64 << 60)
                | (u64::from(owner.0) << 32)
                | SPLIT_CAP_SLOT_BASE
                | (initial_solids + added.len()) as u64,
        );
        for support in &second.supported_by {
            interfaces.push(SupportInterface {
                id: ResolvedItemId((0x9_u64 << 60) | (second.id.0 & ((1_u64 << 60) - 1))),
                owner,
                node: *support,
                bounds: ResolvedBounds {
                    min: second.centre - Vec3::new(0.08, 0.025, 0.08),
                    max: second.centre + Vec3::new(0.08, 0.025, 0.08),
                },
            });
        }
        added.push(second);
    }
    geometry.solids.extend(added);
    geometry.support_interfaces.extend(interfaces);
}

fn interior_cut(start: Vec3, end: Vec3, cut: ResolvedBounds) -> Option<(f32, f32)> {
    let delta = end - start;
    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for (origin, direction, min, max) in [
        (start.x, delta.x, cut.min.x, cut.max.x),
        (start.y, delta.y, cut.min.y, cut.max.y),
        (start.z, delta.z, cut.min.z, cut.max.z),
    ] {
        if direction.abs() <= CUT_PARALLEL_EPSILON {
            if origin < min || origin > max {
                return None;
            }
        } else {
            let a = (min - origin) / direction;
            let b = (max - origin) / direction;
            low = low.max(a.min(b));
            high = high.min(a.max(b));
        }
    }
    (low > 0.0 && high < 1.0 && low < high).then_some((low, high))
}
