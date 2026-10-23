//! Install worker-built occupied geometry without repeating mesh compilation.
use super::*;

pub(super) fn install(
    cache: &mut TacticalBuildingMeshCache,
    program: &BuildingProgram,
    geometry: Arc<super::super::generation::VenueGeometry>,
    recipe: &GeneratedBuildingRecipe,
    meshes: &mut Assets<Mesh>,
) -> Result<Arc<CompiledBuildingLevels>> {
    let mut batches = |source: &[super::super::generation::venue::PreparedBatch]| {
        source
            .iter()
            .map(|batch| {
                let material = batch.material;
                let triangles = batch.triangle_count();
                let mesh = batch.clone().into_mesh();
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
    let origin = recipe.collision.bounds.centre()?;
    let compiled = Arc::new(CompiledBuildingLevels {
        facade_openings: recipe.plan.facade_dynamic_openings()?,
        interior: Some(super::super::interior_lighting::InteriorField::from_plan(
            &recipe.plan,
            origin.metres(),
        )),
        program: program.clone(),
        detail: BuildingDetail::Dynamic,

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
        lod0: batches(&geometry.detail),
        lod1: batches(&geometry.facade),
        lod2: batches(&geometry.shell),
    });
    cache.levels.push(compiled.clone());
    Ok(compiled)
}
