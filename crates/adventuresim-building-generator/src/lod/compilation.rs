//! Select semantic exterior assemblies and compile their render representations.
use super::*;

/// Compiles a render-only LOD from the accepted semantic plan.
pub fn compile_building_lod(plan: &BuildingPlan, level: BuildingLodLevel) -> BuildingLod {
    if plan.small_church.is_some() {
        return small_church::compile(plan, level);
    }
    let facade_runs = extract_facade_runs(plan);
    let mut lod = BuildingLod {
        level,
        facade_runs,
        meshes: Vec::new(),
    };
    if let Some(workplace) = &plan.workplace {
        lod.facade_runs.retain(|run| {
            !run.source_walls
                .iter()
                .any(|id| workplace.walls.contains(id))
        });
    }
    if plan.church.is_some() && level == BuildingLodLevel::Facade {
        exterior::append_facades(&mut lod, plan);
    } else {
        append_wall_envelopes(&mut lod);
    }
    urban_church::append_buttresses(&mut lod, plan);
    append_roofs(&mut lod, plan);
    if plan.church.is_none() || level == BuildingLodLevel::Shell {
        append_opening_details(&mut lod, plan);
    }
    append_timber_details(&mut lod, plan);
    append_gable_details(&mut lod, plan);
    append_crowns(&mut lod, plan);
    for batch in crate::detail::compile_workplace_lod(plan).meshes {
        let target = lod.mesh_mut(batch.material);
        let offset = target.vertices.len() as u32;
        target.vertices.extend(batch.vertices);
        target
            .indices
            .extend(batch.indices.into_iter().map(|index| index + offset));
    }
    lod.meshes
        .retain(|mesh| !mesh.vertices.is_empty() && !mesh.indices.is_empty());
    for mesh in &mut lod.meshes {
        mesh.remap_vertices();
    }
    lod
}

pub(super) fn extract_facade_runs(plan: &BuildingPlan) -> Vec<FacadeRun> {
    let mut runs = plan
        .wall_assemblies
        .iter()
        .filter(|wall| {
            wall.replaced_by_owner.is_none()
                && (wall.frame.outside_room.is_none()
                    // Tower walls straddle the nave roof: a ground-level room
                    // neighbour does not hide their exposed upper weather face.
                    || (plan.church.is_some()
                        && matches!(wall.source, crate::WallSourceId::ChurchTowerFace { .. } | crate::WallSourceId::SquareTowerFace { .. })))
                && !matches!(
                    wall.material,
                    WallMaterialClass::InternalTimber | WallMaterialClass::InternalMasonry
                )
        })
        .map(|wall| {
            if let Some(radial) = wall.radial_frame {
                let radius = wall.length_metres / std::f32::consts::TAU;
                return FacadeRun {
                    material: wall.material,
                    storey_level: wall.storey_level,
                    path: FacadeRunPath::Round {
                        centre: radial.centre,
                        radius_metres: radius,
                        reference_outward: radial.reference_outward,
                    },
                    base_elevation_metres: wall.base_elevation_metres,
                    height_metres: wall.height_metres,
                    thickness_metres: wall.thickness_metres,
                    source_walls: vec![wall.id],
                };
            }
            let mut tangent = wall.frame.tangent.normalize_or_zero();
            if tangent.x < -0.001 || (tangent.x.abs() <= 0.001 && tangent.y < 0.0) {
                tangent = -tangent;
            }
            FacadeRun {
                material: wall.material,
                storey_level: wall.storey_level,
                path: FacadeRunPath::Straight {
                    start: wall.frame.origin - tangent * wall.length_metres * 0.5,
                    end: wall.frame.origin + tangent * wall.length_metres * 0.5,
                    outward: wall.frame.outward,
                },
                base_elevation_metres: wall.base_elevation_metres,
                height_metres: wall.height_metres,
                thickness_metres: wall.thickness_metres,
                source_walls: vec![wall.id],
            }
        })
        .collect::<Vec<_>>();

    let mut changed = true;
    while changed {
        changed = false;
        'outer: for left in 0..runs.len() {
            for right in left + 1..runs.len() {
                if runs[left].can_join(&runs[right]) {
                    let other = runs.remove(right);
                    runs[left].join(other);
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    runs
}
