fn supplement_split_eave_drainage(
    assemblies: &[RoofAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), crate::GenerationError> {
    let _: () = for assembly in assemblies {
        for link in assembly.children.iter().filter(|link| {
            link.kind == RoofChildKind::CrossGable && link.split_eave_edges.len() == 3
        }) {
            // The two retained eaves and the recessed apron at the facade cut are
            // distinct physical recipients.  The apron is not an eave relabel: it
            // catches the narrow strip of parent weather face that terminates at
            // the Zwerchhaus opening instead of allowing it to discharge onto the
            // facade or through the opening cut.
            let receivers = [
                link.split_eave_edges[0],
                link.split_eave_edges[1],
                link.split_eave_edges[2],
            ];
            let existing_edges = geometry
                .roof_drainage_networks
                .iter()
                .filter(|network| network.owner == assembly.owner)
                .map(|network| network.receiving_edge)
                .collect::<HashSet<_>>();
            for (slot, edge_id) in receivers.into_iter().enumerate() {
                if existing_edges.contains(&edge_id) {
                    continue;
                }
                let Some(edge) = assembly.edges.iter().find(|edge| edge.id == edge_id) else {
                    continue;
                };
                let Some(face) = assembly
                    .faces
                    .iter()
                    .find(|face| edge.adjacent_faces.contains(&face.id))
                else {
                    continue;
                };
                let a = Vec2::new(edge.start.x, edge.start.z);
                let b = Vec2::new(edge.end.x, edge.end.z);
                let delta = b - a;
                let length = delta.length().max(0.05);
                let tangent = delta / length;
                let centre = face.polygon.iter().copied().sum::<Vec3>() / face.polygon.len() as f32;
                let downhill = Vec2::new(
                    face.plane.normal.x / face.plane.normal.y,
                    face.plane.normal.z / face.plane.normal.y,
                )
                .normalize_or_zero();
                let sampling = roof_plan_sampling::RoofPlanSampling::new(face);
                let mut samples = Vec::new();
                for origin in sampling.origins() {
                    let Some(hit) = ray_segment_intersection(origin, downhill, a, b) else {
                        continue;
                    };
                    let surface_y = roof_plane_height(face.plane, origin);
                    let edge_y = roof_plane_height(face.plane, hit);
                    if surface_y > edge_y + 0.005 {
                        samples.push(RoofDrainageSample {
                            surface_point: Vec3::new(origin.x, surface_y, origin.y),
                            channel_inlet: Vec3::new(hit.x, edge_y - 0.025, hit.y),
                        });
                    }
                }
                if samples.is_empty() {
                    continue;
                }
                let serial = 0x6800 | ((link.child.0 & 0xFF) << 4) | (slot as u64 * 4);
                let floor = ResolvedItemId((0x8_u64 << 60) | (assembly.id.0 << 16) | serial);
                let lips = [ResolvedItemId(floor.0 | 1), ResolvedItemId(floor.0 | 2)];
                let spout = ResolvedItemId(floor.0 | 3);
                let catchment = ResolvedItemId((0xC_u64 << 60) | (assembly.id.0 << 16) | serial);
                let route = ResolvedItemId((0xD_u64 << 60) | (assembly.id.0 << 16) | serial);
                let outlet_void = ResolvedItemId((0xE_u64 << 60) | (assembly.id.0 << 16) | serial);
                let network = ResolvedItemId((0x7_u64 << 60) | (assembly.id.0 << 16) | serial);
                let forward = (b.x, b.y) >= (a.x, a.y);
                let (high_plan, low_plan, channel_tangent) = if forward {
                    (a, b, tangent)
                } else {
                    (b, a, -tangent)
                };
                let drop = (length * 0.012).max(0.045);
                let mean_y = (edge.start.y + edge.end.y) * 0.5;
                let high = Vec3::new(high_plan.x, mean_y - 0.035, high_plan.y);
                let low = Vec3::new(low_plan.x, mean_y - 0.035 - drop, low_plan.y);
                let outward = (Vec2::new((high.x + low.x) * 0.5, (high.z + low.z) * 0.5)
                    - Vec2::new(centre.x, centre.z))
                .normalize_or_zero();
                let channel_centre = (high + low) * 0.5;
                let yaw = channel_tangent.y.atan2(channel_tangent.x);
                let longfall = -drop.atan2(length);
                let lip_offset = Vec3::new(outward.x, 0.0, outward.y) * 0.075;
                for (id, item_centre, size) in [
                    (floor, channel_centre, Vec3::new(length, 0.035, 0.18)),
                    (
                        lips[0],
                        channel_centre - lip_offset + Vec3::Y * 0.045,
                        Vec3::new(length, 0.11, 0.035),
                    ),
                    (
                        lips[1],
                        channel_centre + lip_offset + Vec3::Y * 0.045,
                        Vec3::new(length, 0.11, 0.035),
                    ),
                ] {
                    geometry.solids.push(ResolvedSolid::new(
                        CollisionCuboid::<Architectural>::from_metres(
                            id,
                            item_centre,
                            size,
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
                            ResolvedItemId((0x9_u64 << 60) | (id.0 & 0x0FFF_FFFF_FFFF_FFFF)),
                            assembly.owner,
                            face.support_nodes[0],
                            SpatialBounds::<Architectural>::from_metres(
                                item_centre - Vec3::splat(0.035),
                                item_centre + Vec3::splat(0.035),
                            )?,
                        ));
                }
                let outlet_plan = low_plan + channel_tangent * 0.08 + outward * 0.38;
                let outlet = Vec3::new(outlet_plan.x, low.y - 0.025, outlet_plan.y);
                let discharge = Vec3::new(outlet.x, 0.24, outlet.z);
                geometry.voids.push(ResolvedVoid {
                    id: outlet_void,
                    owner: assembly.owner,
                    bounds: SpatialBounds::<Architectural>::from_metres(
                        outlet - Vec3::splat(0.04),
                        outlet + Vec3::splat(0.04),
                    )?,
                    role: VoidRole::Drain,
                    shape: crate::ResolvedVoidShape::Box,
                    subtracts_from: assembly.owner,
                });
                geometry.drainage_routes.push(DrainageRoute {
                    id: route,
                    owner: assembly.owner,
                    outlet_void,
                    inlet: samples[0].surface_point,
                    outlet,
                });
                geometry.surfaces.push(ResolvedSurface {
                    id: catchment,
                    owner: assembly.owner,
                    bounds: roof_polygon_bounds(face.id, &face.polygon)?,
                    role: SurfaceRole::RoofDrainage,
                    shape: crate::ResolvedSurfaceShape::Planar,
                });
                geometry.drainage_catchments.push(DrainageCatchment {
                    id: catchment,
                    owner: assembly.owner,
                    walk_solid: face.id,
                    toe_channel_solids: vec![floor, lips[0], lips[1]],
                    drainage_surface: catchment,
                    outlet_route: route,
                    centre,
                    tangent: channel_tangent,
                    outward,
                    length_metres: length,
                    width_metres: 0.18,
                    inner_elevation_metres: samples[0].surface_point.y,
                    outer_elevation_metres: low.y,
                    outlet_along_metres: length * 0.5,
                });
                geometry.solids.push(ResolvedSolid::new(
                    CollisionCuboid::<Architectural>::from_metres(
                        spout,
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
                        ResolvedItemId((0x9_u64 << 60) | (spout.0 & 0x0FFF_FFFF_FFFF_FFFF)),
                        assembly.owner,
                        face.support_nodes[0],
                        SpatialBounds::<Architectural>::from_metres(
                            spout_top - Vec3::splat(0.035),
                            spout_top + Vec3::splat(0.035),
                        )?,
                    ));
                geometry.roof_drainage_networks.push(RoofDrainageNetwork {
                    id: network,
                    owner: assembly.owner,
                    face: face.id,
                    catchment,
                    receiving_edge: edge.id,
                    samples,
                    channel_floor: floor,
                    channel_lips: lips,
                    collector_solids: Vec::new(),
                    outlet_station: network,
                    outlet_void,
                    downspout: Some(spout),
                    channel_high: high,
                    channel_low: low,
                    discharge,
                });
            }
        }
    };
    Ok(())
}
