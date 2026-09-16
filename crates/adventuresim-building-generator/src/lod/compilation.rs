//! Select semantic exterior assemblies and compile their render representations.
use super::*;

/// Compiles a render-only LOD from the accepted semantic plan.
pub fn compile_building_lod(plan: &BuildingPlan, level: BuildingLodLevel) -> BuildingLod {
    compile(plan, level, &std::collections::BTreeSet::new())
}

/// Reserve operable leaves for the same dynamic entities through Facade distance.
/// Shell retains its coarse authored enclosure representation.
pub fn compile_static_building_lod(plan: &BuildingPlan, level: BuildingLodLevel) -> BuildingLod {
    compile(plan, level, &crate::detail::dynamic_closure_solids(plan))
}

fn compile(
    plan: &BuildingPlan,
    level: BuildingLodLevel,
    excluded: &std::collections::BTreeSet<crate::ResolvedItemId>,
) -> BuildingLod {
    if plan.small_church.is_some() {
        return small_church::compile(plan, level, excluded);
    }
    let facade_runs = retained_facade_runs(plan);
    let mut lod = BuildingLod {
        level,
        facade_runs,
        meshes: Vec::new(),
    };
    let exact_facade = level == BuildingLodLevel::Facade && closures::exact_facade(plan);
    if exact_facade {
        exterior::append_facades(&mut lod, plan, excluded);
    } else {
        append_wall_envelopes(&mut lod);
    }
    urban_church::append_buttresses(&mut lod, plan);
    append_roofs(&mut lod, plan);
    gable_openings::append(&mut lod, plan);
    if !exact_facade {
        append_opening_details(&mut lod, plan);
        append_timber_details(&mut lod, plan);
    }
    append_gable_details(&mut lod, plan);
    append_crowns(&mut lod, plan);
    for batch in crate::detail::compile_workplace_lod(plan)
        .meshes
        .into_iter()
        .chain(crate::detail::compile_heating_lod(plan).meshes)
    {
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

/// Retain complete joined runs in the render compiler and capability query.
pub(super) fn retained_facade_runs(plan: &BuildingPlan) -> Vec<FacadeRun> {
    extract_facade_runs(plan)
        .into_iter()
        .filter(|run| {
            !run.source_walls.iter().any(|id| {
                plan.workplace
                    .as_ref()
                    .is_some_and(|work| work.walls.contains(id))
                    || plan
                        .small_church
                        .as_ref()
                        .is_some_and(|church| church.bearing_walls.contains(id))
            })
        })
        .collect()
}

pub(super) fn extract_facade_runs(plan: &BuildingPlan) -> Vec<FacadeRun> {
    let mut runs = plan
        .wall_assemblies
        .iter()
        .filter(|wall| {
            wall.replaced_by_owner.is_none()
                && !matches!(wall.source, crate::WallSourceId::RoofGable { .. })
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
