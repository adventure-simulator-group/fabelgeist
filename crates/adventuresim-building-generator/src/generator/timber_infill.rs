//! Resolve wall-field residual material after the complete opening frame.
use super::*;

pub(super) fn resolve(
    builder: &mut TimberFrameBuilder<'_>,
    walls: &mut [crate::WallAssembly],
    openings: &[crate::OpeningAssembly],
    bays: &mut [crate::TimberFrameBay],
) {
    // Replace monolithic Stage 3 WallHost leaves with bay-local infill
    // panels. Opening jamb/head/sill/spandrel solids retain their independent
    // bearing authority; these residual panels cover only the wall field
    // around the opening and sit behind the structural timber layer.
    let mut removed_panel_ids = std::collections::HashSet::new();
    for wall in walls
        .iter_mut()
        .filter(|wall| wall.material == crate::WallMaterialClass::TimberInfill)
    {
        let old_panels = wall
            .host_solids
            .iter()
            .copied()
            .filter(|id| {
                builder
                    .geometry
                    .solids
                    .iter()
                    .any(|solid| solid.id == *id && solid.role == SolidRole::WallHost)
            })
            .collect::<Vec<_>>();
        removed_panel_ids.extend(old_panels.iter().copied());
        wall.host_solids.retain(|id| !old_panels.contains(id));

        let residual = residual(builder, wall, openings, bays);
        recess_opening_hosts(builder.geometry, wall);
        let panel_ids = append_panels(builder.geometry, wall, residual);
        for bay in bays.iter_mut().filter(|bay| bay.wall == Some(wall.id)) {
            bay.infill_solids = panel_ids.clone();
        }
    }
    builder
        .geometry
        .solids
        .retain(|solid| !removed_panel_ids.contains(&solid.id));
}

fn residual(
    builder: &TimberFrameBuilder<'_>,
    wall: &crate::WallAssembly,
    openings: &[crate::OpeningAssembly],
    bays: &[crate::TimberFrameBay],
) -> MultiPolygon<f32> {
    let half_length = wall.length_metres * 0.5;
    let field = closed_polygon([
        Vec2::new(-half_length, 0.0),
        Vec2::new(half_length, 0.0),
        Vec2::new(half_length, wall.height_metres),
        Vec2::new(-half_length, wall.height_metres),
    ]);
    let mut residual = MultiPolygon(vec![field]);
    for opening in openings
        .iter()
        .filter(|opening| opening.host_wall == wall.id)
    {
        let half_opening = (opening.profile.interior_width_metres() * 0.5).min(half_length - 0.02);
        let centre = (opening.frame.origin - wall.frame.origin).dot(wall.frame.tangent);
        let sill = (opening.sill_elevation_metres - wall.base_elevation_metres)
            .clamp(0.0, wall.height_metres);
        let head = (sill + opening.profile.clear_height_metres()).clamp(sill, wall.height_metres);
        let opening_polygon = closed_polygon([
            Vec2::new(centre - half_opening, sill),
            Vec2::new(centre + half_opening, sill),
            Vec2::new(centre + half_opening, head),
            Vec2::new(centre - half_opening, head),
        ]);
        residual = residual.difference(&opening_polygon);
    }
    let wall_member_ids = bays
        .iter()
        .filter(|bay| bay.wall == Some(wall.id))
        .flat_map(|bay| bay.member_ids.iter().copied())
        .collect::<std::collections::HashSet<_>>();
    for member in builder
        .members
        .iter()
        .filter(|member| wall_member_ids.contains(&member.id))
    {
        residual = residual.difference(&timber_member_wall_polygon(member, wall));
    }

    residual
}

fn recess_opening_hosts(geometry: &mut ResolvedGeometry, wall: &crate::WallAssembly) {
    // Stage 3 opening-bearing solids retain the structural wall depth, but
    // their exposed face is recessed from the Fachwerk plane. Their exact
    // overlap with the opening's jamb/header members is a typed composite
    // opening-frame relation audited below; unrelated timber receives no
    // such permission.
    let opening_recess = 0.012_f32.min(wall.thickness_metres - 0.04);
    let inward = -wall.frame.outward;
    for solid in geometry.solids.iter_mut().filter(|solid| {
        wall.host_solids.contains(&solid.id)
            && matches!(
                solid.role,
                SolidRole::OpeningJamb
                    | SolidRole::OpeningSill
                    | SolidRole::OpeningHead
                    | SolidRole::OpeningSpandrel
            )
    }) {
        solid.centre += Vec3::new(inward.x, 0.0, inward.y) * opening_recess * 0.5;
        if wall.frame.outward.x.abs() > 0.5 {
            solid.size.x = (solid.size.x - opening_recess).max(0.04);
        } else {
            solid.size.z = (solid.size.z - opening_recess).max(0.04);
        }
    }
}

fn append_panels(
    geometry: &mut ResolvedGeometry,
    wall: &mut crate::WallAssembly,
    residual: MultiPolygon<f32>,
) -> Vec<ResolvedItemId> {
    let panel_depth = timber_infill_panel_depth(wall);
    let mut panel_ids = Vec::new();
    let triangles = residual
        .0
        .iter()
        .flat_map(triangulate_panel_polygon)
        .collect::<Vec<_>>();
    for (index, triangle) in triangles.into_iter().enumerate() {
        let id = ResolvedItemId(
            (1_u64 << 60) | (u64::from(wall.owner.0) << 32) | 0x0f00_0000 | index as u64,
        );
        let contact = ResolvedItemId(
            (4_u64 << 60) | (u64::from(wall.owner.0) << 32) | 0x0f00_0000 | index as u64,
        );
        let mid_plane = timber_infill_mid_plane(wall);
        let vertices = triangle.map(|point| {
            let plan = mid_plane + wall.frame.tangent * point.x;
            Vec3::new(plan.x, wall.base_elevation_metres + point.y, plan.y)
        });
        let depth_offset =
            Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y) * panel_depth * 0.5;
        let min = vertices
            .iter()
            .flat_map(|vertex| [*vertex - depth_offset, *vertex + depth_offset])
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let max = vertices
            .iter()
            .flat_map(|vertex| [*vertex - depth_offset, *vertex + depth_offset])
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        let centre = (min + max) * 0.5;
        let size = max - min;
        geometry.solids.push(ResolvedSolid {
            id,
            owner: wall.owner,
            centre,
            size,
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::WallHost,
            shape: crate::ResolvedSolidShape::TimberPanelPrism {
                vertices,
                outward: wall.frame.outward,
                depth_metres: panel_depth,
            },
            supported_by: vec![wall.support_node],
        });
        geometry.support_interfaces.push(SupportInterface {
            id: contact,
            owner: wall.owner,
            node: wall.support_node,
            bounds: ResolvedBounds {
                min: Vec3::new(
                    centre.x - size.x * 0.5,
                    centre.y - size.y * 0.5 - 0.004,
                    centre.z - size.z * 0.5,
                ),
                max: Vec3::new(
                    centre.x + size.x * 0.5,
                    centre.y - size.y * 0.5 + 0.008,
                    centre.z + size.z * 0.5,
                ),
            },
        });
        wall.host_solids.push(id);
        panel_ids.push(id);
    }
    panel_ids
}
