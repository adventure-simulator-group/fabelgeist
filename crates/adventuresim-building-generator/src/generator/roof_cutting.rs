fn split_cross_gable_parent_eave(
    parent: &mut RoofAssembly,
    child_id: RoofAssemblyId,
    origin: Vec2,
    tangent: Vec2,
    width: f32,
) -> Vec<ResolvedItemId> {
    let front_left = origin - tangent * width * 0.5;
    let front_right = origin + tangent * width * 0.5;
    let candidate = parent
        .edges
        .iter()
        .enumerate()
        .find_map(|(edge_index, edge)| {
            if edge.kind != RoofEdgeKind::Eave {
                return None;
            }
            let a = Vec2::new(edge.start.x, edge.start.z);
            let b = Vec2::new(edge.end.x, edge.end.z);
            let delta = b - a;
            if delta.normalize_or_zero().dot(tangent).abs() < 0.99 {
                return None;
            }
            let denominator = delta.length_squared();
            let left_t = (front_left - a).dot(delta) / denominator.max(0.000_001);
            let right_t = (front_right - a).dot(delta) / denominator.max(0.000_001);
            let lo = left_t.min(right_t);
            let hi = left_t.max(right_t);
            (lo > 0.02 && hi < 0.98 && (hi - lo) * delta.length() >= width * 0.85)
                .then_some((edge_index, lo, hi))
        });
    let Some((edge_index, lo, hi)) = candidate else {
        return Vec::new();
    };
    let old = parent.edges.remove(edge_index);
    let left_point = old.start.lerp(old.end, lo);
    let right_point = old.start.lerp(old.end, hi);
    let serial = parent.edges.len() as u64;
    let ids = [0_u64, 1, 2].map(|slot| {
        ResolvedItemId((0xB_u64 << 60) | (parent.id.0 << 16) | 0x0E00 | (serial << 2) | slot)
    });
    parent.edges.extend([
        RoofEdge {
            id: ids[0],
            start: old.start,
            end: left_point,
            kind: RoofEdgeKind::Eave,
            adjacent_faces: old.adjacent_faces.clone(),
            flashing: None,
            drainage_terminal: old.drainage_terminal,
        },
        RoofEdge {
            id: ids[1],
            start: left_point,
            end: right_point,
            kind: RoofEdgeKind::OpeningCut,
            adjacent_faces: old.adjacent_faces.clone(),
            flashing: None,
            drainage_terminal: None,
        },
        RoofEdge {
            id: ids[2],
            start: right_point,
            end: old.end,
            kind: RoofEdgeKind::Eave,
            adjacent_faces: old.adjacent_faces,
            flashing: None,
            drainage_terminal: old.drainage_terminal,
        },
    ]);
    if let Some(link) = parent
        .children
        .iter_mut()
        .find(|link| link.child == child_id)
    {
        link.split_eave_edges = ids.to_vec();
    }
    ids.to_vec()
}

fn clip_plan_polygon_to_child_above_parent(
    mut polygon: Vec<Vec2>,
    parent: RoofPlaneEquation,
    child: RoofPlaneEquation,
) -> Result<Vec<Vec2>, crate::GenerationError> {
    if polygon.is_empty() {
        return Ok(polygon);
    }
    let clearance =
        |point: Vec2| roof_plane_height(child, point) - roof_plane_height(parent, point);
    let input = std::mem::take(&mut polygon);
    for index in 0..input.len() {
        let current = input[index];
        let previous = input[(index + input.len() - 1) % input.len()];
        let current_clearance = clearance(current);
        let previous_clearance = clearance(previous);
        let current_inside = current_clearance >= -0.001;
        let previous_inside = previous_clearance >= -0.001;
        if current_inside != previous_inside {
            let denominator = previous_clearance - current_clearance;
            let fraction = if denominator.abs() <= 0.000_001 {
                0.0
            } else {
                previous_clearance / denominator
            };
            polygon.push(previous.lerp(current, fraction));
        }
        if current_inside {
            polygon.push(current);
        }
    }
    Ok(polygon)
}

