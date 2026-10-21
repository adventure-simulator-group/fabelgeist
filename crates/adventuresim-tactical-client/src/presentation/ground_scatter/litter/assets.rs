//! Admit complete authored litter meshes before publishing retained handles.
use super::*;
use crate::presentation::ground_scatter::{GroundFoliagePresentationCache, foliage_material};
use bevy::asset::{Assets as BevyAssets, RenderAssetUsages};

const WOODLAND_PLANT_WIND_SCALE: f32 = 0.035;

struct PatchMeshes {
    pub dry_leaves: Vec<Handle<Mesh>>,
    pub twigs: Vec<Handle<Mesh>>,
    pub plants: Vec<Handle<Mesh>>,
}

enum PreparedFamily {
    Retained(Vec<Handle<Mesh>>),
    Generated(Vec<Mesh>),
}
impl Assets {
    pub(in crate::presentation::ground_scatter) fn prepare(
        cache: &mut GroundFoliagePresentationCache,
        meshes: &mut BevyAssets<Mesh>,
        materials: &mut BevyAssets<TacticalFoliageMaterial>,
        standard_materials: &mut BevyAssets<StandardMaterial>,
        leaf_materials: &mut BevyAssets<TacticalTreeLeafCardMaterial>,
        procedural_assets: &ProceduralTextureAssets,
    ) -> GeometryResult<Self> {
        let PatchMeshes {
            dry_leaves,
            twigs,
            plants,
        } = prepare_meshes(cache, meshes)?;
        let dry_leaf_material = cache
            .forest_floor_leaves
            .get_or_insert_with(|| {
                leaf_materials.add(forest_floor_leaf_material(procedural_assets))
            })
            .clone();
        let twig_material = cache
            .twig_material
            .get_or_insert_with(|| standard_materials.add(static_twig_material()))
            .clone();
        let woodland_plant_material = cache
            .woodland_plant_material
            .get_or_insert_with(|| {
                materials.add(foliage_material(WOODLAND_PLANT_WIND_SCALE, false))
            })
            .clone();
        Ok(Self {
            dry_leaf_meshes: dry_leaves,
            twig_meshes: twigs,
            dry_leaf_material,
            twig_material,
            woodland_plant_meshes: plants,
            woodland_plant_material,
        })
    }
}

impl PreparedFamily {
    fn prepare(
        existing: &Option<Vec<Handle<Mesh>>>,
        variant_count: u64,
        generate: fn(u64) -> GeometryResult<Mesh>,
    ) -> GeometryResult<Self> {
        if let Some(existing) = existing {
            return Ok(Self::Retained(existing.clone()));
        }
        // Native catalogue ordinals are admitted to checked recipe geometry.
        (0..variant_count)
            .map(generate)
            .collect::<GeometryResult<Vec<_>>>()
            .map(Self::Generated)
    }
    fn install(
        self,
        cache: &mut Option<Vec<Handle<Mesh>>>,
        meshes: &mut BevyAssets<Mesh>,
        usage: RenderAssetUsages,
    ) -> Vec<Handle<Mesh>> {
        match self {
            Self::Retained(handles) => handles,
            Self::Generated(generated) => {
                let handles: Vec<_> = generated
                    .into_iter()
                    .map(|mut mesh| {
                        mesh.asset_usage = usage;
                        meshes.add(mesh)
                    })
                    .collect();
                *cache = Some(handles.clone());
                handles
            }
        }
    }
}

fn prepare_meshes(
    cache: &mut GroundFoliagePresentationCache,
    meshes: &mut BevyAssets<Mesh>,
) -> GeometryResult<PatchMeshes> {
    // Validate every missing family before changing any retained cache entry.
    let dry_leaves = PreparedFamily::prepare(
        &cache.dry_leaf_meshes,
        DRY_LEAF_MESH_VARIANTS,
        dry_leaf_patch_mesh,
    )?;
    let twigs = PreparedFamily::prepare(&cache.twig_meshes, TWIG_MESH_VARIANTS, twig_patch_mesh)?;
    let plants = PreparedFamily::prepare(
        &cache.woodland_plant_meshes,
        WOODLAND_PLANT_MESH_VARIANTS,
        woodland_plant_patch_mesh,
    )?;
    Ok(PatchMeshes {
        dry_leaves: dry_leaves.install(
            &mut cache.dry_leaf_meshes,
            meshes,
            RenderAssetUsages::all(),
        ),
        twigs: twigs.install(&mut cache.twig_meshes, meshes, RenderAssetUsages::all()),
        plants: plants.install(
            &mut cache.woodland_plant_meshes,
            meshes,
            RenderAssetUsages::MAIN_WORLD,
        ),
    })
}
