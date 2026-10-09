//! Admit complete authored litter meshes before publishing retained handles.
use super::*;
use crate::presentation::ground_scatter::GroundFoliagePresentationCache;
use bevy::asset::Assets as MeshAssets;

pub(in crate::presentation::ground_scatter) struct PatchMeshes {
    pub dry_leaves: Vec<Handle<Mesh>>,
    pub twigs: Vec<Handle<Mesh>>,
    pub plants: Vec<Handle<Mesh>>,
}

enum PreparedFamily {
    Retained(Vec<Handle<Mesh>>),
    Generated(Vec<Mesh>),
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
        meshes: &mut MeshAssets<Mesh>,
    ) -> Vec<Handle<Mesh>> {
        match self {
            Self::Retained(handles) => handles,
            Self::Generated(generated) => {
                let handles: Vec<_> = generated.into_iter().map(|mesh| meshes.add(mesh)).collect();
                *cache = Some(handles.clone());
                handles
            }
        }
    }
}

pub(in crate::presentation::ground_scatter) fn prepare_meshes(
    cache: &mut GroundFoliagePresentationCache,
    meshes: &mut MeshAssets<Mesh>,
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
        dry_leaves: dry_leaves.install(&mut cache.dry_leaf_meshes, meshes),
        twigs: twigs.install(&mut cache.twig_meshes, meshes),
        plants: plants.install(&mut cache.woodland_plant_meshes, meshes),
    })
}
