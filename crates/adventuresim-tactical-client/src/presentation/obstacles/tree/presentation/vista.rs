//! Shared terminal vista variant construction and prepared-card resolution.

use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "vista tree preparation receives independent asset stores and the optional fixed-scene prebake"
)]
pub(in crate::presentation) fn ensure_vista_tree_variant(
    variant_seed: Seed,
    competition: f32,
    species: TreePresentationSpecies,
    meshes: &mut Assets<Mesh>,
    tree_materials: &mut Assets<TacticalTreeImpostorMaterial>,
    images: &mut Assets<Image>,
    cache: &mut VistaTreePresentationCache,
    prepared: Option<&PreparedTreeImpostorAsset>,
) -> CachedVistaTreePresentation {
    let competition_key = (competition * 4095.0).round() as u64;
    let cache_key = StreamId::new("visual.tree.vista-cache-key")
        .seed(variant_seed, &[competition_key, species.cache_salt()]);
    if let Some(cached) = cache.variants.get(&cache_key) {
        return cached.clone();
    }
    let (branches, leaves) = vista_tree_source(variant_seed, competition, species);
    let bake = if let Some(prepared) = prepared {
        prepared
            .resolve(PreparedTreeImpostorUsage::Vista, species, variant_seed, 4)
            .unwrap_or_else(|| {
                panic!("prepared tree impostors do not match the requested vista tree")
            })
    } else if species == TreePresentationSpecies::EnglishOak {
        bake_tree_lod(variant_seed, &branches, &leaves, 4)
    } else {
        bake_tree_lod_with_style(variant_seed, &branches, &leaves, 4, species.bake_style())
    };
    validate_tree_bake_provenance(&bake.provenance);
    let texture = images.add(bake.image.clone());
    let cached = CachedVistaTreePresentation {
        mesh: meshes.add(bake.mesh),
        material: tree_materials.add(tree_impostor_material(variant_seed, 4, texture)),
        provenance: bake.provenance,
    };
    cache.variants.insert(cache_key, cached.clone());
    cached
}
