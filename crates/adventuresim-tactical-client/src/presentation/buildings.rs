use adventuresim_building_generator::{
    BuildingLodLevel, BuildingLodMaterial, BuildingProgram, LodMesh, compile_building_lod,
    compile_static_building_detail, compile_static_building_lod,
};
use adventuresim_tactical_core::scene_input::{GeneratedBuildingRecipe, GeneratedBuildingRecipes};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use std::sync::Arc;

use super::recipe_mesh::recipe_mesh;
use super::*;

mod boundaries;
mod gpu;
mod kit;
mod materials;
#[cfg(any(target_family = "wasm", test))]
mod prepared;
pub(in crate::presentation) mod signs;
mod streaming;
pub(crate) use materials::TacticalBuildingMaterials;
pub(in crate::presentation) use materials::setup_tactical_building_materials;
pub(in crate::presentation) use signs::BuildingPresentationPlugin;
pub(crate) use signs::PresentedSign;
pub(crate) use streaming::PendingCityBuildings;

pub(crate) fn city_gpu_ready() -> bool {
    gpu::is_ready()
}
pub(super) fn reset_gpu(world: &mut World) {
    gpu::reset(world);
}

pub(super) const DETAIL_LOD_END_START_METRES: f32 = 55.0;
pub(super) const DETAIL_LOD_END_END_METRES: f32 = 70.0;
const FACADE_LOD_END_START_METRES: f32 = 150.0;
const FACADE_LOD_END_END_METRES: f32 = 175.0;

#[derive(Component)]
pub(crate) struct DistantCityBuildingPresentation;

