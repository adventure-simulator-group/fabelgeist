//! Install worker-built occupied geometry without repeating mesh compilation.
use super::*;

pub(super) fn install(
    cache: &mut TacticalBuildingMeshCache,
    program: &BuildingProgram,
    geometry: super::super::generation::VenueGeometry,
    recipe: &GeneratedBuildingRecipe,
    meshes: &mut Assets<Mesh>,
) -> Arc<CompiledBuildingLevels> {
    let mut batches = |source: Vec<super::super::generation::venue::PreparedBatch>| {
        source
            .into_iter()
            .map(|batch| {
                let material = batch.material;
                let mesh = batch.into_mesh();
                let triangles = mesh.indices().expect("prepared triangle indices").len() / 3;
                CompiledBuildingBatch {
                    material,
                    mesh: meshes.add(mesh),
                    triangles,
                    transform: Mat4::IDENTITY,
                    uv_offset: Vec2::ZERO,
                }
            })
            .collect()
    };
    let origin = recipe.collision.bounds.centre();
    let compiled = Arc::new(CompiledBuildingLevels {
        facade_openings: recipe.plan.facade_dynamic_openings(),
        interior: Some(super::super::interior_lighting::InteriorField::from_plan(
            &recipe.plan,
            origin,
        )),
        program: program.clone(),
        detail: BuildingDetail::Dynamic,
        floor_offset_metres: origin.y - recipe.collision.bounds.min.y,
        local_origin: origin,
        sign_sites: if program
            .usage
            .and_then(adventuresim_building_generator::signs::shop_trade)
            .is_some()
        {
            signs::sites(&recipe.plan)
        } else {
            Vec::new()
        },
        lod0: batches(geometry.detail),
        lod1: batches(geometry.facade),
        lod2: batches(geometry.shell),
    });
    cache.levels.push(compiled.clone());
    compiled
}