fn cut_parent_roof_face(
    assembly: &mut RoofAssembly,
    child: &RoofAssembly,
    cut_bounds: SpatialBounds<Architectural>,
    geometry: &mut ResolvedGeometry,
) -> Result<Vec<ResolvedItemId>, crate::GenerationError> {
    let mut cut_edges = Vec::new();
    let serial_base = assembly.edges.len();
    for face in &mut assembly.faces {
        let projected = face
            .polygon
            .iter()
            .map(|point| Vec2::new(point.x, point.z))
            .collect::<Vec<_>>();
        let mut cut_points = Vec::new();
        for child_face in &child.faces {
            let child_projected = child_face
                .polygon
                .iter()
                .map(|point| Vec2::new(point.x, point.z))
                .collect::<Vec<_>>();
            let bounded = clip_plan_polygon_to_rect(
                projected.clone(),
                Vec2::new(cut_bounds.min().metres().x, cut_bounds.min().metres().z),
                Vec2::new(cut_bounds.max().metres().x, cut_bounds.max().metres().z),
            );
            let mut clipped = clip_plan_polygon_to_convex(bounded, &child_projected);
            clipped =
                clip_plan_polygon_to_child_above_parent(clipped, face.plane, child_face.plane)?;
            let unique = unique_roof_cut_points(clipped);
            cut_points.extend(unique);
        }
        let unique = convex_plan_hull(cut_points);
        let signed_area = signed_plan_area(&unique);
        if unique.len() >= 3 && signed_area.abs() > 0.002 {
            let mut cutout = unique
                .into_iter()
                .map(|point| Vec3::new(point.x, roof_plane_height(face.plane, point), point.y))
                .collect::<Vec<_>>();
            if signed_area > 0.0 {
                cutout.reverse();
            }
            let face_id = face.id;
            face.cutouts.push(cutout.clone());
            for index in 0..cutout.len() {
                let serial = serial_base + cut_edges.len();
                let edge_id = ResolvedItemId(
                    (0xB_u64 << 60) | (assembly.id.0 << 16) | (0x800 + serial) as u64,
                );
                let start = cutout[index];
                let end = cutout[(index + 1) % cutout.len()];
                let flashing_id = ResolvedItemId(
                    (0x8_u64 << 60) | (assembly.id.0 << 16) | (0x800 + serial) as u64,
                );
                let delta = end - start;
                geometry.solids.push(ResolvedSolid::new(
                    CollisionCuboid::<Architectural>::from_metres(
                        flashing_id,
                        (start + end) * 0.5 + Vec3::Y * 0.012,
                        Vec3::new((delta.length() - 0.12).max(0.05), 0.024, 0.10),
                        delta.z.atan2(delta.x),
                        0.08,
                        0.0,
                    )?,
                    assembly.owner,
                    SolidRole::RoofFlashing,
                    crate::ResolvedSolidShape::Cuboid,
                    vec![assembly.support_nodes[0]],
                ));
                geometry
                    .support_interfaces
                    .push(crate::SupportInterface::new(
                        ResolvedItemId(
                            (0x9_u64 << 60) | (assembly.id.0 << 16) | (0x800 + serial) as u64,
                        ),
                        assembly.owner,
                        assembly.support_nodes[0],
                        SpatialBounds::<Architectural>::from_metres(
                            (start + end) * 0.5 - Vec3::new(0.12, 0.04, 0.12),
                            (start + end) * 0.5 + Vec3::new(0.12, 0.08, 0.12),
                        )?,
                    ));
                cut_edges.push(edge_id);
                assembly.edges.push(RoofEdge {
                    id: edge_id,
                    start,
                    end,
                    kind: RoofEdgeKind::OpeningCut,
                    adjacent_faces: vec![face_id],
                    flashing: Some(flashing_id),
                    drainage_terminal: None,
                });
            }
        }
    }
    Ok(cut_edges)
}

