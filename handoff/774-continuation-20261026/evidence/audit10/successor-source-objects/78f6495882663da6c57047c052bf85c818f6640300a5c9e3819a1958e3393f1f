fn finalize_roof_drainage(
    archetype: BuildingArchetype,
    assemblies: &mut [RoofAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), crate::GenerationError> {
    let owners = assemblies
        .iter()
        .map(|roof| roof.owner)
        .collect::<HashSet<_>>();
    geometry
        .roof_drainage_networks
        .retain(|network| !owners.contains(&network.owner));
    geometry
        .solids
        .retain(|solid| !(owners.contains(&solid.owner) && solid.role == SolidRole::RoofGutter));

    let _: () = for assembly in assemblies {
        for (face_index, face) in assembly.faces.iter().enumerate() {
            let Some(edge) = assembly
                .edges
                .iter()
                .filter(|edge| {
                    edge.adjacent_faces.contains(&face.id)
                        && matches!(edge.kind, RoofEdgeKind::Eave | RoofEdgeKind::Valley)
                })
                .min_by(|left, right| {
                    ((left.start.y + left.end.y) * 0.5)
                        .total_cmp(&((right.start.y + right.end.y) * 0.5))
                })
            else {
                continue;
            };
            let edge_a = Vec2::new(edge.start.x, edge.start.z);
            let edge_b = Vec2::new(edge.end.x, edge.end.z);
            let edge_delta = edge_b - edge_a;
            let edge_length = edge_delta.length().max(0.05);
            let tangent = edge_delta / edge_length;
            let centre = face.polygon.iter().copied().sum::<Vec3>() / face.polygon.len() as f32;
            let downhill = Vec2::new(
                face.plane.normal.x / face.plane.normal.y,
                face.plane.normal.z / face.plane.normal.y,
            )
            .normalize_or_zero();
            let sampling = roof_plan_sampling::RoofPlanSampling::new(face);
            let sample_origins = sampling.origins().collect::<Vec<_>>();
            let samples = sample_origins
                .into_iter()
                .filter_map(|origin| {
                    let hit = ray_segment_intersection(origin, downhill, edge_a, edge_b)?;
                    let surface_y = roof_plane_height(face.plane, origin);
                    let edge_y = roof_plane_height(face.plane, hit);
                    (surface_y > edge_y + 0.005).then_some(RoofDrainageSample {
                        surface_point: Vec3::new(origin.x, surface_y, origin.y),
                        channel_inlet: Vec3::new(hit.x, edge_y - 0.025, hit.y),
                    })
                })
                .collect::<Vec<_>>();

            let serial = face_index as u64 * 8;
            let base = (0x8_u64 << 60) | (assembly.id.0 << 16) | 0x6000 | serial;
            let floor_id = ResolvedItemId(base);
            let lip_ids = [ResolvedItemId(base | 1), ResolvedItemId(base | 2)];
            let downspout_id = ResolvedItemId(base | 3);
            let network_id = ResolvedItemId(
                (0x7_u64 << 60) | (assembly.id.0 << 16) | 0x6000 | face_index as u64,
            );
            let compact_child_eave = assembly.parent.is_some()
                && matches!(assembly.kind, RoofKind::Gable | RoofKind::Shed);
            let gutter_width = if compact_child_eave { 0.085 } else { 0.18 };
            let gutter_floor_thickness = if compact_child_eave { 0.018 } else { 0.035 };
            let gutter_lip_height = if compact_child_eave { 0.040 } else { 0.11 };
            let gutter_lip_thickness = if compact_child_eave { 0.018 } else { 0.035 };
            let drop = if compact_child_eave {
                (edge_length * 0.006).max(0.018)
            } else {
                (edge_length * 0.012).max(0.045)
            };
            let lexical_forward = (edge_b.x, edge_b.y) >= (edge_a.x, edge_a.y);
            let (high_plan, low_plan, channel_tangent) = if lexical_forward {
                (edge_a, edge_b, tangent)
            } else {
                (edge_b, edge_a, -tangent)
            };
            let edge_mean_y = (edge.start.y + edge.end.y) * 0.5;
            let mut high = Vec3::new(
                high_plan.x,
                edge_mean_y - gutter_floor_thickness,
                high_plan.y,
            );
            let mut low = Vec3::new(
                low_plan.x,
                edge_mean_y - gutter_floor_thickness - drop,
                low_plan.y,
            );
            let outward = (Vec2::new((high.x + low.x) * 0.5, (high.z + low.z) * 0.5)
                - Vec2::new(centre.x, centre.z))
            .normalize_or_zero();
            if assembly.parent.is_none()
                && matches!(
                    archetype,
                    BuildingArchetype::CastleGatehouse | BuildingArchetype::CourtyardCastle
                )
            {
                // Defensive roof edges sit over deep masonry returns. Keep
                // the physical gutter outside that face instead of centring
                // it inside the wall and escaping with a diagonal collector.
                let fascia_offset = Vec3::new(outward.x, 0.0, outward.y) * 0.35;
                high += fascia_offset;
                low += fascia_offset;
            }
            let channel_centre = (high + low) * 0.5;
            let yaw = channel_tangent.y.atan2(channel_tangent.x);
            let longfall = -drop.atan2(edge_length);
            geometry.solids.push(ResolvedSolid::new(
                CollisionCuboid::<Architectural>::from_metres(
                    floor_id,
                    channel_centre,
                    Vec3::new(edge_length, gutter_floor_thickness, gutter_width),
                    yaw,
                    0.0,
                    longfall,
                )?,
                assembly.owner,
                SolidRole::RoofGutter,
                crate::ResolvedSolidShape::Cuboid,
                face.support_nodes.clone(),
            ));
            geometry
                .support_interfaces
                .push(crate::SupportInterface::new(
                    ResolvedItemId((0x9_u64 << 60) | (assembly.id.0 << 16) | 0x6000 | serial),
                    assembly.owner,
                    face.support_nodes[0],
                    SpatialBounds::<Architectural>::from_metres(
                        channel_centre - Vec3::splat(0.035),
                        channel_centre + Vec3::splat(0.035),
                    )?,
                ));
            let lip_offset = Vec3::new(outward.x, 0.0, outward.y) * (gutter_width * 0.5 - 0.008);
            for (lip_slot, (lip_id, sign)) in lip_ids.into_iter().zip([-1.0_f32, 1.0]).enumerate() {
                let lip_centre = channel_centre
                    + lip_offset * sign
                    + Vec3::Y * (gutter_lip_height * 0.5 - gutter_floor_thickness * 0.25);
                geometry.solids.push(ResolvedSolid::new(
                    CollisionCuboid::<Architectural>::from_metres(
                        lip_id,
                        lip_centre,
                        Vec3::new(edge_length, gutter_lip_height, gutter_lip_thickness),
                        yaw,
                        0.0,
                        longfall,
                    )?,
                    assembly.owner,
                    SolidRole::RoofGutter,
                    crate::ResolvedSolidShape::Cuboid,
                    face.support_nodes.clone(),
                ));
                geometry
                    .support_interfaces
                    .push(crate::SupportInterface::new(
                        ResolvedItemId(
                            (0x9_u64 << 60)
                                | (assembly.id.0 << 16)
                                | 0x6000
                                | serial
                                | (lip_slot as u64 + 1),
                        ),
                        assembly.owner,
                        face.support_nodes[0],
                        SpatialBounds::<Architectural>::from_metres(
                            lip_centre - Vec3::splat(0.035),
                            lip_centre + Vec3::splat(0.035),
                        )?,
                    ));
            }
            let outlet_plan = low_plan
                + channel_tangent * if compact_child_eave { 0.03 } else { 0.08 }
                + outward * if compact_child_eave { 0.09 } else { 0.38 };
            let outlet = Vec3::new(outlet_plan.x, low.y - 0.025, outlet_plan.y);
            let discharge = Vec3::new(outlet.x, 0.24, outlet.z);
            let outlet_id = geometry
                .drainage_catchments
                .iter()
                .find(|catchment| catchment.id == face.drainage_catchment)
                .and_then(|catchment| {
                    geometry
                        .drainage_routes
                        .iter()
                        .find(|route| route.id == catchment.outlet_route)
                        .map(|route| route.outlet_void)
                })
                .ok_or(GenerationError::MissingRoofOutlet {
                    catchment: face.drainage_catchment,
                })?;
            if let Some(void) = geometry.voids.iter_mut().find(|void| void.id == outlet_id) {
                void.bounds = SpatialBounds::<Architectural>::from_metres(
                    outlet - Vec3::splat(0.04),
                    outlet + Vec3::splat(0.04),
                )?;
            }
            if let Some(catchment) = geometry
                .drainage_catchments
                .iter_mut()
                .find(|catchment| catchment.id == face.drainage_catchment)
            {
                catchment.toe_channel_solids = vec![floor_id, lip_ids[0], lip_ids[1]];
                catchment.tangent = channel_tangent;
                catchment.outward = outward;
                catchment.outlet_along_metres = edge_length * 0.5;
            }
            if let Some(route) = geometry
                .drainage_routes
                .iter_mut()
                .find(|route| route.outlet_void == outlet_id)
            {
                route.inlet = samples.first().map_or(high, |sample| sample.surface_point);
                route.outlet = outlet;
            }
            geometry.solids.push(ResolvedSolid::new(
                CollisionCuboid::<Architectural>::from_metres(
                    downspout_id,
                    (outlet + discharge) * 0.5 - Vec3::Y * 0.10,
                    Vec3::new(0.09, (outlet.y - discharge.y - 0.20).max(0.09), 0.09),
                    0.0,
                    0.0,
                    0.0,
                )?,
                assembly.owner,
                SolidRole::RoofGutter,
                crate::ResolvedSolidShape::Cuboid,
                face.support_nodes.clone(),
            ));
            let spout_top = Vec3::new(outlet.x, outlet.y - 0.20, outlet.z);
            geometry
                .support_interfaces
                .push(crate::SupportInterface::new(
                    ResolvedItemId((0x9_u64 << 60) | (assembly.id.0 << 16) | 0x6000 | serial | 3),
                    assembly.owner,
                    face.support_nodes[0],
                    SpatialBounds::<Architectural>::from_metres(
                        spout_top - Vec3::splat(0.035),
                        spout_top + Vec3::splat(0.035),
                    )?,
                ));
            geometry.roof_drainage_networks.push(RoofDrainageNetwork {
                id: network_id,
                owner: assembly.owner,
                face: face.id,
                catchment: face.drainage_catchment,
                receiving_edge: edge.id,
                samples,
                channel_floor: floor_id,
                channel_lips: lip_ids,
                collector_solids: Vec::new(),
                outlet_station: network_id,
                outlet_void: outlet_id,
                downspout: Some(downspout_id),
                channel_high: high,
                channel_low: low,
                discharge,
            });
        }
    };
    Ok(())
}
