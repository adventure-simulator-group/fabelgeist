use super::*;

pub(super) fn append_oriented_cuboid(
    detail: &mut BuildingDetail,
    material: BuildingLodMaterial,
    solid: &ResolvedSolid,
    wall: Option<&crate::WallAssembly>,
) {
    let fachwerk_member = is_fachwerk_member_role(solid.role);
    let resolved_yaw = if matches!(
        solid.role,
        SolidRole::RoofFraming
            | SolidRole::RoofFlashing
            | SolidRole::RoofGutter
            | SolidRole::RoofEdgeTreatment
    ) {
        -solid.yaw_radians
    } else {
        solid.yaw_radians
    };
    let rotation = Quat::from_rotation_y(resolved_yaw)
        * Quat::from_rotation_x(solid.crossfall_radians)
        * Quat::from_rotation_z(solid.longfall_radians);
    let (render_centre, render_size) =
        render_cuboid_placement(solid, wall, fachwerk_member, rotation);
    append_cuboid_faces(detail, material, render_centre, render_size, rotation, wall);
}

pub(super) fn is_fachwerk_member_role(role: SolidRole) -> bool {
    matches!(
        role,
        SolidRole::FrameMember
            | SolidRole::FrameSill
            | SolidRole::FramePost
            | SolidRole::FramePlate
            | SolidRole::FrameRail
            | SolidRole::FrameTie
            | SolidRole::FrameBrace
            | SolidRole::FrameJettyBeam
            | SolidRole::FrameKnagge
            | SolidRole::FrameGableMember
            | SolidRole::FrameDormerTrimmer
            | SolidRole::FrameOrnament
    )
}

pub(super) fn render_cuboid_placement(
    solid: &ResolvedSolid,
    wall: Option<&crate::WallAssembly>,
    fachwerk_member: bool,
    rotation: Quat,
) -> (Vec3, Vec3) {
    let aperture_member =
        wall.is_some_and(|wall| matches!(wall.source, crate::WallSourceId::RoofGable { .. }));
    let mut render_size = if fachwerk_member && !aperture_member {
        solid.size + Vec3::splat(TIMBER_SEAM_COVER_METRES * 2.0)
    } else {
        solid.size
    };
    let mut render_centre = solid.centre;
    if let Some(wall) =
        wall.filter(|wall| fachwerk_member && wall.material == WallMaterialClass::TimberInfill)
    {
        let outward = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
        let local_axes = [rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z];
        let projected_half_extent = local_axes
            .into_iter()
            .zip(render_size.to_array())
            .map(|(axis, extent)| axis.dot(outward).abs() * extent)
            .sum::<f32>()
            * 0.5;
        let inner_plane = wall.frame.origin.dot(wall.frame.outward) - wall.thickness_metres * 0.5;
        let current_inner_extent = render_centre.dot(outward) - projected_half_extent;
        let missing_depth =
            (current_inner_extent - (inner_plane - TIMBER_SEAM_COVER_METRES)).max(0.0);

        if missing_depth > 0.0 {
            let x_alignment = local_axes[0].dot(outward).abs();
            let z_alignment = local_axes[2].dot(outward).abs();
            let (depth_axis, alignment) = if x_alignment >= z_alignment {
                (0, x_alignment)
            } else {
                (2, z_alignment)
            };
            if alignment > f32::EPSILON {
                render_size[depth_axis] += missing_depth / alignment;
                render_centre -= outward * missing_depth * 0.5;
            }
        }
    }
    (render_centre, render_size)
}

pub(super) fn append_cuboid_faces(
    detail: &mut BuildingDetail,
    material: BuildingLodMaterial,
    centre: Vec3,
    size: Vec3,
    rotation: Quat,
    wall: Option<&crate::WallAssembly>,
) {
    let half = size * 0.5;
    let local = [
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y, -half.z),
        Vec3::new(half.x, half.y, -half.z),
        Vec3::new(-half.x, half.y, -half.z),
        Vec3::new(-half.x, -half.y, half.z),
        Vec3::new(half.x, -half.y, half.z),
        Vec3::new(half.x, half.y, half.z),
        Vec3::new(-half.x, half.y, half.z),
    ];
    let point = |index: usize| centre + rotation * local[index];
    for (indices, normal, u_axis, v_axis) in [
        ([0, 3, 2, 1], -Vec3::Z, Vec3::X, Vec3::Y),
        ([4, 5, 6, 7], Vec3::Z, Vec3::X, Vec3::Y),
        ([0, 4, 7, 3], -Vec3::X, Vec3::Z, Vec3::Y),
        ([1, 2, 6, 5], Vec3::X, Vec3::Z, Vec3::Y),
        ([0, 1, 5, 4], -Vec3::Y, Vec3::X, Vec3::Z),
        ([3, 7, 6, 2], Vec3::Y, Vec3::X, Vec3::Z),
    ] {
        let positions = indices.map(point);
        let world_normal = rotation * normal;
        let wall_uvs = wall.filter(|wall| {
            let outward = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
            world_normal.dot(outward).abs() > 0.99
        });
        let timber = matches!(
            material,
            BuildingLodMaterial::Timber | BuildingLodMaterial::InteriorTimber
        );
        let face_material = if timber && crate::member_uv::grain_axis(size).dot(normal).abs() > 0.5
        {
            BuildingLodMaterial::TimberEndGrain
        } else {
            interior_face_material(material, wall, world_normal)
        };
        detail.mesh_mut(face_material).push_quad(
            positions,
            world_normal,
            if timber {
                crate::member_uv::member_uvs(positions, centre, size, rotation, normal)
            } else if let Some(wall) = wall_uvs {
                positions.map(|position| wall_surface_uv(wall, position))
            } else {
                indices.map(|index| {
                    Vec2::new(local[index].dot(u_axis), local[index].dot(v_axis))
                        / BUILDING_DETAIL_UV_METRES_PER_UNIT
                })
            },
        );
    }
}

pub(super) fn interior_face_material(
    material: BuildingLodMaterial,
    wall: Option<&crate::WallAssembly>,
    face_normal: Vec3,
) -> BuildingLodMaterial {
    let Some(wall) = wall else {
        return material;
    };
    let outward = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
    if matches!(material, BuildingLodMaterial::Wall(_))
        && wall.frame.inside_room.is_some()
        && wall.frame.outside_room.is_none()
        && face_normal.dot(outward) < -0.99
    {
        BuildingLodMaterial::InteriorPlaster
    } else {
        material
    }
}

pub(super) fn wall_surface_uv(wall: &crate::WallAssembly, point: Vec3) -> Vec2 {
    let tangent = canonical_wall_texture_tangent(wall.frame.tangent);
    Vec2::new(Vec2::new(point.x, point.z).dot(tangent), point.y)
        / BUILDING_DETAIL_UV_METRES_PER_UNIT
}

fn canonical_wall_texture_tangent(tangent: Vec2) -> Vec2 {
    if tangent.x < -f32::EPSILON || (tangent.x.abs() <= f32::EPSILON && tangent.y < 0.0) {
        -tangent
    } else {
        tangent
    }
}