#[derive(Component)]
pub(crate) struct PresentedBuildingMesh {
    pub(crate) level: BuildingRenderLevel,
    pub(crate) material: BuildingLodMaterial,
    pub(crate) triangles: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BuildingRenderLevel {
    Lod0,
    Lod1,
    Lod2,
}

#[derive(Clone)]
struct CompiledBuildingBatch {
    material: BuildingLodMaterial,
    mesh: Handle<Mesh>,
    triangles: usize,
    transform: Mat4,
    uv_offset: Vec2,
}

impl CompiledBuildingBatch {
    fn from_recipe(
        batch: &LodMesh,
        origin: Vec3,
        detail: BuildingDetail,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        let mut mesh = recipe_mesh(batch, origin);
        if detail != BuildingDetail::Dynamic {
            // GPU assembly owns storage-buffer uploads for these vertices.
            mesh.asset_usage = RenderAssetUsages::MAIN_WORLD;
        }
        Self {
            material: batch.material,
            mesh: meshes.add(mesh),
            triangles: batch.indices.len() / 3,
            transform: Mat4::IDENTITY,
            uv_offset: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuildingDetail {
    Dynamic,
    Static,
    Facade,
}

#[derive(Clone)]
struct CompiledBuildingLevels {
    facade_openings: std::collections::BTreeSet<adventuresim_building_generator::OpeningAssemblyId>,
    interior: Option<super::interior_lighting::InteriorField>,
    program: BuildingProgram,
    detail: BuildingDetail,

    local_origin: adventuresim_building_generator::spatial_geometry::Position<
        adventuresim_building_generator::spatial_geometry::Architectural,
    >,
    sign_sites: Vec<(
        adventuresim_building_generator::signs::SignMount,
        adventuresim_building_generator::signs::SignSite,
    )>,
    lod0: Vec<CompiledBuildingBatch>,
    lod1: Vec<CompiledBuildingBatch>,
    lod2: Vec<CompiledBuildingBatch>,
}

#[derive(Default, Resource)]
pub(crate) struct TacticalBuildingMeshCache {
    levels: Vec<Arc<CompiledBuildingLevels>>,
    components: kit::ComponentCache,
    pub(crate) recipes: GeneratedBuildingRecipes,
}

/// Client-generated geometry transferred to the descriptor observer and then
/// released. Replicated descriptors instead compile their program on arrival.
#[derive(Component)]
pub(crate) struct PreparedBuildingGeometry(pub(crate) GeneratedBuildingRecipe);

#[derive(bevy::ecs::query::QueryData)]
struct BuildingSource {
    building: &'static SceneBuilding,
    establishment: Option<&'static SceneEstablishment>,
    authored_sign: Option<&'static adventuresim_building_generator::signs::ShopSign>,
    prepared: Option<&'static PreparedBuildingGeometry>,
}

fn on_scene_building_added(
    event: On<Add, SceneBuilding>,
    mut commands: Commands,
    buildings: Query<BuildingSource>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
    mut cache: ResMut<TacticalBuildingMeshCache>,
    mut signs: signs::SignAssets,
) -> Result {
    let BuildingSourceItem {
        building,
        establishment,
        authored_sign,
        prepared,
    } = buildings.get(event.entity)?;
    let resolved_sign = authored_sign.cloned().or_else(|| {
        establishment.and_then(|establishment| {
            establishment.shop_name.clone().and_then(|name| {
                adventuresim_building_generator::signs::ShopSign::for_establishment(
                    adventuresim_building_generator::signs::EstablishmentId(building.id),
                    establishment.business_id.key.usage,
                    name,
                )
            })
        })
    });
    let compiled = cached_building_levels(
        &mut cache,
        &building.program,
        BuildingDetail::Dynamic,
        &mut meshes,
        prepared.map(|prepared| &prepared.0),
    )?;
    commands
        .entity(event.entity)
        .remove::<PreparedBuildingGeometry>()
        .insert((
            Visibility::default(),
            super::building_closures::FacadeOpenings(compiled.facade_openings.clone()),
            compiled
                .interior
                .clone()
                .ok_or("playable building needs interior lighting")?,
        ))
        .with_children(|parent| {
            spawn_building_levels(parent, building.id, &compiled, &materials);
            signs.spawn(parent, resolved_sign.as_ref(), &compiled, &mut meshes);
        });
    Ok(())
}

fn on_scene_vista_buildings(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    existing: Query<Entity, With<DistantCityBuildingPresentation>>,
    streaming: Option<Res<StreamCityTraffic>>,
    mut assets: streaming::CityBuildingAssets,
) -> Result {
    assets.gpu.clear();
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if streaming.is_some() {
        commands.insert_resource(PendingCityBuildings::new(
            &bundle.distant_buildings,
            &bundle.establishments,
        ));
    } else {
        for placement in &bundle.distant_buildings {
            let establishment = bundle
                .establishments
                .iter()
                .find(|establishment| establishment.building_id == placement.id);
            assets.spawn(
                &mut commands,
                placement,
                establishment,
                BuildingDetail::Static,
            )?;
        }
    }
    Ok(())
}

fn cached_building_levels(
    cache: &mut TacticalBuildingMeshCache,
    program: &BuildingProgram,
    detail: BuildingDetail,
    meshes: &mut Assets<Mesh>,
    prepared: Option<&GeneratedBuildingRecipe>,
) -> Result<Arc<CompiledBuildingLevels>> {
    if let Some(compiled) = cache
        .levels
        .iter()
        .find(|compiled| compiled.program == *program && compiled.detail == detail)
    {
        return Ok(compiled.clone());
    }

    #[cfg(target_family = "wasm")]
    if detail == BuildingDetail::Facade {
        let prepared = super::generation::take_facade(program)?;
        return Ok(kit::install_facade(cache, prepared, meshes));
    }

    let generated;
    let geometry = if let Some(prepared) = prepared {
        prepared
    } else {
        generated = match cache.recipes.take(program) {
            Some(recipe) => recipe,
            None => GeneratedBuildingRecipe::generate(program.clone())?,
        };
        &generated
    };
    let plan = &geometry.plan;
    #[cfg(target_family = "wasm")]
    if detail == BuildingDetail::Dynamic && prepared.is_some() {
        let meshes_ready = super::generation::take_venue_geometry(program)?;
        return self::prepared::install(cache, program, meshes_ready, geometry, meshes);
    }
    let collision = &geometry.collision;
    let local_origin = collision.bounds.centre()?;
    let RecipeMeshes {
        kit,
        detail_meshes,
        facade,
    } = RecipeMeshes::compile(plan, detail)?;
    let shell = match (detail == BuildingDetail::Facade)
        .then(|| adventuresim_building_generator::compile_program_shell(program))
        .flatten()
    {
        Some(shell) => shell,
        None => compile_building_lod(plan, BuildingLodLevel::Shell)?,
    };
    let compile_batches = |source: &[LodMesh], meshes: &mut Assets<Mesh>| {
        source
            .iter()
            .map(|batch| {
                CompiledBuildingBatch::from_recipe(batch, local_origin.metres(), detail, meshes)
            })
            .collect()
    };
    let mut compiled = CompiledBuildingLevels {
        facade_openings: if detail == BuildingDetail::Dynamic {
            plan.facade_dynamic_openings()?
        } else {
            Default::default()
        },
        interior: (detail == BuildingDetail::Dynamic).then(|| {
            super::interior_lighting::InteriorField::from_plan(plan, local_origin.metres())
        }),
        program: program.clone(),
        detail,
        local_origin,
        sign_sites: if program
            .usage
            .and_then(adventuresim_building_generator::signs::shop_trade)
            .is_some()
        {
            signs::sites(plan)
        } else {
            Vec::new()
        },
        lod0: detail_meshes.map_or_else(Vec::new, |detail| compile_batches(&detail.meshes, meshes)),
        lod1: compile_batches(&facade.meshes, meshes),
        lod2: compile_batches(&shell.meshes, meshes),
    };
    if let Some(kit) = kit {
        cache
            .components
            .append(&kit.instances, &mut compiled, meshes);
    }
    let compiled = Arc::new(compiled);
    cache.levels.push(compiled.clone());
    Ok(compiled)
}

fn spawn_building_levels(
    parent: &mut ChildSpawnerCommands,
    building_id: u64,
    compiled: &CompiledBuildingLevels,
    materials: &TacticalBuildingMaterials,
) {
    let palette = materials.for_building(building_id);
    for (level, batches) in [
        (BuildingRenderLevel::Lod0, &compiled.lod0),
        (BuildingRenderLevel::Lod1, &compiled.lod1),
        (BuildingRenderLevel::Lod2, &compiled.lod2),
    ] {
        for batch in batches {
            parent.spawn((
                Name::new(format!("Building {:?} {:?}", level, batch.material)),
                PresentedBuildingMesh {
                    level,
                    material: batch.material,
                    triangles: batch.triangles,
                },
                Mesh3d(batch.mesh.clone()),
                Transform::from_matrix(batch.transform),
                MeshMaterial3d(palette.get(batch.material)),
                if matches!(
                    (compiled.detail, level),
                    (BuildingDetail::Facade, BuildingRenderLevel::Lod1)
                ) {
                    VisibilityRange {
                        start_margin: 0.0..0.0,
                        ..building_lod_visibility(level)
                    }
                } else {
                    building_lod_visibility(level)
                },
            ));
        }
    }
}

pub(super) fn building_lod_visibility(level: BuildingRenderLevel) -> VisibilityRange {
    match level {
        BuildingRenderLevel::Lod0 => VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: DETAIL_LOD_END_START_METRES..DETAIL_LOD_END_END_METRES,
            use_aabb: false,
        },
        BuildingRenderLevel::Lod1 => VisibilityRange {
            start_margin: DETAIL_LOD_END_START_METRES..DETAIL_LOD_END_END_METRES,
            end_margin: FACADE_LOD_END_START_METRES..FACADE_LOD_END_END_METRES,
            use_aabb: false,
        },
        BuildingRenderLevel::Lod2 => VisibilityRange {
            start_margin: FACADE_LOD_END_START_METRES..FACADE_LOD_END_END_METRES,
            // The generated settlement is already finite. Keeping its cheapest
            // shell visible avoids cutting the far half of a city from elevated
            // or exterior viewpoints.
            end_margin: f32::MAX..f32::MAX,
            use_aabb: false,
        },
    }
}

struct RecipeMeshes<'a> {
    kit: Option<adventuresim_building_generator::BuildingKit<'a>>,
    detail_meshes: Option<adventuresim_building_generator::BuildingDetail>,
    facade: adventuresim_building_generator::BuildingLod,
}
impl<'a> RecipeMeshes<'a> {
    fn compile(
        plan: &'a adventuresim_building_generator::BuildingPlan,
        detail: BuildingDetail,
    ) -> Result<Self> {
        Ok(match detail {
            BuildingDetail::Dynamic => Self {
                kit: None,
                detail_meshes: Some(compile_static_building_detail(plan)?),
                facade: compile_static_building_lod(plan, BuildingLodLevel::Facade)?,
            },
            BuildingDetail::Static | BuildingDetail::Facade => {
                let kit = adventuresim_building_generator::BuildingKit::new(plan)?;
                let meshes = if detail == BuildingDetail::Static {
                    Some(kit.detail()?)
                } else {
                    None
                };
                let facade = kit.facade()?;
                Self {
                    kit: Some(kit),
                    detail_meshes: meshes,
                    facade,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use adventuresim_building_generator::{LodVertex, compile_building_collision, generate};

    use super::recipe_mesh::recipe_mesh;
    use super::*;

    #[test]
    fn prepared_geometry_and_replicated_recipes_produce_the_same_meshes() {
        let program = BuildingProgram::fixture(
            adventuresim_building_generator::BuildingArchetype::TownHouse,
            42,
        );
        let plan = generate(&program).unwrap();
        let collision = compile_building_collision(&plan).unwrap();
        let prepared = GeneratedBuildingRecipe {
            program: program.clone(),
            plan,
            collision,
        };
        let mut meshes = Assets::default();
        let mut cache = TacticalBuildingMeshCache::default();
        let local = cached_building_levels(
            &mut cache,
            &program,
            BuildingDetail::Dynamic,
            &mut meshes,
            Some(&prepared),
        )
        .unwrap();
        cache.levels.clear();
        let remote = cached_building_levels(
            &mut cache,
            &program,
            BuildingDetail::Dynamic,
            &mut meshes,
            None,
        )
        .unwrap();
        assert_eq!(local.local_origin, remote.local_origin);
        assert_eq!(local.facade_openings, remote.facade_openings);
        assert!(local.interior.is_some());
        for (local, remote) in [
            (&local.lod0, &remote.lod0),
            (&local.lod1, &remote.lod1),
            (&local.lod2, &remote.lod2),
        ] {
            assert_eq!(local.len(), remote.len());
            for (local, remote) in local.iter().zip(remote) {
                assert!(
                    meshes
                        .get(&local.mesh)
                        .unwrap()
                        .asset_usage
                        .contains(RenderAssetUsages::RENDER_WORLD)
                );
                assert_eq!(local.material, remote.material);
                assert_eq!(local.triangles, remote.triangles);
                let positions = |mesh: &Handle<Mesh>| {
                    meshes
                        .get(mesh)
                        .unwrap()
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap()
                };
                assert_eq!(positions(&local.mesh), positions(&remote.mesh));
            }
        }
        let facade = cached_building_levels(
            &mut cache,
            &program,
            BuildingDetail::Facade,
            &mut meshes,
            Some(&prepared),
        )
        .unwrap();
        assert!(facade.interior.is_none());
        assert!(facade.facade_openings.is_empty());
        assert!(facade.lod0.is_empty());
        assert!(!facade.lod1.is_empty() && !facade.lod2.is_empty());
        for batch in facade.lod1.iter().chain(&facade.lod2) {
            assert_eq!(
                meshes.get(&batch.mesh).unwrap().asset_usage,
                RenderAssetUsages::MAIN_WORLD
            );
        }
    }

    #[test]
    fn city_consumes_prepared_recipes_and_reuses_resident_meshes() {
        let program = BuildingProgram::fixture(
            adventuresim_building_generator::BuildingArchetype::TownHouse,
            42,
        );
        let mut meshes = Assets::default();
        let mut cache = TacticalBuildingMeshCache::default();
        cache.recipes.get_or_generate(&program).unwrap();
        let first = cached_building_levels(
            &mut cache,
            &program,
            BuildingDetail::Facade,
            &mut meshes,
            None,
        )
        .unwrap();
        assert!(
            cache.recipes.take(&program).is_none(),
            "rendering must consume the prepared plan"
        );
        let resident_count = meshes.len();
        let second = cached_building_levels(
            &mut cache,
            &program,
            BuildingDetail::Facade,
            &mut meshes,
            None,
        )
        .unwrap();
        assert_eq!(meshes.len(), resident_count);
        for (first, second) in first
            .lod1
            .iter()
            .zip(&second.lod1)
            .chain(first.lod2.iter().zip(&second.lod2))
        {
            assert_eq!(first.mesh, second.mesh);
        }
    }

    #[test]
    fn shell_lod_has_no_artificial_distance_cutoff() {
        let visibility = building_lod_visibility(BuildingRenderLevel::Lod2);
        assert_eq!(visibility.end_margin, f32::MAX..f32::MAX);
    }

    #[test]
    fn interior_plaster_mesh_carries_tangent_space_for_its_bound_normal_map() {
        let batch = LodMesh {
            material: BuildingLodMaterial::InteriorPlaster,
            vertices: vec![
                LodVertex {
                    position: Vec3::ZERO,
                    normal: Vec3::Z,
                    uv: Vec2::ZERO,
                },
                LodVertex {
                    position: Vec3::X,
                    normal: Vec3::Z,
                    uv: Vec2::X,
                },
                LodVertex {
                    position: Vec3::X + Vec3::Y,
                    normal: Vec3::Z,
                    uv: Vec2::ONE,
                },
                LodVertex {
                    position: Vec3::Y,
                    normal: Vec3::Z,
                    uv: Vec2::Y,
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };

        let mesh = recipe_mesh(&batch, Vec3::ZERO);

        assert!(mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_some());
    }
}
