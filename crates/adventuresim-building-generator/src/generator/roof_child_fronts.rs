fn resolve_roof_child_front_openings(
    program: &BuildingProgram,
    dormers: &[RoofDormer],
    roofs: &mut [RoofAssembly],
    walls: &mut Vec<crate::WallAssembly>,
    openings: &mut Vec<crate::OpeningAssembly>,
    geometry: &mut ResolvedGeometry,
) {
    for (index, dormer) in dormers.iter().copied().enumerate() {
        let roof_id = RoofAssemblyId(1_000 + index as u64);
        let parent_owner = roofs
            .iter()
            .find(|roof| roof.id == roof_id)
            .and_then(|roof| roof.parent)
            .and_then(|parent| roofs.iter().find(|roof| roof.id == parent))
            .map(|roof| roof.owner);
        let parent_support_nodes = roofs
            .iter()
            .find(|roof| roof.id == roof_id)
            .and_then(|roof| roof.parent)
            .and_then(|parent| roofs.iter().find(|roof| roof.id == parent))
            .map(|roof| roof.support_nodes.clone())
            .unwrap_or_default();
        let Some(child) = roofs.iter_mut().find(|roof| roof.id == roof_id) else {
            continue;
        };
        let front_enclosure_id = ResolvedItemId((0xA_u64 << 60) | (roof_id.0 << 16) | 0x4100);
        let Some(front) = child
            .enclosure_faces
            .iter()
            .find(|face| face.id == front_enclosure_id)
            .cloned()
        else {
            continue;
        };
        child
            .enclosure_faces
            .retain(|face| face.id != front_enclosure_id);

        let wall_id = crate::WallAssemblyId(1_000_000 + index as u64);
        let opening_id = crate::OpeningAssemblyId(1_000_000 + index as u64);
        let owner = GeometryOwnerId(70_000 + index as u32);
        let outward = direction_vector(dormer.facing);
        let tangent = if outward.y.abs() > 0.5 {
            Vec2::X
        } else {
            Vec2::Y
        };
        let origin = dormer.centre;
        let width = front
            .polygon
            .iter()
            .map(|point| Vec2::new(point.x, point.z).dot(tangent))
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                (min.min(value), max.max(value))
            });
        let width = width.1 - width.0;
        let facade_wall = (dormer.kind == DormerKind::TransverseGable)
            .then(|| {
                walls
                    .iter()
                    .filter(|wall| {
                        matches!(wall.source, crate::WallSourceId::StoreyWall { .. })
                            && wall.frame.outside_room.is_none()
                            && wall.frame.outward.dot(outward) > 0.99
                    })
                    .min_by(|left, right| {
                        left.frame
                            .origin
                            .distance(origin)
                            .total_cmp(&right.frame.origin.distance(origin))
                    })
                    .map(|wall| (wall.id, wall.support_node))
            })
            .flatten();
        let base = front
            .polygon
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min);
        let top = front
            .polygon
            .iter()
            .map(|point| point.y)
            .fold(f32::NEG_INFINITY, f32::max);
        let height = (top - base).max(1.15);
        let thickness = 0.20;
        let wall_node = StructuralNodeId((u64::from(owner.0) << 16) | 1);
        geometry.structural_nodes.push(StructuralNode {
            id: wall_node,
            owner,
            kind: StructuralNodeKind::RoofWallPlate,
            position: Vec3::new(origin.x, base, origin.y),
            supported_by: facade_wall
                .map(|(_, node)| vec![node])
                .unwrap_or_else(|| parent_support_nodes.clone()),
            grounded: false,
        });
        // The child facade/cheeks carry the child roof; the parent roof carries
        // their curb/trimmers.  Do not reverse this edge (wall -> child roof),
        // which forms a semantic cycle and previously encouraged generic
        // ground-to-eave fallback posts.
        for roof_node_id in &child.support_nodes {
            if let Some(roof_node) = geometry
                .structural_nodes
                .iter_mut()
                .find(|node| node.id == *roof_node_id)
            {
                roof_node.supported_by.push(wall_node);
                roof_node.supported_by.sort_unstable();
                roof_node.supported_by.dedup();
            }
        }
        let (wall, opening) = roof_wall_opening::RectangularRoofWindow {
            wall_id, opening_id, owner,
            source: crate::WallSourceId::RoofChildFront { roof: roof_id },
            origin, tangent, outward, base, width, height, thickness, wall_node,
            opening_width: (width - 0.42).clamp(0.48, 0.82),
            clear_height: (height - 0.44).clamp(0.68, 1.12),
            sill_height: 0.22, head_height: 0.14, head_member: None,
            storey_level: program.storeys.len() as u16,
            ornamental_frame: program.archetype.timber_frame_program().is_none(),
        }.build(geometry);
        walls.push(wall);
        openings.push(opening);
        for (bond_slot, roof_owner) in parent_owner.into_iter().enumerate() {
            geometry.junction_bonds.push(JunctionBond {
                id: ResolvedItemId(
                    (0x6_u64 << 60) | (u64::from(owner.0) << 16) | (1 + bond_slot as u64),
                ),
                owners: [roof_owner, owner],
                bounds: ResolvedBounds {
                    min: Vec3::new(
                        origin.x - tangent.x.abs() * width * 0.55 - outward.x.abs() * 0.30,
                        base - 0.12,
                        origin.y - tangent.y.abs() * width * 0.55 - outward.y.abs() * 0.30,
                    ),
                    max: Vec3::new(
                        origin.x + tangent.x.abs() * width * 0.55 + outward.x.abs() * 0.30,
                        top + 0.18,
                        origin.y + tangent.y.abs() * width * 0.55 + outward.y.abs() * 0.30,
                    ),
                },
                minimum_interface_area_square_metres: 0.005,
                maximum_penetration_metres: 0.18,
            });
        }
        if dormer.kind == DormerKind::TransverseGable
            && let (Some(parent_id), Some((facade_id, _))) = (child.parent, facade_wall)
            && let Some(parent) = roofs.iter_mut().find(|roof| roof.id == parent_id)
            && let Some(link) = parent
                .children
                .iter_mut()
                .find(|link| link.child == roof_id)
        {
            link.facade_wall = Some(facade_id);
        }
    }
}