fn bind_child_valleys(
    parent: &mut RoofAssembly,
    child: &RoofAssembly,
    cut_edges: &[ResolvedItemId],
    geometry: &mut ResolvedGeometry,
) -> Result<Vec<ResolvedItemId>, crate::GenerationError> {
    let mut valleys = Vec::new();
    let mut candidates = cut_edges
        .iter()
        .filter_map(|id| {
            parent
                .edges
                .iter()
                .find(|edge| edge.id == *id)
                .map(|edge| (*id, (edge.start.y - edge.end.y).abs()))
        })
        .filter(|(_, fall)| *fall > 0.02)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.1.total_cmp(&left.1));
    for (edge_id, _) in candidates.into_iter().take(2) {
        if let Some(edge) = parent.edges.iter_mut().find(|edge| edge.id == edge_id) {
            edge.kind = RoofEdgeKind::Valley;
            let (high, low) = if edge.start.y >= edge.end.y {
                (edge.start, edge.end)
            } else {
                (edge.end, edge.start)
            };
            let suffix = edge.id.0 & ((1_u64 << 60) - 1);
            let outlet_id = ResolvedItemId((0xE_u64 << 60) | suffix);
            let route_id = ResolvedItemId((0xD_u64 << 60) | suffix);
            geometry.voids.push(ResolvedVoid {
                id: outlet_id,
                owner: parent.owner,
                bounds: SpatialBounds::<Architectural>::from_metres(
                    // The terminal is an upward-open throat beginning at the
                    // weather surface. Extending it below the valley would
                    // falsely cut the receiving eave gutter or wall plate.
                    low - Vec3::new(0.04, 0.0, 0.04),
                    low + Vec3::new(0.04, 0.08, 0.04),
                )?,
                role: VoidRole::Drain,
                shape: crate::ResolvedVoidShape::Box,
                subtracts_from: parent.owner,
            });
            geometry.drainage_routes.push(DrainageRoute {
                id: route_id,
                owner: parent.owner,
                outlet_void: outlet_id,
                inlet: high,
                outlet: low,
            });
            edge.drainage_terminal = Some(outlet_id);
            if let Some(flashing) = edge
                .flashing
                .and_then(|id| geometry.solids.iter_mut().find(|solid| solid.id == id))
            {
                let delta = edge.end - edge.start;
                let run = Vec2::new(delta.x, delta.z).length().max(0.01);
                let uphill = (high - low).normalize_or_zero();
                flashing.centre = Position::<Architectural>::from_metres(
                    (high + low) * 0.5 + uphill * 0.06 + Vec3::Y * (flashing.size.metres().y * 0.5),
                )?;
                {
                    let mut native_geometry = flashing.size.metres();
                    native_geometry.x = (delta.length() - 0.12).max(0.05);
                    flashing.size =
                        CuboidDimensions::from_metres(native_geometry).map_err(|cause| {
                            crate::CollisionError {
                                source_id: flashing.id,
                                cause,
                            }
                        })?;
                };
                flashing.longfall_radians = Radians::new(delta.y.atan2(run))?;
            }
            if let Some(face) = child.faces.iter().min_by(|left, right| {
                let midpoint = (edge.start + edge.end) * 0.5;
                let left_distance = left
                    .polygon
                    .iter()
                    .map(|point| point.distance_squared(midpoint))
                    .fold(f32::INFINITY, f32::min);
                let right_distance = right
                    .polygon
                    .iter()
                    .map(|point| point.distance_squared(midpoint))
                    .fold(f32::INFINITY, f32::min);
                left_distance.total_cmp(&right_distance)
            }) {
                edge.adjacent_faces.push(face.id);
            }
            valleys.push(edge_id);
        }
    }
    Ok(valleys)
}

