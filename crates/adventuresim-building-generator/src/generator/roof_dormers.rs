//! Resolve attached dormer roofs, their enclosing walls and parent cuts.
use super::*;
use dormer_layout::DormerLayout;

pub(super) fn append(
    assemblies: &mut Vec<RoofAssembly>,
    dormers: &[RoofDormer],
    walls: &[crate::WallAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), GenerationError> {
    let parent = assemblies.first().map(|roof| roof.id);
    for (index, dormer) in dormers.iter().copied().enumerate() {
        let DormerLayout {
            inward,
            tangent,
            half_width,
            top,
            enclosure_depth,
            ridge_seam_depth,
            recipe,
        } = DormerLayout::new(&assemblies[0], dormer)?;
        let id = RoofAssemblyId(1_000 + index as u64);
        let mut child = resolve_one_roof(
            id,
            GeometryOwnerId(61_000 + index as u32),
            recipe,
            None,
            None,
            parent,
            RoofPhase::AttachedChild,
            (recipe.kind == RoofKind::Shed).then_some(dormer.facing.opposite()),
            assemblies.first(),
            walls,
            geometry,
        );
        extend_ridge(&mut child, inward, ridge_seam_depth, enclosure_depth);
        remove_rear_gable(
            &mut child,
            dormer,
            recipe,
            inward,
            enclosure_depth,
            id,
            geometry,
        );
        append_enclosures(
            &mut child,
            assemblies,
            dormer,
            tangent,
            half_width,
            inward,
            enclosure_depth,
            top,
            id,
            walls,
        );
        attach_to_parent(
            &child,
            assemblies,
            dormer,
            recipe,
            id,
            tangent,
            if dormer.kind == DormerKind::TransverseGable {
                2.20
            } else {
                1.0
            },
            geometry,
        );
        child.phase = RoofPhase::AttachedChild;
        assemblies.push(child);
    }
    Ok(())
}

fn extend_ridge(
    child: &mut RoofAssembly,
    inward: Vec2,
    ridge_seam_depth: f32,
    enclosure_depth: f32,
) {
    if ridge_seam_depth > enclosure_depth + 0.01 {
        let extension = inward * (ridge_seam_depth - enclosure_depth);
        let mut moved_ridge_points = Vec::new();
        for face in &mut child.faces {
            let ridge_height = face
                .polygon
                .iter()
                .map(|point| point.y)
                .fold(f32::NEG_INFINITY, f32::max);
            let Some(rear_ridge_index) = face
                .polygon
                .iter()
                .enumerate()
                .filter(|(_, point)| (point.y - ridge_height).abs() <= 0.01)
                .max_by(|(_, left), (_, right)| {
                    Vec2::new(left.x, left.z)
                        .dot(inward)
                        .total_cmp(&Vec2::new(right.x, right.z).dot(inward))
                })
                .map(|(index, _)| index)
            else {
                continue;
            };
            let old = face.polygon[rear_ridge_index];
            let new = old + Vec3::new(extension.x, 0.0, extension.y);
            face.polygon[rear_ridge_index] = new;
            moved_ridge_points.push((old, new));
        }
        for edge in &mut child.edges {
            for (old, new) in &moved_ridge_points {
                if edge.start.distance_squared(*old) <= 0.000_004 {
                    edge.start = *new;
                }
                if edge.end.distance_squared(*old) <= 0.000_004 {
                    edge.end = *new;
                }
            }
        }
    }
}

fn remove_rear_gable(
    child: &mut RoofAssembly,
    dormer: RoofDormer,
    recipe: RoofPiece,
    inward: Vec2,
    enclosure_depth: f32,
    id: RoofAssemblyId,
    geometry: &mut ResolvedGeometry,
) {
    if recipe.kind == RoofKind::Gable {
        // `resolve_one_roof` normally closes both gable ends. A dormer
        // owns only the visible front gable; its rear terminates in the
        // parent weather plane. Remove the otherwise floating rear
        // triangle and its two verge caps. The parent's cut-edge flashing
        // owns the seated head joint.
        child.enclosure_faces.retain(|face| {
            let mean_depth = face
                .polygon
                .iter()
                .map(|point| (Vec2::new(point.x, point.z) - dormer.centre).dot(-inward))
                .sum::<f32>()
                / face.polygon.len() as f32;
            mean_depth > -enclosure_depth + 0.02
        });
        let rear_edge_depth = -(enclosure_depth + recipe.eave_metres);
        for (edge_index, edge) in child.edges.iter_mut().enumerate() {
            let start_depth = (Vec2::new(edge.start.x, edge.start.z) - dormer.centre).dot(-inward);
            let end_depth = (Vec2::new(edge.end.x, edge.end.z) - dormer.centre).dot(-inward);
            if edge.kind == RoofEdgeKind::GableVerge
                && start_depth <= rear_edge_depth + 0.02
                && end_depth <= rear_edge_depth + 0.02
            {
                edge.kind = RoofEdgeKind::OpeningCut;
                let weather_id =
                    ResolvedItemId((0x8_u64 << 60) | (id.0 << 16) | 0x5000 | edge_index as u64);
                let interface_id =
                    ResolvedItemId((0x9_u64 << 60) | (id.0 << 16) | 0x5000 | edge_index as u64);
                geometry.solids.retain(|solid| solid.id != weather_id);
                geometry
                    .support_interfaces
                    .retain(|interface| interface.id != interface_id);
            }
        }
    }
}

fn append_enclosures(
    child: &mut RoofAssembly,
    assemblies: &[RoofAssembly],
    dormer: RoofDormer,
    tangent: Vec2,
    half_width: f32,
    inward: Vec2,
    enclosure_depth: f32,
    top: f32,
    id: RoofAssemblyId,
    walls: &[crate::WallAssembly],
) {
    let front_left = dormer.centre - tangent * half_width;
    let front_right = dormer.centre + tangent * half_width;
    let rear_left = front_left + inward * enclosure_depth;
    let rear_right = front_right + inward * enclosure_depth;
    let ceiling = |point: Vec2| {
        if dormer.kind == DormerKind::Shed {
            shed_dormers::underside_height(child, point)
        } else {
            top
        }
    };
    let parent_height = |point: Vec2| {
        assemblies
            .first()
            .and_then(|parent| roof_surface_height_at(parent, point))
            .unwrap_or(dormer.base_height_metres)
    };
    for (slot, polygon) in [
        vec![
            Vec3::new(front_left.x, parent_height(front_left), front_left.y),
            Vec3::new(front_right.x, parent_height(front_right), front_right.y),
            Vec3::new(front_right.x, ceiling(front_right), front_right.y),
            Vec3::new(front_left.x, ceiling(front_left), front_left.y),
        ],
        vec![
            Vec3::new(front_left.x, parent_height(front_left), front_left.y),
            Vec3::new(front_left.x, ceiling(front_left), front_left.y),
            Vec3::new(rear_left.x, ceiling(rear_left), rear_left.y),
            Vec3::new(rear_left.x, parent_height(rear_left), rear_left.y),
        ],
        vec![
            Vec3::new(front_right.x, parent_height(front_right), front_right.y),
            Vec3::new(rear_right.x, parent_height(rear_right), rear_right.y),
            Vec3::new(rear_right.x, ceiling(rear_right), rear_right.y),
            Vec3::new(front_right.x, ceiling(front_right), front_right.y),
        ],
    ]
    .into_iter()
    .enumerate()
    {
        child.enclosure_faces.push(RoofEnclosureFace::new(
            ResolvedItemId((0xA_u64 << 60) | (id.0 << 16) | 0x4100 | slot as u64),
            polygon,
            if walls
                .iter()
                .any(|wall| wall.material == crate::WallMaterialClass::TimberInfill)
            {
                RoofMaterial::TimberInfill
            } else {
                RoofMaterial::MasonryInfill
            },
            child.support_nodes.clone(),
        ));
    }
}

fn attach_to_parent(
    child: &RoofAssembly,
    assemblies: &mut [RoofAssembly],
    dormer: RoofDormer,
    recipe: RoofPiece,
    id: RoofAssemblyId,
    tangent: Vec2,
    scale: f32,
    geometry: &mut ResolvedGeometry,
) {
    if child.parent.is_some() {
        let cut_id = ResolvedItemId((0xF_u64 << 60) | id.0);
        let bounds = ResolvedBounds {
            min: Vec3::new(
                recipe.centre.x - recipe.size.x * 0.5,
                recipe.base_height_metres - 0.2,
                recipe.centre.y - recipe.size.y * 0.5,
            ),
            max: Vec3::new(
                recipe.centre.x + recipe.size.x * 0.5,
                recipe.base_height_metres + 4.0,
                recipe.centre.y + recipe.size.y * 0.5,
            ),
        };
        geometry.voids.push(ResolvedVoid {
            id: cut_id,
            owner: assemblies[0].owner,
            bounds,
            role: VoidRole::RoofOpening,
            shape: crate::ResolvedVoidShape::Box,
            subtracts_from: assemblies[0].owner,
        });
        let child_kind = match dormer.kind {
            DormerKind::Gabled | DormerKind::Hipped => RoofChildKind::GabledDormer,
            DormerKind::Shed => RoofChildKind::ShedDormer,
            DormerKind::TransverseGable => RoofChildKind::CrossGable,
        };
        let cut_edges = cut_parent_roof_face(&mut assemblies[0], child, bounds, geometry);
        let valleys = bind_child_valleys(&mut assemblies[0], child, &cut_edges, geometry);
        let flashing_ids = assemblies[0]
            .edges
            .iter()
            .filter(|edge| cut_edges.contains(&edge.id))
            .filter_map(|edge| edge.flashing)
            .collect();
        assemblies[0].children.push(RoofChildAssembly {
            child: id,
            kind: child_kind,
            parent_cut: cut_id,
            trimmer_nodes: child.support_nodes.clone(),
            valley_edges: valleys,
            flashing_ids,
            facade_wall: None,
            split_eave_edges: Vec::new(),
        });
        if dormer.kind == DormerKind::TransverseGable {
            split_cross_gable_parent_eave(
                &mut assemblies[0],
                id,
                dormer.centre,
                tangent,
                dormer.width_metres * scale,
            );
        }
    }
}
