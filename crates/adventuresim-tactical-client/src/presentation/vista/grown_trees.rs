//! Coarse grown instances reuse the active catalogue and source allocations.
use super::tree_sites::{
    VistaTreeBand, VistaTreeCandidate, VistaTreePlacement, vista_tree_visibility,
};
use super::trees::VistaTreeCensus;
use super::*;
use adventuresim_environment::render::{Level, grown::GrownMaterialHandle};
use anyhow::{Context, Result, ensure};
use bevy::camera::visibility::NoAutoAabb;
use fabelgeist_math::units::Metres;

/// A render-only regional part, separate from playable physical-root streams.
#[derive(Component, Clone)]
pub(crate) struct GrownVistaPart(pub(crate) GrownPartBinding);

pub(super) fn spawn(
    commands: &mut Commands,
    scene: Entity,
    band: &VistaTreeBand<'_>,
    sites: &[VistaTreePlacement],
    catalogue: &GrownCatalogueAssets,
) -> Result<VistaTreeCensus> {
    let mut census = VistaTreeCensus::default();
    for site in sites {
        let source = catalogue.select(site.origin.position());
        let assets = catalogue.assets(source, Level::Coarse)?;
        let model = assets
            .bank()
            .library()
            .models
            .get(source.variant.index())
            .context("vista grown source bounds")?;
        let height =
            Metres::from_metres(model.bounds.high[1].metres() - model.bounds.low[1].metres());
        ensure!(
            height.metres().is_finite() && height.metres() > 0.0,
            "invalid vista grown height"
        );
        let range = vista_tree_visibility(
            Metres::from_metres(band.lod.spacing_metres),
            height,
            site.scale,
        );
        let root = commands
            .spawn((
                Name::new("Distant grown tree"),
                VistaTerrain(band.lod.level),
                VistaTreePresentation,
                VistaTreeCandidate(site.seed),
                site.origin.frame(site.scale),
                GlobalTransform::default(),
                Visibility::Inherited,
            ))
            .id();
        census.roots.push(root);
        for part in assets.parts() {
            let mut child = commands.spawn((
                GrownVistaPart(GrownPartBinding {
                    scene,
                    root,
                    bank: assets.bank().id(),
                    source,
                    level: Level::Coarse,
                    role: part.role(),
                    records: part.records(),
                }),
                Mesh3d(part.mesh().clone()),
                part.bounds(),
                NoAutoAabb,
                range.clone(),
                NotShadowCaster,
                Transform::IDENTITY,
                GlobalTransform::default(),
                Visibility::Inherited,
                ChildOf(root),
            ));
            match part.material() {
                GrownMaterialHandle::Wood(material) => {
                    child.insert(MeshMaterial3d(material.clone()));
                }
                GrownMaterialHandle::Leaves(material) => {
                    child.insert(MeshMaterial3d(material.clone()));
                }
            }
            census.parts.push(child.id());
        }
    }
    Ok(census)
}