fn trim_roof_edge_treatments_for_cut(
    owner: GeometryOwnerId,
    cut: SpatialBounds<Architectural>,
    geometry: &mut ResolvedGeometry,
) -> Result<(), crate::GenerationError> {
    let cut_centre = Vec2::new(
        (cut.min().metres().x + cut.max().metres().x) * 0.5,
        (cut.min().metres().z + cut.max().metres().z) * 0.5,
    );
    let cut_half = Vec2::new(
        (cut.max().metres().x - cut.min().metres().x) * 0.5,
        (cut.max().metres().z - cut.min().metres().z) * 0.5,
    );
    for solid in geometry.solids.iter_mut().filter(|solid| {
        solid.owner == owner
            && matches!(
                solid.role,
                SolidRole::RoofEdgeTreatment | SolidRole::RoofGutter
            )
    }) {
        let tangent = Vec2::new(
            solid.yaw_radians.radians().cos(),
            solid.yaw_radians.radians().sin(),
        );
        let plan_scale = solid.longfall_radians.radians().cos().abs().max(0.01);
        let normal = Vec2::new(-tangent.y, tangent.x);
        let centre = Vec2::new(solid.centre.metres().x, solid.centre.metres().z);
        let offset = cut_centre - centre;
        let lateral_extent = cut_half.x * normal.x.abs() + cut_half.y * normal.y.abs();
        if offset.dot(normal).abs() > lateral_extent + solid.size.metres().z * 0.5 + 0.01 {
            continue;
        }
        let cut_along = (cut_half.x * tangent.x.abs() + cut_half.y * tangent.y.abs()) / plan_scale;
        let cut_centre_along = offset.dot(tangent) / plan_scale;
        let cut_min = cut_centre_along - cut_along;
        let cut_max = cut_centre_along + cut_along;
        let old_min = -solid.size.metres().x * 0.5;
        let old_max = solid.size.metres().x * 0.5;
        if cut_max <= old_min + 0.01 || cut_min >= old_max - 0.01 {
            continue;
        }
        let kept = if cut_min <= old_min + 0.01 && cut_max < old_max - 0.05 {
            Some((cut_max + 0.08, old_max))
        } else if cut_max >= old_max - 0.01 && cut_min > old_min + 0.05 {
            Some((old_min, cut_min - 0.08))
        } else {
            // No curated tower presently bisects a ridge. Rejecting this
            // topology later is safer than leaving a treatment through the
            // opening; a full two-segment edge graph is required for it.
            None
        };
        if let Some((from, to)) = kept {
            let shift = (from + to) * 0.5;
            let mut centre = solid.centre.metres();
            centre.x += tangent.x * plan_scale * shift;
            centre.y += solid.longfall_radians.radians().sin() * shift;
            centre.z += tangent.y * plan_scale * shift;
            solid.centre = Position::<Architectural>::from_metres(centre).map_err(|cause| {
                crate::CollisionError {
                    source_id: solid.id,
                    cause,
                }
            })?;
            {
                let mut native_geometry = solid.size.metres();
                native_geometry.x = to - from;
                solid.size = CuboidDimensions::from_metres(native_geometry).map_err(|cause| {
                    crate::CollisionError {
                        source_id: solid.id,
                        cause,
                    }
                })?;
            };
            let interface_id = ResolvedItemId((0x9_u64 << 60) | (solid.id.0 & ((1_u64 << 60) - 1)));
            if let Some(interface) = geometry
                .support_interfaces
                .iter_mut()
                .find(|interface| interface.id == interface_id)
            {
                interface.bounds = SpatialBounds::from_metres(
                    solid.centre.metres() - Vec3::new(0.08, 0.025, 0.08),
                    solid.centre.metres() + Vec3::new(0.08, 0.025, 0.08),
                )?;
            }
        } else {
            {
                let mut native_geometry = solid.size.metres();
                native_geometry.x = 0.0;
                solid.size = CuboidDimensions::from_metres(native_geometry).map_err(|cause| {
                    crate::CollisionError {
                        source_id: solid.id,
                        cause,
                    }
                })?;
            };
        }
    }
    geometry
        .solids
        .retain(|solid| solid.size.metres().x > 0.001);

    Ok(())
}

fn trim_roof_boundary_edges_for_cut(
    assembly: &mut RoofAssembly,
    cut: SpatialBounds<Architectural>,
) {
    let inside = |point: Vec3| {
        point.x >= cut.min().metres().x - 0.002
            && point.x <= cut.max().metres().x + 0.002
            && point.z >= cut.min().metres().z - 0.002
            && point.z <= cut.max().metres().z + 0.002
    };
    for edge in assembly.edges.iter_mut().filter(|edge| {
        matches!(edge.kind, RoofEdgeKind::Eave | RoofEdgeKind::GableVerge)
            && edge.adjacent_faces.len() == 1
    }) {
        let start_inside = inside(edge.start);
        let end_inside = inside(edge.end);
        if start_inside == end_inside {
            continue;
        }
        let from = edge.start;
        let delta = edge.end - edge.start;
        let mut intersections = Vec::new();
        for (axis_start, axis_delta, low, high) in [
            (from.x, delta.x, cut.min().metres().x, cut.max().metres().x),
            (from.z, delta.z, cut.min().metres().z, cut.max().metres().z),
        ] {
            if axis_delta.abs() <= 0.000_001 {
                continue;
            }
            for boundary in [low, high] {
                let t = (boundary - axis_start) / axis_delta;
                if (0.0..=1.0).contains(&t) {
                    let point = from + delta * t;
                    if point.x >= cut.min().metres().x - 0.003
                        && point.x <= cut.max().metres().x + 0.003
                        && point.z >= cut.min().metres().z - 0.003
                        && point.z <= cut.max().metres().z + 0.003
                    {
                        intersections.push((t, point));
                    }
                }
            }
        }
        if start_inside {
            if let Some((_, point)) = intersections
                .into_iter()
                .max_by(|left, right| left.0.total_cmp(&right.0))
            {
                edge.start = point + delta.normalize_or_zero() * 0.10;
            }
        } else if let Some((_, point)) = intersections
            .into_iter()
            .min_by(|left, right| left.0.total_cmp(&right.0))
        {
            edge.end = point - delta.normalize_or_zero() * 0.10;
        }
    }
}

fn unique_roof_cut_points(clipped: Vec<Vec2>) -> Vec<Vec2> {
    let mut unique = Vec::new();
    for point in clipped {
        if !unique
            .iter()
            .any(|existing: &Vec2| existing.distance_squared(point) <= 0.000_004)
        {
            unique.push(point);
        }
    }
    unique
}
