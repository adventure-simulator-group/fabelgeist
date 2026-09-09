//! Physical support interfaces must overlap their owning solid, including its full rotation.
use crate::{AuditIssue, ResolvedSolid, SolidRole, SupportInterface};

use super::{bounds_overlap_3d, issue, resolved_solid_bounds, resolved_solid_overlaps_bounds};

const MINIMUM_BEARING_OVERLAP_METRES: f32 = 0.001;

pub(super) fn audit_positive_bearing(
    solid: &ResolvedSolid,
    interfaces: &[SupportInterface],
    issues: &mut Vec<AuditIssue>,
) {
    let oriented_cuboid_member = matches!(
        solid.role,
        SolidRole::FrameSill
            | SolidRole::FramePost
            | SolidRole::FramePlate
            | SolidRole::FrameRail
            | SolidRole::FrameJoist
            | SolidRole::FrameGirder
            | SolidRole::FrameTie
            | SolidRole::FrameBrace
            | SolidRole::FrameJettyBeam
            | SolidRole::FrameKnagge
            | SolidRole::FrameGableMember
            | SolidRole::FrameDormerTrimmer
            | SolidRole::FrameOrnament
            | SolidRole::WorkplacePart
    );
    let has_bearing = interfaces.iter().any(|bearing| {
        bearing.owner == solid.owner
            && solid.supported_by.contains(&bearing.node)
            && if oriented_cuboid_member {
                resolved_solid_overlaps_bounds(
                    solid,
                    (bearing.bounds.min, bearing.bounds.max),
                    MINIMUM_BEARING_OVERLAP_METRES,
                )
            } else {
                bounds_overlap_3d(
                    resolved_solid_bounds(solid),
                    (bearing.bounds.min, bearing.bounds.max),
                    MINIMUM_BEARING_OVERLAP_METRES,
                )
            }
    });
    if !has_bearing {
        issues.push(issue(
            "missing_positive_bearing",
            format!(
                "resolved solid {} has no positive bearing interface",
                solid.id.0
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GeometryOwnerId, ResolvedBounds, ResolvedItemId, ResolvedSolidShape, StructuralNodeId,
    };
    use bevy::math::{Quat, Vec3};

    fn pitched_member() -> (ResolvedSolid, SupportInterface) {
        let solid = ResolvedSolid {
            id: ResolvedItemId(1),
            owner: GeometryOwnerId(1),
            centre: Vec3::new(0.0, 1.5, 0.0),
            size: Vec3::new(0.4, 0.2, 3.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.6,
            longfall_radians: 0.0,
            role: SolidRole::WorkplacePart,
            shape: ResolvedSolidShape::Cuboid,
            supported_by: vec![StructuralNodeId(1)],
        };
        let rotation = Quat::from_rotation_x(solid.crossfall_radians);
        let half = solid.size * 0.5;
        let extent = (rotation * Vec3::X).abs() * half.x
            + (rotation * Vec3::Y).abs() * half.y
            + (rotation * Vec3::Z).abs() * half.z;
        let min = solid.centre - extent;
        let max = solid.centre + extent;
        let interface = SupportInterface {
            id: ResolvedItemId(2),
            owner: solid.owner,
            node: solid.supported_by[0],
            bounds: ResolvedBounds {
                min,
                max: Vec3::new(max.x, min.y + 0.04, max.z),
            },
        };
        (solid, interface)
    }

    #[test]
    fn pitched_working_member_uses_its_actual_low_end_bearing() {
        let (solid, interface) = pitched_member();
        assert!(interface.bounds.max.y < solid.centre.y - solid.size.y * 0.5);
        let mut issues = Vec::new();
        audit_positive_bearing(&solid, &[interface], &mut issues);
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn bearing_requires_physical_overlap_and_matching_owner_and_node() {
        let (solid, interface) = pitched_member();
        let mut wrong_owner = interface.clone();
        wrong_owner.owner = GeometryOwnerId(2);
        let mut wrong_node = interface.clone();
        wrong_node.node = StructuralNodeId(2);
        let mut detached = interface;
        detached.bounds.min.x += 10.0;
        detached.bounds.max.x += 10.0;
        for invalid in [wrong_owner, wrong_node, detached] {
            let mut issues = Vec::new();
            audit_positive_bearing(&solid, &[invalid], &mut issues);
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].code, "missing_positive_bearing");
        }
    }
}
