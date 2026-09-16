//! Aperture hosts and fixed closures remain real solids at both distance levels.
use super::*;

pub(super) fn append(lod: &mut BuildingLod, plan: &BuildingPlan) {
    let mut ids = std::collections::BTreeSet::new();
    for wall in plan
        .wall_assemblies
        .iter()
        .filter(|wall| matches!(wall.source, crate::WallSourceId::RoofGable { .. }))
    {
        ids.extend(&wall.host_solids);
        for opening in plan
            .opening_assemblies
            .iter()
            .filter(|opening| opening.host_wall == wall.id)
        {
            ids.extend(&opening.closure_solids);
        }
        if let Some(frame) = &plan.timber_frame {
            for bay in frame.bays.iter().filter(|bay| bay.wall == Some(wall.id)) {
                ids.extend(
                    frame
                        .members
                        .iter()
                        .filter(|member| bay.member_ids.contains(&member.id))
                        .map(|m| m.solid),
                );
            }
        }
    }
    for solid in plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| ids.contains(&solid.id))
    {
        for mesh in crate::compile_solid_detail(plan, solid).meshes {
            let target = lod.mesh_mut(mesh.material);
            let offset = target.vertices.len() as u32;
            target.vertices.extend(mesh.vertices);
            target
                .indices
                .extend(mesh.indices.into_iter().map(|index| index + offset));
        }
    }
}

pub(super) fn owns_member(plan: &BuildingPlan, id: crate::TimberMemberId) -> bool {
    plan.timber_frame.as_ref().is_some_and(|frame| {
        frame.bays.iter().any(|bay| {
            bay.member_ids.contains(&id)
                && bay.wall.is_some_and(|wall| {
                    plan.wall_assemblies.iter().any(|w| {
                        w.id == wall && matches!(w.source, crate::WallSourceId::RoofGable { .. })
                    })
                })
        })
    })
}
