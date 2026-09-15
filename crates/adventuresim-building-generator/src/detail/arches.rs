use super::*;

#[cfg(test)]
mod tests;

/// Render the actual resolved arch cut, retaining the same sections as collision.
pub(super) fn append(
    detail: &mut BuildingDetail,
    material: BuildingLodMaterial,
    solid: &ResolvedSolid,
    wall: Option<&crate::WallAssembly>,
) -> bool {
    let Some(arch) = crate::arch_geometry::ArchGeometry::from_solid(solid, wall) else {
        return false;
    };
    let mesh = detail.mesh_mut(material);
    let strips = arch.strips();
    for (section, strip) in strips.iter().enumerate() {
        let front = strip.front;
        let back = front.map(|point| point + strip.depth);
        let mut faces = vec![front, [back[3], back[2], back[1], back[0]]];
        for index in 0..4 {
            // Adjacent strips share their vertical ends. Keep only the two
            // exterior caps; all visible surfaces retain their exact sections.
            if (index == 1 && section + 1 < strips.len()) || (index == 3 && section > 0) {
                continue;
            }
            let next = (index + 1) % 4;
            faces.push([front[index], back[index], back[next], front[next]]);
        }
        for face in faces {
            let normal = (face[1] - face[0])
                .cross(face[2] - face[0])
                .normalize_or_zero();
            if normal == Vec3::ZERO {
                continue;
            }
            let tangent = if normal.y.abs() < 0.99 {
                Vec3::new(normal.z, 0.0, -normal.x).normalize()
            } else {
                Vec3::X
            };
            let bitangent = normal.cross(tangent);
            mesh.push_quad(
                face,
                normal,
                face.map(|point| {
                    Vec2::new(point.dot(tangent), point.dot(bitangent))
                        / BUILDING_DETAIL_UV_METRES_PER_UNIT
                }),
            );
        }
    }
    true
}
