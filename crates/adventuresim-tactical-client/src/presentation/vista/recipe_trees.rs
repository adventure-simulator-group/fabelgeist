//! Existing calibrated card assets consuming the shared placement owner.
use super::tree_sites::{
    VistaTreeBand, VistaTreeCandidate, VistaTreePlacement, vista_tree_visibility,
};
use super::trees::VistaTreeCensus;
use super::*;
use fabelgeist_math::units::Metres;

#[expect(
    clippy::too_many_arguments,
    reason = "the card consumer borrows independent Bevy asset stores"
)]
pub(super) fn spawn(
    commands: &mut Commands,
    band: &VistaTreeBand<'_>,
    sites: &[VistaTreePlacement],
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<TacticalTreeImpostorMaterial>,
    images: &mut Assets<Image>,
    cache: &mut VistaTreePresentationCache,
    prepared: Option<&PreparedTreeImpostorAsset>,
) -> VistaTreeCensus {
    let mut census = VistaTreeCensus::default();
    for site in sites {
        let variant_seed = crate::presentation::obstacles::tree::specimen::oak_variant_seed(0);
        let species = vista_tree_species(Some(band.environment), site.origin.position().xz());
        let cached = ensure_vista_tree_variant(
            variant_seed,
            0.5,
            species,
            meshes,
            materials,
            images,
            cache,
            prepared,
        );
        let height = cached
            .provenance
            .records
            .first()
            .map_or(12.0, |record| record.projected_bounds.w);
        let entity = commands
            .spawn((
                Name::new(format!("Distant vista {} billboard", species.name())),
                VistaTerrain(band.lod.level),
                VistaTreePresentation,
                VistaTreeCandidate(site.seed),
                billboard_bounds(height),
                NotShadowCaster,
                Mesh3d(cached.mesh.clone()),
                MeshMaterial3d(cached.material.clone()),
                cached.provenance.clone(),
                vista_tree_visibility(
                    Metres::from_metres(band.lod.spacing_metres),
                    Metres::from_metres(height),
                    site.scale,
                ),
                site.origin.frame(site.scale),
            ))
            .id();
        census.roots.push(entity);
        census.parts.push(entity);
    }
    census
}

fn billboard_bounds(card_height: f32) -> bevy::camera::primitives::Aabb {
    bevy::camera::primitives::Aabb {
        center: bevy::math::Vec3A::new(0.0, card_height * 0.5, 0.0),
        half_extents: bevy::math::Vec3A::new(
            card_height * 0.8,
            card_height * 0.6,
            card_height * 0.8,
        ),
    }
}
