use std::collections::HashSet;

use bevy::math::{Vec2, Vec3};

use super::{BuildingLod, BuildingLodMaterial, FACADE_DETAIL_OFFSET_METRES, plan_vertex};
use crate::{BuildingPlan, OpeningUse};

pub(super) fn append_opening_details(lod: &mut BuildingLod, plan: &BuildingPlan) {
    for opening in &plan.opening_assemblies {
        let width = opening.profile.exterior_width_metres();
        let height = opening.profile.clear_height_metres();
        let tangent = opening.frame.tangent.normalize_or_zero();
        let outward = opening.frame.outward.normalize_or_zero();
        let centre = opening.frame.origin
            + outward
                * (plan
                    .wall_assemblies
                    .iter()
                    .find(|wall| wall.id == opening.host_wall)
                    .map_or(0.2, |wall| wall.thickness_metres * 0.5)
                    + FACADE_DETAIL_OFFSET_METRES);
        let left = centre - tangent * width * 0.5;
        let right = centre + tangent * width * 0.5;
        let bottom = opening.sill_elevation_metres;
        let top = bottom + height;
        let (u0, u1) = if opening
            .closure
            .layers
            .contains(&crate::ClosureKind::TimberShutter)
        {
            opening_atlas_interval(OpeningUse::Door)
        } else {
            opening_atlas_interval(opening.use_kind)
        };
        lod.mesh_mut(BuildingLodMaterial::FacadeDetails).push_quad(
            [
                plan_vertex(left, bottom),
                plan_vertex(right, bottom),
                plan_vertex(right, top),
                plan_vertex(left, top),
            ],
            Vec3::new(outward.x, 0.0, outward.y),
            [
                Vec2::new(u0, 0.0),
                Vec2::new(u1, 0.0),
                Vec2::new(u1, 1.0),
                Vec2::new(u0, 1.0),
            ],
        );
    }
}

pub(super) fn append_timber_details(lod: &mut BuildingLod, plan: &BuildingPlan) {
    let Some(frame) = &plan.timber_frame else {
        return;
    };
    let mut emitted = HashSet::new();
    for bay in &frame.bays {
        let Some(wall_id) = bay.wall else {
            continue;
        };
        let Some(wall) = plan.wall_assemblies.iter().find(|wall| wall.id == wall_id) else {
            continue;
        };
        let outward_2d = wall.frame.outward.normalize_or_zero();
        let outward = Vec3::new(outward_2d.x, 0.0, outward_2d.y);
        let surface_plane = wall.frame.origin.dot(outward_2d)
            + wall.thickness_metres * 0.5
            + FACADE_DETAIL_OFFSET_METRES;
        let plane_key = (surface_plane * 1_000.0).round() as i32;
        let outward_key = (
            (outward_2d.x * 1_000.0).round() as i16,
            (outward_2d.y * 1_000.0).round() as i16,
        );
        for member_id in &bay.member_ids {
            if !emitted.insert((*member_id, outward_key, plane_key)) {
                continue;
            }
            let Some(member) = frame.members.iter().find(|member| member.id == *member_id) else {
                continue;
            };
            let axis = (member.end - member.start).normalize_or_zero();
            let side = outward.cross(axis).normalize_or_zero() * member.section_metres.x * 0.5;
            if side.length_squared() <= f32::EPSILON || axis.dot(outward).abs() > 0.001 {
                continue;
            }
            let project = |point: Vec3| point + outward * (surface_plane - point.dot(outward));
            let start = project(member.start);
            let end = project(member.end);
            lod.mesh_mut(BuildingLodMaterial::FacadeDetails).push_quad(
                [start - side, end - side, end + side, start + side],
                outward,
                [
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.25, 0.0),
                    Vec2::new(0.25, 1.0),
                    Vec2::new(0.0, 1.0),
                ],
            );
        }
    }
}

pub(super) fn append_gable_details(lod: &mut BuildingLod, plan: &BuildingPlan) {
    let Some(frame) = &plan.timber_frame else {
        return;
    };
    for roof in plan
        .roof_assemblies
        .iter()
        .filter(|roof| roof.parent.is_none())
    {
        for face in &roof.enclosure_faces {
            for member in crate::gable_frame::members(face, frame) {
                let outward = crate::gable_frame::normal(face);
                let axis = (member.end - member.start).normalize();
                let overlap = crate::TIMBER_SEAM_COVER_METRES;
                let side = outward.cross(axis) * (member.section_metres.x * 0.5 + overlap);
                let surface = outward * (member.section_metres.y * 0.5 + overlap);
                let start = member.start + surface - axis * overlap;
                let end = member.end + surface + axis * overlap;
                lod.mesh_mut(BuildingLodMaterial::FacadeDetails).push_quad(
                    [start - side, end - side, end + side, start + side],
                    outward,
                    [
                        Vec2::new(0.0, 0.0),
                        Vec2::new(0.25, 0.0),
                        Vec2::new(0.25, 1.0),
                        Vec2::new(0.0, 1.0),
                    ],
                );
            }
        }
    }
}

fn opening_atlas_interval(kind: OpeningUse) -> (f32, f32) {
    match kind {
        OpeningUse::Window => (0.25, 0.375),
        OpeningUse::Door => (0.375, 0.5),
        OpeningUse::Gate => (0.5, 0.625),
        OpeningUse::ArrowLoop => (0.625, 0.75),
        OpeningUse::GunLoop => (0.75, 0.875),
        OpeningUse::BellOpening => (0.875, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BuildingArchetype, BuildingLodLevel, BuildingProgram, compile_solid_detail, generate,
    };

    #[test]
    fn gable_lod_corners_are_actual_exterior_timber_corners() {
        for seed in [42, 47, 101] {
            let plan = generate(&BuildingProgram::fixture(
                BuildingArchetype::FachwerkMerchantHouse,
                seed,
            ))
            .unwrap();
            let frame = plan.timber_frame.as_ref().unwrap();
            let solids = frame
                .members
                .iter()
                .map(|member| {
                    let solid = plan
                        .resolved_geometry
                        .solids
                        .iter()
                        .find(|solid| solid.id == member.solid)
                        .unwrap();
                    compile_solid_detail(&plan, solid)
                })
                .collect::<Vec<_>>();
            for level in [BuildingLodLevel::Facade, BuildingLodLevel::Shell] {
                let mut lod = BuildingLod {
                    level,
                    facade_runs: vec![],
                    meshes: vec![],
                };
                append_gable_details(&mut lod, &plan);
                let mesh = &lod.meshes[0];
                assert!(mesh.vertices.len() >= 40);
                for quad in mesh.vertices.as_chunks::<4>().0 {
                    assert!(
                        solids.iter().any(|solid| quad.iter().all(|vertex| {
                            solid
                                .meshes
                                .iter()
                                .flat_map(|mesh| &mesh.vertices)
                                .any(|exact| {
                                    exact.normal.dot(vertex.normal) > 0.999
                                        && exact.position.distance(vertex.position) < 0.0001
                                })
                        })),
                        "gable LOD must use a face of a single physical timber"
                    );
                }
            }
        }
    }

    #[test]
    fn half_hip_does_not_project_its_inset_truss_onto_the_gable() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::HallHouse, 42)).unwrap();
        let mut lod = BuildingLod {
            level: BuildingLodLevel::Shell,
            facade_runs: vec![],
            meshes: vec![],
        };
        append_gable_details(&mut lod, &plan);
        assert!(lod.meshes.is_empty());
    }
}
