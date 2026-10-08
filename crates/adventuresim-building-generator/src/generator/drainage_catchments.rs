fn resolved_axis_bounds(
    centre: Vec3,
    size: Vec3,
) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
    Ok(SpatialBounds::<Architectural>::from_metres(
        centre - size * 0.5,
        centre + size * 0.5,
    )?)
}

fn push_drainage_catchment(
    geometry: &mut ResolvedGeometry,
    crown: &CrownAssembly,
    route: DrainageRoute,
    centre: Vec2,
    tangent: Vec2,
    outward: Vec2,
    length_metres: f32,
) -> Result<(), crate::GenerationError> {
    let width_metres = crown.profile.walk_clear_width_metres;
    let channel_width_metres = CROWN_DRAIN_CHANNEL_WIDTH_METRES;
    let slab_width_metres = width_metres - channel_width_metres;
    let outer_elevation_metres = crown.base_height_metres + 0.02;
    let inner_elevation_metres = outer_elevation_metres + 0.06;
    let yaw_radians = -tangent.y.atan2(tangent.x);
    let local_z = Vec2::new(yaw_radians.sin(), yaw_radians.cos());
    let crossfall_sign = local_z.dot(outward).signum();
    let crossfall_radians = crossfall_sign
        * ((inner_elevation_metres - outer_elevation_metres) / slab_width_metres).atan();
    let slab_thickness = 0.12;
    let solid_index = geometry.solids.len();
    let solid_id =
        ResolvedItemId((1_u64 << 60) | (u64::from(crown.owner.0) << 32) | solid_index as u64);
    let support_node = StructuralNodeId(u64::from(crown.owner.0) * 10 + 1);
    let slab_centre = centre - outward * (channel_width_metres * 0.5);
    let solid = ResolvedSolid::new(
        CollisionCuboid::<Architectural>::from_metres(
            solid_id,
            Vec3::new(
                slab_centre.x,
                (inner_elevation_metres + outer_elevation_metres) * 0.5 - slab_thickness * 0.5,
                slab_centre.y,
            ),
            Vec3::new(length_metres, slab_thickness, slab_width_metres),
            yaw_radians,
            crossfall_radians,
            0.0,
        )?,
        crown.owner,
        SolidRole::WalkSurface,
        crate::ResolvedSolidShape::Cuboid,
        vec![support_node],
    );
    let toe_centre = centre + outward * (width_metres * 0.5 - channel_width_metres * 0.5);
    let inlet = Vec2::new(route.inlet.x, route.inlet.z);
    let outlet_along_metres = (inlet - toe_centre).dot(tangent);
    let outlet_end = if outlet_along_metres >= 0.0 {
        crown_drainage::ChannelOutletEnd::PositiveTangent
    } else {
        crown_drainage::ChannelOutletEnd::NegativeTangent
    };
    let channel_points = crown_drainage::CrownChannelSetOut::from_metres(
        crown.owner,
        toe_centre,
        inlet,
        tangent,
        length_metres,
        outlet_end,
    )?
    .points(crown.path);
    // Keep the entire open channel floor below the scupper's lower edge. The
    // high end is still below the adjacent toe, so water never has to climb a
    // renderer-only curb before reaching the outlet.
    let channel_drop_metres = 0.018;
    let channel_thickness = 0.05;
    let tangent_extent = tangent.abs() * (length_metres * 0.5);
    let outward_extent = outward.abs() * (width_metres * 0.5);
    let extent = tangent_extent + outward_extent;
    let slab_tangent_extent = tangent.abs() * (length_metres * 0.5);
    let slab_outward_extent = outward.abs() * (slab_width_metres * 0.5);
    let slab_extent = slab_tangent_extent + slab_outward_extent;
    let surface_index = geometry.surfaces.len();
    let surface_id =
        ResolvedItemId((2_u64 << 60) | (u64::from(crown.owner.0) << 32) | surface_index as u64);
    let solid_bottom = solid.centre.metres().y - solid.size.metres().y * 0.5;
    geometry.solids.push(solid);
    let mut channel_ids = Vec::new();
    let channel_count = channel_points.len() - 1;
    for (segment, points) in channel_points.windows(2).enumerate() {
        let channel_index = geometry.solids.len();
        let channel_id =
            ResolvedItemId((1_u64 << 60) | (u64::from(crown.owner.0) << 32) | channel_index as u64);
        let delta = points[1] - points[0];
        let channel_length = delta.length().max(0.04);
        let channel_tangent = delta.normalize_or(tangent);
        let start_height =
            route.inlet.y + channel_drop_metres * (1.0 - segment as f32 / channel_count as f32);
        let end_height = route.inlet.y
            + channel_drop_metres * (1.0 - (segment + 1) as f32 / channel_count as f32);
        let channel = ResolvedSolid::new(
            CollisionCuboid::<Architectural>::from_metres(
                channel_id,
                Vec3::new(
                    (points[0].x + points[1].x) * 0.5,
                    (start_height + end_height) * 0.5 - channel_thickness * 0.5,
                    (points[0].y + points[1].y) * 0.5,
                ),
                Vec3::new(channel_length, channel_thickness, channel_width_metres),
                -channel_tangent.y.atan2(channel_tangent.x),
                0.0,
                -((start_height - end_height) / channel_length).atan(),
            )?,
            crown.owner,
            SolidRole::DrainageChannel,
            crate::ResolvedSolidShape::Cuboid,
            vec![support_node],
        );
        let channel_bottom = channel.centre.metres().y - channel.size.metres().y * 0.5;
        let channel_extent = channel_tangent.abs() * (channel_length * 0.5)
            + Vec2::new(-channel_tangent.y, channel_tangent.x).abs() * 0.08;
        geometry
            .support_interfaces
            .push(crate::SupportInterface::new(
                ResolvedItemId((4_u64 << 60) | channel_index as u64),
                crown.owner,
                support_node,
                SpatialBounds::<Architectural>::from_metres(
                    Vec3::new(
                        (points[0].x + points[1].x) * 0.5 - channel_extent.x,
                        channel_bottom - 0.015,
                        (points[0].y + points[1].y) * 0.5 - channel_extent.y,
                    ),
                    Vec3::new(
                        (points[0].x + points[1].x) * 0.5 + channel_extent.x,
                        channel_bottom + 0.015,
                        (points[0].y + points[1].y) * 0.5 + channel_extent.y,
                    ),
                )?,
            ));
        geometry.solids.push(channel);
        channel_ids.push(channel_id);
    }
    geometry.surfaces.push(ResolvedSurface {
        id: surface_id,
        owner: crown.owner,
        bounds: SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                centre.x - extent.x,
                outer_elevation_metres,
                centre.y - extent.y,
            ),
            Vec3::new(
                centre.x + extent.x,
                inner_elevation_metres,
                centre.y + extent.y,
            ),
        )?,
        role: SurfaceRole::Drainage,
        shape: crate::ResolvedSurfaceShape::Planar,
    });
    geometry
        .support_interfaces
        .push(crate::SupportInterface::new(
            ResolvedItemId((4_u64 << 60) | solid_index as u64),
            crown.owner,
            support_node,
            SpatialBounds::<Architectural>::from_metres(
                Vec3::new(
                    slab_centre.x - slab_extent.x,
                    solid_bottom - 0.015,
                    slab_centre.y - slab_extent.y,
                ),
                Vec3::new(
                    slab_centre.x + slab_extent.x,
                    solid_bottom + 0.015,
                    slab_centre.y + slab_extent.y,
                ),
            )?,
        ));
    geometry.drainage_catchments.push(DrainageCatchment {
        id: ResolvedItemId((7_u64 << 60) | geometry.drainage_catchments.len() as u64),
        owner: crown.owner,
        walk_solid: solid_id,
        toe_channel_solids: channel_ids,
        drainage_surface: surface_id,
        outlet_route: route.id,
        centre: Vec3::new(
            centre.x,
            (inner_elevation_metres + outer_elevation_metres) * 0.5,
            centre.y,
        ),
        tangent,
        outward,
        length_metres,
        width_metres,
        inner_elevation_metres,
        outer_elevation_metres,
        outlet_along_metres,
    });

    Ok(())
}
