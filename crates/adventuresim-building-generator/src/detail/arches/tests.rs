use super::*;
use crate::{GeometryOwnerId, ResolvedItemId, audit_triangle_mesh};

#[test]
fn arch_surface_is_closed_without_internal_section_caps() {
    let solid = ResolvedSolid {
        id: ResolvedItemId(1),
        owner: GeometryOwnerId(1),
        centre: Vec3::ZERO,
        size: Vec3::new(3.0, 1.5, 0.4),
        yaw_radians: 0.0,
        crossfall_radians: 0.0,
        longfall_radians: 0.0,
        role: SolidRole::OpeningHead,
        shape: ResolvedSolidShape::SegmentalArchRing {
            clear_span_metres: 2.0,
            spring_height_metres: 1.0,
            rise_metres: 0.5,
            ring_depth_metres: 0.4,
        },
        supported_by: Vec::new(),
    };
    let mut detail = BuildingDetail { meshes: Vec::new() };
    assert!(append(
        &mut detail,
        BuildingLodMaterial::DressedStone,
        &solid,
        None
    ));
    let mesh = &detail.meshes[0];
    let positions: Vec<_> = mesh
        .vertices
        .iter()
        .map(|v| v.position.to_array())
        .collect();
    let audit = audit_triangle_mesh(&positions, &mesh.indices);
    assert!(audit.passes_closed_solid(), "{audit:?}");
    for vertex in &mesh.vertices {
        if vertex.normal.x.abs() > 0.999 {
            assert!((vertex.position.x.abs() - 1.5).abs() < 0.0001);
        }
    }
}
